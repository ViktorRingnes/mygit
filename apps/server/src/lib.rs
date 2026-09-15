pub mod api;
pub mod error;
pub mod git;
pub mod settings;
pub mod state;

use axum::Router;
use axum::http::HeaderValue;
use axum::http::header::InvalidHeaderValue;
use tower_http::cors::{Any, CorsLayer};

use settings::Settings;
use state::AppState;

pub fn app(settings: &Settings, state: AppState) -> Result<Router, InvalidHeaderValue> {
    Ok(api::router()
        .with_state(state)
        .layer(cors(&settings.cors.origins)?))
}

fn cors(origins: &[String]) -> Result<CorsLayer, InvalidHeaderValue> {
    let origins: Vec<HeaderValue> = origins
        .iter()
        .map(|origin| origin.parse())
        .collect::<Result<_, _>>()?;

    Ok(CorsLayer::new()
        .allow_origin(origins)
        .allow_methods(Any)
        .allow_headers(Any))
}
