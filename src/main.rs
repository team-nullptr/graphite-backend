use axum::{routing::get, Router};
use std::{error::Error, net::SocketAddr};

mod error;
mod projects;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let db_conn_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:dev@localhost:5432/graphite".to_string());

    // TODO: Look into PgPoolOptions
    let pool = sqlx::postgres::PgPool::connect(&db_conn_url)
        .await
        .expect("can't connect to the database");

    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("can't run database migrations");

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));

    let app = Router::new()
        .route(
            "/projects",
            get(projects::routes::get_all_projects).post(projects::routes::create_project),
        )
        .with_state(pool);

    println!("Listening on {}", addr);

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await?;

    Ok(())
}
