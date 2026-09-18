use axum::Router;
use utoipa::OpenApi;
use utoipa::openapi::OpenApi as Document;
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;

use crate::error::AppError;
use crate::state::AppState;

pub mod branches;
pub mod commits;
pub mod file;
pub mod tree;

#[derive(OpenApi)]
#[openapi(
    info(title = "mygit", version = env!("CARGO_PKG_VERSION")),
    servers((url = "/"))
)]
struct ApiDoc;

pub fn router() -> Router<AppState> {
    build().split_for_parts().0
}

pub fn document() -> Document {
    build().into_openapi()
}

fn build() -> OpenApiRouter<AppState> {
    OpenApiRouter::with_openapi(ApiDoc::openapi())
        .routes(routes!(branches::list_branches))
        .routes(routes!(tree::get_tree))
        .routes(routes!(file::get_file))
        .routes(routes!(commits::list_commits))
}

fn clean(path: Option<&str>) -> Result<String, AppError> {
    let path = path.unwrap_or_default().trim_matches('/');

    if path
        .split('/')
        .any(|segment| segment == "." || segment == "..")
    {
        return Err(AppError::bad_request(format!("invalid path '{path}'")));
    }

    Ok(path.to_owned())
}
