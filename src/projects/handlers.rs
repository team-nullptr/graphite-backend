use axum::{extract::State, http::StatusCode, response::IntoResponse, Extension, Json};

use super::model::{Project, ProjectCreate};
use crate::{api, auth::session::Session, AppState};

/// Handler for creating a new project.
pub async fn create_project(
    State(app_state): State<AppState>,
    session: Extension<Session>,
    Json(project_create): Json<ProjectCreate>,
) -> Result<(StatusCode, Json<Project>), api::Error> {
    println!("{:?}", session);

    let result = app_state.project_service.create(project_create).await;

    match result {
        Ok(created_project) => Ok((StatusCode::OK, Json(created_project))),
        Err(e) => Err(api::Error::Internal(e.into())),
    }
}

/// Handler for getting all user's projects.
pub async fn get_all_projects(
    State(app_state): State<AppState>,
    session: Extension<Session>,
) -> impl IntoResponse {
    println!("session: {:?}", session);

    let result = app_state.project_service.get_all().await;

    match result {
        Ok(projects) => Ok((StatusCode::OK, Json(projects))),
        Err(e) => Err(api::Error::Internal(e.into())),
    }
}
