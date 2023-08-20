use axum::{response::IntoResponse, Json};
use hyper::StatusCode;
use serde_json::json;
use validator::ValidationErrors;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("internal error")]
    Internal(#[source] anyhow::Error),

    #[error("bad request")]
    BadRequest {
        #[source]
        source: Option<anyhow::Error>,
        message: String,
    },

    #[error("validation error")]
    ValidationError(#[source] ValidationErrors),

    #[error("resource not found")]
    NotFound,
}

impl IntoResponse for Error {
    fn into_response(self) -> axum::response::Response {
        match self {
            Self::Internal(source) => {
                // TODO: Add proper logging
                println!("Internal: {:?}", source);

                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({
                        "status": "error",
                        "message": "Internal server error."
                    })),
                )
                    .into_response()
            }
            Self::BadRequest { source, message } => {
                // TODO: Add proper logging
                println!("Bad Request: {:?}", source);

                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({
                        "status": "error",
                        "message": message
                    })),
                )
                    .into_response()
            }
            Self::NotFound => {
                println!("resource not found");

                (
                    StatusCode::NOT_FOUND,
                    Json(json!({
                        "status": "error",
                        "message": "Not Found"
                    })),
                )
                    .into_response()
            }
            Self::ValidationError(e) => {
                // TODO: Add proper logging
                println!("Validation Error: {:?}", e);

                let validation_errors =
                    serde_json::to_string(e.errors()).map_err(|e| Self::Internal(e.into()));

                match validation_errors {
                    Ok(validation_errors) => (
                        StatusCode::BAD_REQUEST,
                        Json(json!({
                            "status": "validation_error",
                            "fields": validation_errors
                        })),
                    )
                        .into_response(),
                    Err(e) => e.into_response(),
                }
            }
        }
    }
}
