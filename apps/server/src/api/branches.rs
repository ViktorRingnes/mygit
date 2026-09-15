use axum::Json;
use axum::extract::State;

use crate::error::{AppError, ErrorResponse};
use crate::state::AppState;

#[utoipa::path(
    get,
    path = "/branches",
    tag = "branches",
    responses(
        (status = 200, body = Vec<String>),
        (status = 500, body = ErrorResponse)
    )
)]
pub async fn list_branches(State(state): State<AppState>) -> Result<Json<Vec<String>>, AppError> {
    Ok(Json(state.git.branches().await?))
}
