use std::collections::HashMap;

use anyhow::anyhow;
use axum::{
    extract::{Query, State},
    response::{AppendHeaders, IntoResponse},
};
use hyper::{
    header::{LOCATION, SET_COOKIE},
    StatusCode,
};

use crate::{api, AppState};

pub async fn github_oauth_redirect(
    State(app_state): State<AppState>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<impl IntoResponse, api::Error> {
    let code = params
        .get("code")
        .ok_or(api::Error::Internal(anyhow::anyhow!(
            "Expected code query param"
        )))?;

    let access_token_resp = app_state
        .github_service
        .get_github_access_token(
            code,
            &app_state.config.oauth.github_client_id,
            &app_state.config.oauth.github_secret_id,
        )
        .await
        .map_err(|e| api::Error::Internal(e))?;

    println!("{:?}", access_token_resp);

    let github_user = app_state
        .github_service
        .get_github_user(&access_token_resp.access_token)
        .await
        .map_err(api::Error::Internal)?;

    println!("{:?}", github_user);

    let session_id = app_state
        .session_manager
        .create_session(github_user)
        .await
        .map_err(|_| api::Error::Internal(anyhow!("Failed to create session")))?;

    println!("{}", session_id);

    // TODO: Make cookie secure on production
    let session_cookie = format!(
        "sid={}; Path=/; Max-Age={}; SameSite=None; Secure; HttpOnly",
        session_id,
        60 * 60 * 24 * 7,
    );

    Ok((
        StatusCode::PERMANENT_REDIRECT,
        AppendHeaders([
            (SET_COOKIE, session_cookie),
            (LOCATION, "http://localhost:5173/".to_string()),
        ]),
    ))
}
