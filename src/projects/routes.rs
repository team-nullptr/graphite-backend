use axum::{extract::State, Json};
use sqlx::PgPool;

use super::{
    model::{Project, ProjectCreate},
    repo,
};
use crate::error::AppError;

pub async fn create_project(
    State(pool): State<PgPool>,
    Json(project_create): Json<ProjectCreate>,
) -> Result<Json<Project>, AppError> {
    Ok(Json(repo::create_project(&pool, project_create).await?))
}

pub async fn get_all_projects(State(pool): State<PgPool>) -> Result<Json<Vec<Project>>, AppError> {
    Ok(Json(repo::get_all_projects(&pool).await?))
}
