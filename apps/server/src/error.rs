use axum::Json;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde::Serialize;
use utoipa::ToSchema;

use crate::git::error::GitError;

#[derive(Debug, Serialize, ToSchema)]
pub struct ErrorResponse {
    pub message: String,
}

#[derive(Debug)]
pub struct AppError {
    status: StatusCode,
    message: String,
}

impl AppError {
    pub fn bad_request(message: impl Into<String>) -> Self {
        Self {
            status: StatusCode::BAD_REQUEST,
            message: message.into(),
        }
    }
}

impl From<GitError> for AppError {
    fn from(error: GitError) -> Self {
        let status = match error {
            GitError::NotFound(_) => StatusCode::NOT_FOUND,
            GitError::Invalid(_) => StatusCode::BAD_REQUEST,
            GitError::Internal(_) | GitError::Git(_) => StatusCode::INTERNAL_SERVER_ERROR,
        };

        Self {
            status,
            message: error.to_string(),
        }
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        eprintln!("{} {}", self.status, self.message);

        let body = ErrorResponse {
            message: self.message,
        };

        (self.status, Json(body)).into_response()
    }
}
