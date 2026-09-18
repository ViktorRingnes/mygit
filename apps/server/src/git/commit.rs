use std::path::Path;

use git2::{Commit as GitCommit, ErrorCode, Oid, Repository, Signature as GitSignature, Sort};
use serde::Serialize;
use utoipa::ToSchema;

use super::GitService;
use super::error::GitError;
use super::rev;

pub const MAX_SCAN: usize = 250_000;

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct Signature {
    pub name: String,
    pub email: String,
    pub timestamp: i64,
    pub offset_minutes: i32,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct Commit {
    pub oid: String,
    pub summary: String,
    #[schema(required)]
    pub body: Option<String>,
    pub author: Signature,
    pub committer: Signature,
    pub parents: Vec<String>,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct Commits {
    pub rev: String,
    pub commit: String,
    #[schema(required)]
    pub path: Option<String>,
    pub entries: Vec<Commit>,
    pub offset: u64,
    pub has_more: bool,
}

pub struct Request {
    pub rev: Option<String>,
    pub path: Option<String>,
    pub offset: u64,
    pub limit: u32,
}

impl GitService {
    pub async fn commits(&self, request: Request) -> Result<Commits, GitError> {
        self.read(move |repo| commits(repo, &request)).await
    }
}

fn commits(repo: &Repository, request: &Request) -> Result<Commits, GitError> {
    let revision = rev::resolve(repo, request.rev.as_deref())?;
    let path = request.path.as_deref().filter(|path| !path.is_empty());

    let mut walk = repo.revwalk()?;
    walk.set_sorting(Sort::TIME | Sort::TOPOLOGICAL)?;
    walk.push(revision.commit.id())?;

    let limit = request.limit as usize;
    let mut entries = Vec::new();
    let mut skipped = 0;
    let mut scanned = 0;
    let mut has_more = false;

    for oid in walk {
        scanned += 1;

        if scanned > MAX_SCAN {
            has_more = true;
            break;
        }

        let commit = repo.find_commit(oid?)?;

        if let Some(path) = path
            && !touches(&commit, path)?
        {
            continue;
        }

        if skipped < request.offset {
            skipped += 1;
            continue;
        }

        if entries.len() >= limit {
            has_more = true;
            break;
        }

        entries.push(summarize(&commit)?);
    }

    Ok(Commits {
        rev: revision.name,
        commit: revision.commit.id().to_string(),
        path: path.map(ToOwned::to_owned),
        entries,
        offset: request.offset,
        has_more,
    })
}

fn touches(commit: &GitCommit<'_>, path: &str) -> Result<bool, GitError> {
    let current = entry(commit, path)?;
    let mut parents = false;

    for parent in commit.parents() {
        parents = true;

        if entry(&parent, path)? == current {
            return Ok(false);
        }
    }

    Ok(if parents { true } else { current.is_some() })
}

fn entry(commit: &GitCommit<'_>, path: &str) -> Result<Option<Oid>, GitError> {
    match commit.tree()?.get_path(Path::new(path)) {
        Ok(entry) => Ok(Some(entry.id())),
        Err(error) if error.code() == ErrorCode::NotFound => Ok(None),
        Err(error) => Err(error.into()),
    }
}

fn summarize(commit: &GitCommit<'_>) -> Result<Commit, GitError> {
    Ok(Commit {
        oid: commit.id().to_string(),
        summary: commit.summary()?.unwrap_or_default().to_owned(),
        body: commit.body()?.map(ToOwned::to_owned),
        author: signature(&commit.author()),
        committer: signature(&commit.committer()),
        parents: commit.parent_ids().map(|oid| oid.to_string()).collect(),
    })
}

fn signature(signature: &GitSignature<'_>) -> Signature {
    Signature {
        name: signature.name().unwrap_or_default().to_owned(),
        email: signature.email().unwrap_or_default().to_owned(),
        timestamp: signature.when().seconds(),
        offset_minutes: signature.when().offset_minutes(),
    }
}

#[cfg(test)]
mod tests {
    use super::super::fixture::Fixture;
    use super::{Request, commits};

    fn fixture() -> Fixture {
        let fixture = Fixture::new("main");

        fixture.commit("add lib", &[("src/lib.rs", b"one")]);
        fixture.commit("add readme", &[("readme.md", b"docs")]);
        fixture.commit("touch lib", &[("src/lib.rs", b"two")]);

        fixture
    }

    fn request() -> Request {
        Request {
            rev: None,
            path: None,
            offset: 0,
            limit: 50,
        }
    }

    #[test]
    fn walks_history_newest_first() {
        let fixture = fixture();

        let log = commits(&fixture.repo, &request()).unwrap();

        let summaries: Vec<&str> = log
            .entries
            .iter()
            .map(|commit| commit.summary.as_str())
            .collect();

        assert_eq!(summaries, ["touch lib", "add readme", "add lib"]);
        assert_eq!(log.rev, "main");
        assert!(!log.has_more);
    }

    #[test]
    fn records_authorship_and_parents() {
        let fixture = fixture();

        let log = commits(&fixture.repo, &request()).unwrap();
        let newest = &log.entries[0];
        let oldest = &log.entries[2];

        assert_eq!(newest.author.name, "test");
        assert_eq!(newest.author.email, "test@example.com");
        assert_eq!(newest.parents, [log.entries[1].oid.clone()]);
        assert!(oldest.parents.is_empty());
    }

    #[test]
    fn filters_by_path() {
        let fixture = fixture();

        let log = commits(
            &fixture.repo,
            &Request {
                path: Some("src/lib.rs".to_owned()),
                ..request()
            },
        )
        .unwrap();

        let summaries: Vec<&str> = log
            .entries
            .iter()
            .map(|commit| commit.summary.as_str())
            .collect();

        assert_eq!(summaries, ["touch lib", "add lib"]);
        assert_eq!(log.path.as_deref(), Some("src/lib.rs"));
    }

    #[test]
    fn pages_through_history() {
        let fixture = fixture();

        let first = commits(
            &fixture.repo,
            &Request {
                limit: 2,
                ..request()
            },
        )
        .unwrap();

        assert_eq!(first.entries.len(), 2);
        assert!(first.has_more);

        let second = commits(
            &fixture.repo,
            &Request {
                offset: 2,
                limit: 2,
                ..request()
            },
        )
        .unwrap();

        assert_eq!(second.entries.len(), 1);
        assert_eq!(second.entries[0].summary, "add lib");
        assert!(!second.has_more);
    }

    #[test]
    fn pages_a_filtered_log() {
        let fixture = fixture();

        let page = commits(
            &fixture.repo,
            &Request {
                path: Some("src/lib.rs".to_owned()),
                offset: 1,
                limit: 1,
                ..request()
            },
        )
        .unwrap();

        assert_eq!(page.entries.len(), 1);
        assert_eq!(page.entries[0].summary, "add lib");
        assert!(!page.has_more);
    }
}
