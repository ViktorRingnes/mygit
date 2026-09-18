use axum::Json;
use axum::extract::{Query, State};
use serde::Deserialize;
use utoipa::IntoParams;

use crate::api::clean;
use crate::error::{AppError, ErrorResponse};
use crate::git::tree::{self, Tree};
use crate::state::AppState;

const DEFAULT_LIMIT: u32 = 1_000;
const MAX_LIMIT: u32 = 10_000;

#[derive(Debug, Deserialize, IntoParams)]
#[into_params(parameter_in = Query)]
pub struct Params {
    rev: Option<String>,
    path: Option<String>,
    depth: Option<u32>,
    offset: Option<u64>,
    limit: Option<u32>,
}

#[utoipa::path(
    get,
    path = "/tree",
    tag = "files",
    params(
        ("rev" = Option<String>, Query, description = "Branch, tag or commit; defaults to the repository head"),
        ("path" = Option<String>, Query, description = "Directory to list; defaults to the repository root"),
        ("depth" = Option<u32>, Query, description = "Levels to descend, 1 by default; 0 lists every level"),
        ("offset" = Option<u64>, Query, description = "Entries to skip"),
        ("limit" = Option<u32>, Query, description = "Entries to return, 1000 by default and 10000 at most"),
    ),
    responses(
        (status = 200, body = Tree),
        (status = 400, body = ErrorResponse),
        (status = 404, body = ErrorResponse),
        (status = 500, body = ErrorResponse)
    )
)]
pub async fn get_tree(
    State(state): State<AppState>,
    Query(params): Query<Params>,
) -> Result<Json<Tree>, AppError> {
    let request = tree::Request {
        rev: params.rev,
        path: clean(params.path.as_deref())?,
        depth: params.depth.unwrap_or(1),
        offset: params.offset.unwrap_or(0),
        limit: params.limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT),
    };

    Ok(Json(state.git.tree(request).await?))
}
