use axum::Router;
use utoipa::OpenApi;
use utoipa::openapi::OpenApi as Document;
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;

use crate::state::AppState;

pub mod branches;

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
    OpenApiRouter::with_openapi(ApiDoc::openapi()).routes(routes!(branches::list_branches))
}
