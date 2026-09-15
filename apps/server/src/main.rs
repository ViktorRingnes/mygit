use axum::{Router, routing::get};
use std::path::PathBuf;

mod data;
mod handlers;

#[tokio::main]
async fn main() {
    let repo_path = PathBuf::from("../../")
        .canonicalize()
        .expect("failed to resolve repository path");

    data::init(repo_path.to_string_lossy().into_owned())
        .expect("failed to initialize repository path");

    let app = Router::new()
        .route("/", get(|| async { "Hello, World!" }))
        .route("/branches", get(handlers::branch::get_branches_handler));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:5000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
