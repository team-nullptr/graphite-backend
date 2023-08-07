use axum::{http::StatusCode, response::IntoResponse, routing::get, Router};
use clap::Parser;
use hyper::{client::HttpConnector, Client};
use hyper_tls::HttpsConnector;
use sqlx::{postgres::PgPoolOptions, PgPool};
use std::sync::Arc;
use thiserror::Error;
use tower_http::trace::{self, TraceLayer};
use tracing::Level;

mod auth;
mod config;
mod github;
mod projects;

#[derive(Error, Debug)]
pub enum ApiError {
    #[error("Internal error has occured")]
    Internal(#[source] anyhow::Error),
}

impl IntoResponse for ApiError {
    fn into_response(self) -> axum::response::Response {
        match self {
            Self::Internal(e) => {
                println!("{:?}", e);
                StatusCode::INTERNAL_SERVER_ERROR.into_response()
            }
        }
    }
}

#[derive(Clone)]
pub struct AppState {
    config: Arc<config::Config>,
    project_service: projects::service::ProjectService,
    user_service: auth::service::UserService,
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
async fn connect_database(config: &config::Config) -> PgPool {
    let pool = PgPoolOptions::new()
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
    let addr = format!("{}:{}", config.server.address, config.server.port);

    let db = connect_database(&config).await;
    let https_client = get_https_client();

    let projects_repo = projects::repo::ProjectRepo::new(db.clone());
    let project_service = projects::service::ProjectService::new(projects_repo);

    let user_repo = auth::repo::UserRepo::new(db.clone());
    let user_service = auth::service::UserService::new(user_repo);

    let app = Router::new()
        .route("/auth/github", get(auth::handlers::github_oauth_redirect))
        .route(
            "/projects",
            get(projects::handlers::get_all_projects).post(projects::handlers::create_project),
        )
        .with_state(AppState {
            config: Arc::new(config),
            project_service,
            user_service,
            github_service: github::GitHubService::new(https_client),
        })
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(trace::DefaultMakeSpan::new().level(Level::INFO))
                .on_response(trace::DefaultOnResponse::new().level(Level::INFO)),
        );

    println!("Server starting at {}", addr);
    axum::Server::bind(&addr.parse()?)
        .serve(app.into_make_service())
        .await
        .expect("failed to start the server");

    Ok(())
}
