use axum::Json;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde::Serialize;
use utoipa::ToSchema;

#[derive(Debug, Serialize, ToSchema)]
pub struct ErrorResponse {
    pub message: String,
}

#[derive(Debug)]
pub struct AppError {
    status: StatusCode,
    message: String,
}

impl From<git2::Error> for AppError {
    fn from(error: git2::Error) -> Self {
        Self {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            message: error.message().to_owned(),
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
