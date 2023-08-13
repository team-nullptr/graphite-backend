use std::{future::Future, pin::Pin};

use axum::response::{IntoResponse, Response};
use axum_extra::extract::cookie::Cookie;
use hyper::{header::COOKIE, Body, Request, StatusCode};
use sqlx::{query, query_as, FromRow, MySqlPool};
use tower::{Layer, Service};
use uuid::Uuid;

use crate::github::GitHubUser;

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

impl<S> Layer<S> for SessionManager {
    type Service = SessionService<S>;

    fn layer(&self, inner: S) -> Self::Service {
        SessionService {
            inner,
            session_manager: self.clone(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct SessionService<S> {
    inner: S,
    session_manager: SessionManager,
}

// TODO: Rewrite this to
impl<S> Service<Request<Body>> for SessionService<S>
where
    S: Service<Request<Body>, Response = Response> + Send + Clone + 'static,
    S::Future: Send + 'static,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future =
        Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send + 'static>>;

    fn poll_ready(
        &mut self,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, mut req: Request<Body>) -> Self::Future {
        println!("{:?}", req.headers());

        // We need to extract session cookie
        let session_id = req
            .headers()
            .get_all(COOKIE)
            .iter()
            .filter_map(|cookie_header| cookie_header.to_str().ok())
            // Some cookie headers might contain multiple cookies
            .flat_map(|cookie_header| cookie_header.split(";"))
            .filter_map(|cookie_header| Cookie::parse_encoded(cookie_header.trim()).ok())
            .filter(|cookie| cookie.name() == "sid")
            .find_map(|cookie| Some(cookie.value().to_string()));

        let session_manager = self.session_manager.clone();
        let mut inner = self.inner.clone();

        Box::pin(async move {
            match session_id {
                Some(session_id) => match session_manager.get_session(session_id).await {
                    Ok(session) => {
                        if let Some(session) = session {
                            println!("extracted session: {:?}", session);

                            req.extensions_mut().insert(session);
                            println!("a");
                            let response = inner.call(req);
                            return Ok(response.await?);
                        }

                        Ok(StatusCode::UNAUTHORIZED.into_response())
                    }
                    Err(e) => {
                        println!("{:?}", e);
                        Ok(StatusCode::INTERNAL_SERVER_ERROR.into_response())
                    }
                },
                None => Ok(StatusCode::BAD_REQUEST.into_response()),
            }
        })
    }
}
