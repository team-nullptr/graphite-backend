use axum::{
    async_trait,
    extract::{FromRequestParts, State},
    http::request::Parts,
    middleware::Next,
    response::{IntoResponse, Response},
};
use axum_extra::extract::cookie::Cookie;
use hyper::{header::COOKIE, Request, StatusCode};
use sqlx::{query, query_as, FromRow, MySqlPool};
use uuid::Uuid;

use crate::{github::GitHubUser, AppState};

#[derive(Debug, Clone, FromRow)]
pub struct Session {
    pub id: i32,
    #[sqlx(skip)]
    pub session_id: String,
    pub name: String,
    pub avatar_url: String,
}

#[derive(thiserror::Error, Debug)]
#[error("failed to create session")]
pub struct CreateSessionError(#[source] sqlx::Error);

#[derive(thiserror::Error, Debug)]
#[error("failed to get session")]
pub struct GetSessionError(#[source] sqlx::Error);

#[derive(Debug, Clone)]
pub struct SessionManager {
    db: MySqlPool,
}

impl SessionManager {
    pub fn new(db: MySqlPool) -> Self {
        Self { db }
    }

    pub async fn create_session(
        &self,
        github_user: GitHubUser,
    ) -> Result<String, CreateSessionError> {
        let sql = "INSERT INTO sessions (session_id, name, avatar_url) VALUES (?, ?, ?)";
        let session_id = Uuid::new_v4().to_string();

        query(sql)
            .bind(&session_id)
            .bind(github_user.name)
            .bind(github_user.avatar_url)
            .execute(&self.db)
            .await
            .map_err(|e| {
                println!("{:?}", e);
                CreateSessionError(e)
            })?;

        Ok(session_id)
    }

    pub async fn get_session(
        &self,
        session_id: String,
    ) -> Result<Option<Session>, GetSessionError> {
        let sql = "SELECT * FROM sessions WHERE session_id = ?";

        Ok(query_as::<_, Session>(sql)
            .bind(&session_id)
            .fetch_optional(&self.db)
            .await
            .map_err(GetSessionError)?)
    }
}

/// Extracts session id from session cookie.
/// If there will be any errors with extracting the session cookie reponds with 400 Bad Request.
///
/// If you need to access current session user use session::auth_middleware instead.
pub struct ExtractSessionId(String);

#[async_trait]
impl<S> FromRequestParts<S> for ExtractSessionId
where
    S: Send + Sync,
{
    type Rejection = StatusCode;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        // TODO: What status codes should be returned when?

        // This extractor assumes that HTTP/1.1 is used where ther is a single `Cookie` header with cookies
        // separated by semicolons.
        let cookie_header = parts
            .headers
            .get(COOKIE)
            .ok_or(StatusCode::BAD_REQUEST)?
            .to_str()
            .map_err(|_| StatusCode::BAD_REQUEST)?;

        let session_id = cookie_header
            .split(";")
            .into_iter()
            .filter_map(|cookie_header| Cookie::parse_encoded(cookie_header.trim()).ok())
            .filter(|cookie| cookie.name() == "sid")
            .find_map(|cookie| Some(cookie.value().to_string()));

        if let Some(session_id) = session_id {
            Ok(ExtractSessionId(session_id))
        } else {
            Err(StatusCode::UNAUTHORIZED)
        }
    }
}

/// Auth middleware extracts the user using sent session cookie.
/// If session does not valid or expired responds with 401 Unauthorized.
pub async fn auth_middleware<B>(
    state: State<AppState>,
    ExtractSessionId(session_id): ExtractSessionId,
    mut request: Request<B>,
    next: Next<B>,
) -> Response {
    // TODO: Add checks for expired sessions etc.
    if let Ok(Some(session)) = state.session_manager.get_session(session_id).await {
        request.extensions_mut().insert(session);
        next.run(request).await.into_response()
    } else {
        StatusCode::UNAUTHORIZED.into_response()
    }
}
