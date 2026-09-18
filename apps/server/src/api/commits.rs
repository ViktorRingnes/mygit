use axum::Json;
use axum::extract::{Query, State};
use serde::Deserialize;
use utoipa::IntoParams;

use crate::api::clean;
use crate::error::{AppError, ErrorResponse};
use crate::git::commit::{self, Commits};
use crate::state::AppState;

const DEFAULT_LIMIT: u32 = 50;
const MAX_LIMIT: u32 = 1_000;

#[derive(Debug, Deserialize, IntoParams)]
#[into_params(parameter_in = Query)]
pub struct Params {
    rev: Option<String>,
    path: Option<String>,
    offset: Option<u64>,
    limit: Option<u32>,
}

#[utoipa::path(
    get,
    path = "/commits",
    tag = "commits",
    params(
        ("rev" = Option<String>, Query, description = "Branch, tag or commit to walk back from; defaults to the repository head"),
        ("path" = Option<String>, Query, description = "Only commits that changed this file or directory"),
        ("offset" = Option<u64>, Query, description = "Commits to skip"),
        ("limit" = Option<u32>, Query, description = "Commits to return, 50 by default and 1000 at most"),
    ),
    responses(
        (status = 200, body = Commits),
        (status = 400, body = ErrorResponse),
        (status = 404, body = ErrorResponse),
        (status = 500, body = ErrorResponse)
    )
)]
pub async fn list_commits(
    State(state): State<AppState>,
    Query(params): Query<Params>,
) -> Result<Json<Commits>, AppError> {
    let path = clean(params.path.as_deref())?;

    let request = commit::Request {
        rev: params.rev,
        path: (!path.is_empty()).then_some(path),
        offset: params.offset.unwrap_or(0),
        limit: params.limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT),
    };

    Ok(Json(state.git.commits(request).await?))
}
