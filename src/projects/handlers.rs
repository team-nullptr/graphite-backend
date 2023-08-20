use std::sync::Arc;

use axum::{
    extract::{Path, State},
    http::StatusCode,
    middleware,
    response::IntoResponse,
    routing::get,
    Extension, Json, Router,
};

use super::model::{ProjectCreate, ProjectUpdate};
use crate::{
    api,
    auth::session::{auth_middleware, Session},
    projects::service::ServiceError,
    AppState,
};

/// Returns router for /projects resource.
pub fn projects_resource(app_state: Arc<AppState>) -> Router<Arc<AppState>> {
    Router::new()
        .route("/", get(find_all_for_user).post(create))
        .route("/:id", get(find_for_user).patch(update))
        .route_layer(middleware::from_fn_with_state(
            app_state.clone(),
            auth_middleware,
        ))
}

/// Handler for getting all user's projects.
pub async fn find_all_for_user(
    State(app_state): State<Arc<AppState>>,
    session: Extension<Session>,
) -> impl IntoResponse {
    let result = app_state
        .project_service
        .find_all_for_user(session.user_id)
        .await;

    match result {
        Ok(projects) => Ok((StatusCode::OK, Json(projects))),
        Err(e) => Err(api::Error::Internal(e.into())),
    }
}

/// Handler for getting project by id.
/// Requires valid session.
pub async fn find_for_user(
    State(app_state): State<Arc<AppState>>,
    Path(project_id): Path<u64>,
    session: Extension<Session>,
) -> impl IntoResponse {
    let result = app_state
        .project_service
        .find_for_user(session.user_id, project_id)
        .await;

    match result {
        Ok(project) => match project {
            Some(project) => Ok((StatusCode::OK, Json(project))),
            None => Err(api::Error::NotFound),
        },
        Err(e) => Err(api::Error::Internal(e.into())),
    }
}

/// Handler for creating a new project.
/// Requires valid session.
pub async fn create(
    State(app_state): State<Arc<AppState>>,
    session: Extension<Session>,
    Json(project_create): Json<ProjectCreate>,
) -> impl IntoResponse {
    let create_result = app_state
        .project_service
        .create(session.user_id, project_create)
        .await;

    match create_result {
        Ok(created_project) => Ok((StatusCode::OK, Json(created_project))),
        Err(e) => match e {
            ServiceError::ValidationError(e) => Err(api::Error::ValidationError(e)),
            e => Err(api::Error::Internal(e.into())),
        },
    }
}

/// Handler for updating project by id.
/// Requires valid session.
pub async fn update(
    State(app_state): State<Arc<AppState>>,
    Path(project_id): Path<u64>,
    session: Extension<Session>,
    Json(project_update): Json<ProjectUpdate>,
) -> impl IntoResponse {
    let result = app_state
        .project_service
        .update(session.user_id, project_id, project_update)
        .await;

    match result {
        Ok(_) => Ok(StatusCode::OK),
        Err(e) => match e {
            ServiceError::ValidationError(e) => Err(api::Error::ValidationError(e)),
            e => Err(api::Error::Internal(e.into())),
        },
    }
}

#[cfg(test)]
mod tests {}
