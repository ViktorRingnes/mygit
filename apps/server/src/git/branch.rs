use git2::{BranchType, ErrorCode, Repository};
use serde::Serialize;
use utoipa::ToSchema;

use super::error::GitError;
use super::{GitService, remotes};

const CONVENTIONAL: [&str; 4] = ["main", "master", "trunk", "develop"];

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct Branches {
    pub names: Vec<String>,
    #[schema(required)]
    pub default_branch: Option<String>,
    #[schema(required)]
    pub head: Option<String>,
}

impl GitService {
    pub async fn branches(&self) -> Result<Branches, GitError> {
        self.read(branches).await
    }
}

pub(super) fn default_branch(repo: &Repository) -> Result<Option<String>, GitError> {
    match identified(repo)? {
        Some(name) => Ok(Some(name)),
        None => Ok(names(repo)?.first().cloned()),
    }
}

fn branches(repo: &Repository) -> Result<Branches, GitError> {
    let names = names(repo)?;

    let default_branch = match identified(repo)? {
        Some(name) => Some(name),
        None => names.first().cloned(),
    };

    Ok(Branches {
        names,
        default_branch,
        head: checked_out(repo)?,
    })
}

fn identified(repo: &Repository) -> Result<Option<String>, GitError> {
    let remotes = remotes(repo)?;
    let checked_out = checked_out(repo)?;
    let remote = remote_head(repo, &remotes)?;

    let conventional = CONVENTIONAL
        .iter()
        .find(|name| exists(repo, name, &remotes))
        .map(|name| (*name).to_owned());

    let ordered = if repo.is_bare() {
        [checked_out.clone(), remote, conventional]
    } else {
        [remote, conventional, checked_out.clone()]
    };

    Ok(ordered
        .into_iter()
        .flatten()
        .find(|name| exists(repo, name, &remotes))
        .or(checked_out))
}

fn exists(repo: &Repository, name: &str, remotes: &[String]) -> bool {
    repo.find_branch(name, BranchType::Local).is_ok()
        || remotes.iter().any(|remote| {
            repo.find_branch(&format!("{remote}/{name}"), BranchType::Remote)
                .is_ok()
        })
}

fn names(repo: &Repository) -> Result<Vec<String>, GitError> {
    let mut names = Vec::new();

    for result in repo.branches(None)? {
        let (branch, kind) = result?;

        if branch.get().symbolic_target()?.is_some() {
            continue;
        }

        let name = branch
            .name()?
            .ok_or_else(|| GitError::Invalid("branch name is not valid utf8".to_owned()))?;

        names.push(short(name, kind).to_owned());
    }

    names.sort_unstable();
    names.dedup();

    Ok(names)
}

fn checked_out(repo: &Repository) -> Result<Option<String>, GitError> {
    Ok(symbolic(repo, "HEAD")?
        .and_then(|target| target.strip_prefix("refs/heads/").map(ToOwned::to_owned)))
}

fn remote_head(repo: &Repository, remotes: &[String]) -> Result<Option<String>, GitError> {
    for remote in remotes {
        let prefix = format!("refs/remotes/{remote}/");

        if let Some(target) = symbolic(repo, &format!("{prefix}HEAD"))?
            && let Some(name) = target.strip_prefix(&prefix)
        {
            return Ok(Some(name.to_owned()));
        }
    }

    Ok(None)
}

fn symbolic(repo: &Repository, name: &str) -> Result<Option<String>, GitError> {
    match repo.find_reference(name) {
        Ok(reference) => Ok(reference.symbolic_target()?.map(ToOwned::to_owned)),
        Err(error) if error.code() == ErrorCode::NotFound => Ok(None),
        Err(error) => Err(error.into()),
    }
}

fn short(name: &str, kind: BranchType) -> &str {
    match kind {
        BranchType::Remote => name.split_once('/').map_or(name, |(_, short)| short),
        BranchType::Local => name,
    }
}

#[cfg(test)]
mod tests {
    use super::super::fixture::Fixture;
    use super::{Branches, branches};

    fn default_of(fixture: &Fixture) -> Option<String> {
        branches(&fixture.repo).unwrap().default_branch
    }

    #[test]
    fn ignores_the_checked_out_branch_in_favour_of_the_remote_default() {
        let fixture = Fixture::new("main");
        let oid = fixture.commit("init", &[]);
        fixture.remote_head("origin", "main", oid);
        fixture.branch("user/test", oid);
        fixture.repo.set_head("refs/heads/user/test").unwrap();

        let Branches {
            names,
            default_branch,
            head,
        } = branches(&fixture.repo).unwrap();

        assert_eq!(names, ["main", "user/test"]);
        assert_eq!(default_branch.as_deref(), Some("main"));
        assert_eq!(head.as_deref(), Some("user/test"));
    }

    #[test]
    fn reports_the_checked_out_branch_separately_from_the_default() {
        let fixture = Fixture::new("main");
        let oid = fixture.commit("init", &[]);
        fixture.remote_head("origin", "main", oid);
        fixture.branch("user/test", oid);
        fixture.repo.set_head("refs/heads/user/test").unwrap();

        let result = branches(&fixture.repo).unwrap();

        assert_eq!(result.default_branch.as_deref(), Some("main"));
        assert_eq!(result.head.as_deref(), Some("user/test"));
    }

    #[test]
    fn reports_no_head_when_detached() {
        let fixture = Fixture::new("main");
        let oid = fixture.commit("init", &[]);
        fixture.repo.set_head_detached(oid).unwrap();

        assert_eq!(branches(&fixture.repo).unwrap().head, None);
    }

    #[test]
    fn prefers_the_remote_default_over_a_conventional_name() {
        let fixture = Fixture::new("main");
        let oid = fixture.commit("init", &[]);
        fixture.branch("release", oid);
        fixture.remote_head("origin", "release", oid);

        assert_eq!(default_of(&fixture).as_deref(), Some("release"));
    }

    #[test]
    fn falls_back_to_a_conventional_name_without_a_remote() {
        let fixture = Fixture::new("main");
        let oid = fixture.commit("init", &[]);
        fixture.branch("user/test", oid);
        fixture.repo.set_head("refs/heads/user/test").unwrap();

        assert_eq!(default_of(&fixture).as_deref(), Some("main"));
    }

    #[test]
    fn uses_the_checked_out_branch_when_nothing_else_identifies_one() {
        let fixture = Fixture::new("scratch");
        let oid = fixture.commit("init", &[]);
        fixture.branch("zeta", oid);

        assert_eq!(default_of(&fixture).as_deref(), Some("scratch"));
    }

    #[test]
    fn trusts_head_in_a_bare_repository() {
        let fixture = Fixture::bare("release");
        let oid = fixture.commit("init", &[]);
        fixture.branch("main", oid);

        assert_eq!(default_of(&fixture).as_deref(), Some("release"));
    }

    #[test]
    fn falls_back_to_the_remote_head_when_detached() {
        let fixture = Fixture::new("trunk");
        let oid = fixture.commit("init", &[]);
        fixture.remote_head("origin", "trunk", oid);
        fixture.repo.set_head_detached(oid).unwrap();

        let Branches {
            names,
            default_branch,
            ..
        } = branches(&fixture.repo).unwrap();

        assert_eq!(names, ["trunk"]);
        assert_eq!(default_branch.as_deref(), Some("trunk"));
    }

    #[test]
    fn reports_an_unborn_head_for_an_empty_repository() {
        let fixture = Fixture::new("main");

        let Branches {
            names,
            default_branch,
            ..
        } = branches(&fixture.repo).unwrap();

        assert!(names.is_empty());
        assert_eq!(default_branch.as_deref(), Some("main"));
    }
}
