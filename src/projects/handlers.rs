use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};

use super::model::{Project, ProjectCreate};
use crate::{ApiError, AppState};

pub async fn create_project(
    State(app_state): State<AppState>,
    Json(project_create): Json<ProjectCreate>,
) -> Result<(StatusCode, Json<Project>), ApiError> {
    let result = app_state.project_service.create(project_create).await;

    match result {
        Ok(created_project) => Ok((StatusCode::OK, Json(created_project))),
        Err(e) => match e {
            _ => Err(ApiError::Internal(e.into())),
        },
    }
}

pub async fn get_all_projects(State(app_state): State<AppState>) -> impl IntoResponse {
    let result = app_state.project_service.get_all().await;

    match result {
        Ok(projects) => Ok((StatusCode::OK, Json(projects))),
        Err(e) => match e {
            _ => Err(ApiError::Internal(e.into())),
        },
    }
}
