use std::{collections::HashMap, sync::Arc};

use anyhow::Context;
use axum::{
    extract::{Query, State},
    response::{AppendHeaders, IntoResponse},
};
use hyper::{
    header::{LOCATION, SET_COOKIE},
    StatusCode,
};

use crate::{api, AppState};

const SESSION_COOKIE_LIFETIME: i32 = 60 * 60 * 24 * 7;

pub async fn github_oauth_redirect(
    State(app_state): State<Arc<AppState>>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<impl IntoResponse, api::Error> {
    let code = params.get("code").ok_or(api::Error::BadRequest {
        source: None,
        message: "expected code query param".to_string(),
    })?;

    let access_token_resp = app_state
        .github_service
        .get_github_access_token(
            code,
            &app_state.config.oauth.github_client_id,
            &app_state.config.oauth.github_secret_id,
        )
        .await
        .map_err(|e| api::Error::Internal(e.into()))?;

    println!("{:?}", access_token_resp);

    let github_user = app_state
        .github_service
        .get_github_user(&access_token_resp.access_token)
        .await
        .map_err(|e| api::Error::Internal(e.into()))?;

    println!("{:?}", github_user);

    let session_id = app_state
        .session_manager
        .create_session(github_user)
        .await
        .context("failed to create a new session")
        .map_err(|e| api::Error::Internal(e))?;

    println!("{}", session_id);

    let session_cookie = format!(
        "sid={}; Path=/; Max-Age={}; SameSite=None; Secure; HttpOnly",
        session_id, SESSION_COOKIE_LIFETIME
    );

    Ok((
        StatusCode::PERMANENT_REDIRECT,
        AppendHeaders([
            (SET_COOKIE, session_cookie),
            (LOCATION, "http://localhost:5173/".to_string()),
        ]),
    ))
}
