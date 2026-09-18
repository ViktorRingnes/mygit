use std::path::Path;
use std::str;

use git2::Repository;
use serde::Serialize;
use utoipa::ToSchema;

use super::GitService;
use super::error::GitError;
use super::rev;

pub const MAX_BYTES: usize = 5 * 1024 * 1024;

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct File {
    pub rev: String,
    pub commit: String,
    pub path: String,
    pub oid: String,
    pub size: u64,
    pub binary: bool,
    pub oversized: bool,
    #[schema(required)]
    pub text: Option<String>,
}

pub struct Request {
    pub rev: Option<String>,
    pub path: String,
}

impl GitService {
    pub async fn file(&self, request: Request) -> Result<File, GitError> {
        self.read(move |repo| file(repo, &request)).await
    }
}

fn file(repo: &Repository, request: &Request) -> Result<File, GitError> {
    let revision = rev::resolve(repo, request.rev.as_deref())?;

    let entry = revision
        .commit
        .tree()?
        .get_path(Path::new(&request.path))
        .map_err(|_| GitError::NotFound(format!("path '{}' does not exist", request.path)))?;

    let blob = entry
        .to_object(repo)?
        .into_blob()
        .map_err(|_| GitError::Invalid(format!("path '{}' is not a file", request.path)))?;

    let content = blob.content();
    let oversized = content.len() > MAX_BYTES;

    let text = if oversized || blob.is_binary() {
        None
    } else {
        str::from_utf8(content).ok().map(ToOwned::to_owned)
    };

    Ok(File {
        rev: revision.name,
        commit: revision.commit.id().to_string(),
        path: request.path.clone(),
        oid: blob.id().to_string(),
        size: u64::try_from(content.len()).unwrap_or(u64::MAX),
        binary: !oversized && text.is_none(),
        oversized,
        text,
    })
}

#[cfg(test)]
mod tests {
    use super::super::error::GitError;
    use super::super::fixture::Fixture;
    use super::{Request, file};

    fn fixture() -> Fixture {
        let fixture = Fixture::new("main");

        fixture.commit(
            "init",
            &[
                ("src/lib.rs", b"fn lib() {}\n"),
                ("logo.png", &[0x89, 0x50, 0x4E, 0x47, 0x00, 0x01, 0x02]),
            ],
        );

        fixture
    }

    fn request(path: &str) -> Request {
        Request {
            rev: None,
            path: path.to_owned(),
        }
    }

    #[test]
    fn reads_text_content() {
        let fixture = fixture();

        let blob = file(&fixture.repo, &request("src/lib.rs")).unwrap();

        assert_eq!(blob.text.as_deref(), Some("fn lib() {}\n"));
        assert_eq!(blob.size, 12);
        assert_eq!(blob.rev, "main");
        assert!(!blob.binary);
        assert!(!blob.oversized);
    }

    #[test]
    fn withholds_binary_content() {
        let fixture = fixture();

        let blob = file(&fixture.repo, &request("logo.png")).unwrap();

        assert!(blob.binary);
        assert_eq!(blob.text, None);
        assert_eq!(blob.size, 7);
    }

    #[test]
    fn rejects_a_directory() {
        let fixture = fixture();

        let error = file(&fixture.repo, &request("src")).unwrap_err();

        assert!(matches!(error, GitError::Invalid(_)));
    }

    #[test]
    fn rejects_a_missing_file() {
        let fixture = fixture();

        let error = file(&fixture.repo, &request("nope.rs")).unwrap_err();

        assert!(matches!(error, GitError::NotFound(_)));
    }
}
