use std::str;

use anyhow::Context;
use hyper::{client::HttpConnector, header, Body, Client, Method, Request, Uri};
use hyper_tls::HttpsConnector;
use serde::{Deserialize, Serialize};

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

#[derive(thiserror::Error, Debug)]
#[error("failed to get github user")]
pub struct GetGithubUserError(#[source] anyhow::Error);

#[derive(thiserror::Error, Debug)]
#[error("failed to get github user access token")]
pub struct GetGithubAccessTokenError(#[source] anyhow::Error);

#[derive(Clone)]
pub struct GitHubService {
    https_client: Client<HttpsConnector<HttpConnector>>,
}

impl GitHubService {
    pub fn new(https_client: Client<HttpsConnector<HttpConnector>>) -> Self {
        Self { https_client }
    }

    /// Gets github user given user's access token.
    pub async fn get_github_user(
        &self,
        access_token: &str,
    ) -> Result<GitHubUser, GetGithubUserError> {
        let uri = format!("{GITHUB_API}/user")
            .parse::<Uri>()
            .context("failed to parse github user uri")
            .map_err(GetGithubUserError)?;

        let req = Request::builder()
            .method(Method::GET)
            .uri(uri)
            .header(header::USER_AGENT, "Graphite")
            .header(header::ACCEPT, "application/json")
            .header("Authorization", format!("Bearer {}", access_token))
            .body(Body::empty())
            .context("failed to prepare the request")
            .map_err(GetGithubUserError)?;

        let mut response = self
            .https_client
            .request(req)
            .await
            .context("failed to make a request")
            .map_err(GetGithubUserError)?;

        let body = hyper::body::to_bytes(response.body_mut())
            .await
            .context("failed to extract response body")
            .map_err(GetGithubUserError)?;

        Ok(serde_json::from_slice::<GitHubUser>(&body)
            .context("failed to deserialize user data response")
            .map_err(GetGithubUserError)?)
    }

    /// Gets access token for given auth code.
    pub async fn get_github_access_token(
        &self,
        code: &str,
        github_client_id: &str,
        github_secret_id: &str,
    ) -> Result<AccessTokenResp, GetGithubAccessTokenError> {
        let uri = format!(
            "https://github.com/login/oauth/access_token?client_id={}&client_secret={}&code={}",
            github_client_id, github_secret_id, code
        )
        .parse::<Uri>()
        .context("failed to parse github access_token uri")
        .map_err(GetGithubAccessTokenError)?;

        let req = Request::builder()
            .method(Method::GET)
            .uri(uri)
            .header(header::ACCEPT, "application/json")
            .body(Body::empty())
            .context("failed to prepare the request")
            .map_err(GetGithubAccessTokenError)?;

        let mut resp = self
            .https_client
            .request(req)
            .await
            .context("failed to make a request")
            .map_err(GetGithubAccessTokenError)?;

        let body = hyper::body::to_bytes(resp.body_mut())
            .await
            .context("failed to extract response body")
            .map_err(GetGithubAccessTokenError)?;

        Ok(serde_json::from_slice::<AccessTokenResp>(&body)
            .context("failed to deserialize acces token response")
            .map_err(GetGithubAccessTokenError)?)
    }
}
