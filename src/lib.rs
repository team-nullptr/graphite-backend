use std::{path::PathBuf, sync::Arc};

use axum::{routing::get, Router};
use clap::Parser;
use config::Config;
use hyper::{
    client::HttpConnector,
    header::{ACCEPT, AUTHORIZATION, CONTENT_LENGTH, CONTENT_TYPE},
    http::HeaderValue,
    Client, Method,
};
use hyper_tls::HttpsConnector;
use projects::handlers::projects_resource;
use sqlx::{mysql::MySqlPoolOptions, MySqlPool};
use tower_http::{
    cors::CorsLayer,
    trace::{self, TraceLayer},
};
use tracing::{event, Level};

mod api;
mod auth;
mod config;
mod github;
mod projects;
mod server;

#[derive(Clone)]
pub struct AppState {
    // TODO: Arc? Does it make any sense?
    config: Arc<config::Config>,
    project_service: Arc<dyn projects::service::ProjectServiceExt + Send + Sync>,
    session_manager: Arc<auth::session::SessionManager>,
    github_service: Arc<github::GitHubService>,
}

/// Gets app config.
fn get_config() -> Result<Arc<config::Config>, Box<dyn std::error::Error>> {
    let args = config::Args::parse();
    Ok(Arc::from(Config::load(args.config)?))
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
    tracing_subscriber::fmt()
        .with_target(false)
        .compact()
        .init();

    let config = get_config()?;
    let app_state = build_app_state(config.clone()).await;

    let cors = CorsLayer::new()
        .allow_headers([AUTHORIZATION, ACCEPT, CONTENT_TYPE, CONTENT_LENGTH])
        .allow_methods([Method::GET, Method::POST])
        .allow_origin(config.general.client_addr.parse::<HeaderValue>().unwrap())
        .allow_credentials(true);

    let app = Router::new()
        .route("/ping", get(|| async { "pong" }))
        .route("/auth/github", get(auth::handlers::github_oauth_redirect))
        .nest("/projects", projects_resource(app_state.clone()))
        .with_state(app_state)
        .layer(cors)
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(trace::DefaultMakeSpan::new().level(Level::INFO))
                .on_response(trace::DefaultOnResponse::new().level(Level::INFO)),
        );

    let cert_config = server::CertConfig {
        key: PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(&config.server.tls_key),
        cert: PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(&config.server.tls_cert),
    };

    let addr = format!("{}:{}", config.server.address, config.server.port);
    event!(Level::INFO, "Server starting at https://{}", addr);

    server::listen(addr, cert_config, app.into_make_service()).await;

    Ok(())
}

/// Assembles the app state.
async fn build_app_state(config: Arc<Config>) -> Arc<AppState> {
    let db = connect_to_database(&config).await;
    let https_client = get_https_client();

    let project_repo = Arc::from(projects::repo::ProjectRepo::new(db.clone()));
    let session_manager = Arc::from(auth::session::SessionManager::new(db.clone()));

    let github_service = Arc::from(github::GitHubService::new(https_client));
    let project_service = Arc::from(projects::service::ProjectService::new(project_repo));

    Arc::from(AppState {
        config,
        project_service,
        session_manager,
        github_service,
    })
}
