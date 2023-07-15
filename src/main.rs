use std::{error::Error, net::SocketAddr, sync::Arc};

use axum::{
    extract::{Json, State},
    http::StatusCode,
    response::IntoResponse,
    routing::post,
    Router,
};

mod projects;

pub struct AppError(anyhow::Error);

impl<E> From<E> for AppError
where
    E: Into<anyhow::Error>,
{
    fn from(err: E) -> Self {
        Self(err.into())
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Something went wrong: {}", self.0),
        )
            .into_response()
    }
}

async fn create_project_handler(
    State(app_state): State<Arc<AppState>>,
    Json(project_create): Json<projects::model::ProjectCreate>,
) -> Result<Json<projects::model::Project>, AppError> {
    let created_project = projects::repo::create_project(&app_state.pool, project_create).await?;
    Ok(Json(created_project))
}

pub struct AppState {
    pool: sqlx::PgPool,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let url = "postgres://postgres:dev@localhost:5432/graphite";
    let pool = sqlx::postgres::PgPool::connect(url).await?;

    sqlx::migrate!("./migrations").run(&pool).await?;

    let app_state = Arc::new(AppState { pool });
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));

    let app = Router::new()
        .route("/projects", post(create_project_handler))
        .with_state(app_state);

    println!("Listening on {}", addr);

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await?;

    Ok(())
}
