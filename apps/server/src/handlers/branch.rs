use axum::{Json, http::StatusCode, response::IntoResponse};

pub fn get_branches() -> Result<Vec<String>, git2::Error> {
    let repo = git2::Repository::open(&crate::data::get().repo_path)?;

    repo.branches(None)?
        .map(|branch_result| {
            let (branch, _) = branch_result?;

            let name = branch
                .name()?
                .ok_or_else(|| git2::Error::from_str("invalid UTF-8 branch name"))?;

            Ok(name.to_owned())
        })
        .collect()
}

pub async fn get_branches_handler() -> impl IntoResponse {
    match get_branches() {
        Ok(branches) => (StatusCode::OK, Json(branches)).into_response(),

        Err(error) => {
            eprintln!("failed to get branches: {error}");

            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to read Git branches",
            )
                .into_response()
        }
    }
}
