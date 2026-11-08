use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Authentication error: {0}")]
    Auth(String),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Conflict: {0}")]
    Conflict(String),

    #[error("Bad request: {0}")]
    BadRequest(String),

    #[error("Internal server error: {0}")]
    Internal(String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            AppError::NotFound(msg) => (StatusCode::NOT_FOUND, msg),
            AppError::Conflict(msg) => (StatusCode::CONFLICT, msg),
            AppError::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg),
            AppError::Validation(msg) => (StatusCode::BAD_REQUEST, msg),
            AppError::Auth(msg) => (StatusCode::UNAUTHORIZED, msg),
            AppError::Database(err) => {
                let err_str = err.to_string();
                if err_str.contains("unique") || err_str.contains("duplicate") {
                    (
                        StatusCode::CONFLICT,
                        "Resource with identifier already exists".to_string(),
                    )
                } else if err_str.contains("foreign key") {
                    (
                        StatusCode::BAD_REQUEST,
                        "Referenced entity does not exist".to_string(),
                    )
                } else {
                    eprintln!("DATABASE ERROR: {:?}", err);
                    tracing::error!("Database error: {:?}", err);
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        format!("Database error: {}", err),
                    )
                }
            }
            AppError::Internal(msg) => {
                tracing::error!("Internal error: {}", msg);
                (StatusCode::INTERNAL_SERVER_ERROR, msg)
            }
        };

        let body = Json(json!({
            "error": message,
            "status": status.as_u16()
        }));

        (status, body).into_response()
    }
}
