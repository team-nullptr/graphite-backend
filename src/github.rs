use crate::{config, ApiError};
use anyhow::Context;
use hyper::{client::HttpConnector, header, Body, Client, Method, Request, Uri};
use hyper_tls::HttpsConnector;
use serde::{Deserialize, Serialize};
use std::str;

const GITHUB_API: &str = "https://api.github.com";

#[derive(Debug, Deserialize, Serialize)]
/// internal GitHub user representation with required fields.
pub struct GitHubUser {
    pub id: i32,
    pub avatar_url: String,
    pub name: String,
}

#[derive(Debug, Deserialize)]
/// GitHub access token response.
pub struct AccessTokenResp {
    pub access_token: String,
}

#[derive(Clone)]
pub struct GitHubService {
    https_client: Client<HttpsConnector<HttpConnector>>,
}

impl GitHubService {
    pub fn new(https_client: Client<HttpsConnector<HttpConnector>>) -> Self {
        Self { https_client }
    }

    /// Gets github user given user's access token.
    pub async fn get_github_user(&self, access_token: &str) -> Result<GitHubUser, anyhow::Error> {
        let uri = format!("{GITHUB_API}/user")
            .parse::<Uri>()
            .context("failed to parse github user api uri")?;

        let req = Request::builder()
            .method(Method::GET)
            .uri(uri)
            .header(header::USER_AGENT, "Graphite")
            .header(header::ACCEPT, "application/json")
            .header("Authorization", format!("Bearer {}", access_token))
            .body(Body::empty())?;

        let mut resp = self.https_client.request(req).await?;
        let body = hyper::body::to_bytes(resp.body_mut()).await?;

        Ok(serde_json::from_slice::<GitHubUser>(&body)?)
    }

    /// Gets access token for given auth code.
    pub async fn get_github_access_token(
        &self,
        code: &str,
        config: &config::Config,
    ) -> Result<AccessTokenResp, anyhow::Error> {
        let uri = format!(
            "https://github.com/login/oauth/access_token?client_id={}&client_secret={}&code={}",
            config.oauth.github_client_id, config.oauth.github_secret_id, code
        )
        .parse::<Uri>()
        .map_err(|_| {
            ApiError::Internal(anyhow::anyhow!("failed to parse github access_token uri"))
        })?;

        let req = Request::builder()
            .method(Method::GET)
            .uri(uri)
            .header(header::ACCEPT, "application/json")
            .body(Body::empty())?;

        let mut resp = self.https_client.request(req).await?;
        let body = hyper::body::to_bytes(resp.body_mut()).await?;

        Ok(serde_json::from_slice::<AccessTokenResp>(&body)?)
    }
}
