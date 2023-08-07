use crate::{ApiError, AppState};
use anyhow::Context;
use axum::extract::{Query, State};
use hyper::StatusCode;
use std::collections::HashMap;

pub async fn github_oauth_redirect(
    State(app_state): State<AppState>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<StatusCode, ApiError> {
    let code = params
        .get("code")
        .ok_or(ApiError::Internal(anyhow::anyhow!(
            "Expected code query param"
        )))?;

    let access_token_resp = app_state
        .github_service
        .get_github_access_token(code, &app_state.config)
        .await
        .map_err(|e| ApiError::Internal(e))?;

    let github_user = app_state
        .github_service
        .get_github_user(&access_token_resp.access_token)
        .await
        .map_err(|e| ApiError::Internal(e))?;

    // TODO: Create a new user in our database if he does not exist yet.

    app_state
        .user_service
        .create_user_if_new(github_user)
        .await
        .context("failed to create user if he does not exist")
        .map_err(|e| ApiError::Internal(e))?;

    // TODO: Create a session and return it back to the user.

    Ok(StatusCode::OK)
}
