use axum::Json;
use axum::extract::{Query, State};
use serde::Deserialize;
use utoipa::IntoParams;

use crate::api::clean;
use crate::error::{AppError, ErrorResponse};
use crate::git::file::{self, File};
use crate::state::AppState;

#[derive(Debug, Deserialize, IntoParams)]
#[into_params(parameter_in = Query)]
pub struct Params {
    rev: Option<String>,
    path: Option<String>,
}

#[utoipa::path(
    get,
    path = "/file",
    tag = "files",
    params(
        ("rev" = Option<String>, Query, description = "Branch, tag or commit; defaults to the repository head"),
        ("path" = String, Query, description = "File to read"),
    ),
    responses(
        (status = 200, body = File),
        (status = 400, body = ErrorResponse),
        (status = 404, body = ErrorResponse),
        (status = 500, body = ErrorResponse)
    )
)]
pub async fn get_file(
    State(state): State<AppState>,
    Query(params): Query<Params>,
) -> Result<Json<File>, AppError> {
    let path = clean(params.path.as_deref())?;

    if path.is_empty() {
        return Err(AppError::bad_request("path is required"));
    }

    let request = file::Request {
        rev: params.rev,
        path,
    };

    Ok(Json(state.git.file(request).await?))
}
