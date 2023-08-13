use std::{
    fs::File,
    io::BufReader,
    path::{Path, PathBuf},
    pin::Pin,
    sync::Arc,
};

use axum::{routing::get, Router};
use clap::Parser;
use futures_util::future::poll_fn;
use hyper::{
    client::HttpConnector,
    header::{ACCEPT, AUTHORIZATION, CONTENT_LENGTH, CONTENT_TYPE},
    http::HeaderValue,
    server::{
        accept::Accept,
        conn::{AddrIncoming, Http},
    },
    Client, Method, Request,
};
use hyper_tls::HttpsConnector;
use rustls_pemfile::{certs, pkcs8_private_keys};
use sqlx::{mysql::MySqlPoolOptions, MySqlPool};
use tokio::net::TcpListener;
use tokio_rustls::{
    rustls::{Certificate, PrivateKey, ServerConfig},
    TlsAcceptor,
};
use tower::MakeService;
use tower_http::{
    cors::CorsLayer,
    trace::{self, TraceLayer},
};
use tracing::Level;

mod api;
mod auth;
mod config;
mod github;
mod projects;

#[derive(Clone)]
pub struct AppState {
    // TODO: Arc? Does it make any sense?
    config: Arc<config::Config>,
    project_service: projects::service::ProjectService,
    session_manager: auth::session::SessionManager,
    github_service: github::GitHubService,
}

/// Gets app config.
fn get_config() -> Result<config::Config, Box<dyn std::error::Error>> {
    let args = config::Args::parse();
    Ok(config::Config::load(args.config)?)
}

/// Creates a new https client.
fn get_https_client() -> Client<HttpsConnector<HttpConnector>> {
    let https = HttpsConnector::new();
    Client::builder().build::<_, hyper::Body>(https)
}

/// Connects with postgres database.
async fn connect_to_database(config: &config::Config) -> MySqlPool {
    let pool = MySqlPoolOptions::new()
        .max_connections(5)
        .connect(&config.database.connection)
        .await
        .expect("failed to connect with database");

    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("can't run database migrations");

    pool
}

/// Entrypoint to the api server. Reads configuration, establishes db connection and starts api.
pub async fn start_app() -> Result<(), Box<dyn std::error::Error>> {
    // Setup tracing
    tracing_subscriber::fmt()
        .with_target(false)
        .compact()
        .init();

    // Get config
    let config = get_config()?;
    let addr = format!("{}:{}", config.server.address, config.server.port);
    let db = connect_to_database(&config).await;

    let https_client = get_https_client();

    // Setup repositories/services etc.
    let session_manager = auth::session::SessionManager::new(db.clone());
    let github_service = github::GitHubService::new(https_client);

    let project_repo = projects::repo::ProjectRepo::new(db.clone());
    let project_service = projects::service::ProjectService::new(project_repo);

    // Register REST handlers
    let app_state = AppState {
        config: Arc::new(config),
        project_service,
        session_manager,
        github_service,
    };

    let cors = CorsLayer::new()
        .allow_headers([AUTHORIZATION, ACCEPT, CONTENT_TYPE, CONTENT_LENGTH])
        .allow_methods([Method::GET, Method::POST])
        // TODO: Allow to configure client addr
        .allow_origin("http://localhost:5173".parse::<HeaderValue>().unwrap())
        .allow_credentials(true);

    // Routing
    let projects_route = Router::new().route(
        "/",
        get(projects::handlers::get_all_projects)
            .post(projects::handlers::create_project)
            .layer(auth::session::SessionManager::new(db.clone())),
    );

    let mut app = Router::new()
        .route("/auth/github", get(auth::handlers::github_oauth_redirect))
        .nest("/projects", projects_route)
        .with_state(app_state)
        .layer(cors)
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(trace::DefaultMakeSpan::new().level(Level::INFO))
                .on_response(trace::DefaultOnResponse::new().level(Level::INFO)),
        )
        .into_make_service();

    // Start the server
    let rustls_config = rustls_server_config(
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("certs")
            .join("graphite.test+3-key.pem"),
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("certs")
            .join("graphite.test+3.pem"),
    );

    let protocol = Arc::new(Http::new());
    let mut listener =
        AddrIncoming::from_listener(TcpListener::bind(&addr).await.unwrap()).unwrap();
    let acceptor = TlsAcceptor::from(rustls_config);

    loop {
        let stream = poll_fn(|cx| Pin::new(&mut listener).poll_accept(cx))
            .await
            .unwrap()
            .unwrap();

        let acceptor = acceptor.clone();
        let protocol = protocol.clone();
        let svc = MakeService::<_, Request<hyper::Body>>::make_service(&mut app, &stream);

        tokio::spawn(async move {
            if let Ok(stream) = acceptor.accept(stream).await {
                let _ = protocol.serve_connection(stream, svc.await.unwrap()).await;
            }
        });
    }
}

fn rustls_server_config(key: impl AsRef<Path>, cert: impl AsRef<Path>) -> Arc<ServerConfig> {
    let mut key_reader = BufReader::new(File::open(key).unwrap());
    let mut cert_reader = BufReader::new(File::open(cert).unwrap());

    let key = PrivateKey(pkcs8_private_keys(&mut key_reader).unwrap().remove(0));
    let certs = certs(&mut cert_reader)
        .unwrap()
        .into_iter()
        .map(Certificate)
        .collect();

    let mut config = ServerConfig::builder()
        .with_safe_defaults()
        .with_no_client_auth()
        .with_single_cert(certs, key)
        .expect("Invalid certificate or key");

    config.alpn_protocols = vec![b"h2".to_vec(), b"http/1.1".to_vec()];

    Arc::new(config)
}
