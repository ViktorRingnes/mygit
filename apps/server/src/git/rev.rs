use git2::{Commit, ErrorCode, Object, Repository};

use super::error::GitError;
use super::{branch, remotes};

pub struct Revision<'repo> {
    pub name: String,
    pub commit: Commit<'repo>,
}

pub fn resolve<'repo>(
    repo: &'repo Repository,
    rev: Option<&str>,
) -> Result<Revision<'repo>, GitError> {
    let name = match rev {
        Some(rev) if !rev.is_empty() => rev.to_owned(),
        _ => branch::default_branch(repo)?
            .ok_or_else(|| GitError::NotFound("repository has no branches".to_owned()))?,
    };

    let commit = find(repo, &name)?
        .peel_to_commit()
        .map_err(|_| GitError::Invalid(format!("revision '{name}' does not point at a commit")))?;

    Ok(Revision { name, commit })
}

fn find<'repo>(repo: &'repo Repository, name: &str) -> Result<Object<'repo>, GitError> {
    match repo.revparse_single(name) {
        Ok(object) => return Ok(object),
        Err(error) if error.code() == ErrorCode::NotFound => {}
        Err(error) => return Err(error.into()),
    }

    for remote in remotes(repo)? {
        if let Ok(object) = repo.revparse_single(&format!("{remote}/{name}")) {
            return Ok(object);
        }
    }

    Err(GitError::NotFound(format!("revision '{name}' not found")))
}

#[cfg(test)]
mod tests {
    use super::super::error::GitError;
    use super::super::fixture::Fixture;
    use super::resolve;

    #[test]
    fn defaults_to_the_repository_head() {
        let fixture = Fixture::new("trunk");
        let oid = fixture.commit("init", &[]);

        let revision = resolve(&fixture.repo, None).unwrap();

        assert_eq!(revision.name, "trunk");
        assert_eq!(revision.commit.id(), oid);
    }

    #[test]
    fn resolves_a_branch_a_tag_and_a_sha() {
        let fixture = Fixture::new("main");
        let oid = fixture.commit("init", &[]);
        fixture.branch("feature", oid);

        let commit = fixture.repo.find_commit(oid).unwrap();
        fixture
            .repo
            .tag_lightweight("v1", commit.as_object(), false)
            .unwrap();

        for name in ["feature", "v1", &oid.to_string()] {
            assert_eq!(resolve(&fixture.repo, Some(name)).unwrap().commit.id(), oid);
        }
    }

    #[test]
    fn resolves_a_remote_only_branch_by_its_short_name() {
        let fixture = Fixture::new("main");
        let oid = fixture.commit("init", &[]);

        fixture
            .repo
            .remote("origin", "https://example.com/repo.git")
            .unwrap();
        fixture
            .repo
            .reference("refs/remotes/origin/release", oid, true, "")
            .unwrap();

        assert_eq!(
            resolve(&fixture.repo, Some("release")).unwrap().commit.id(),
            oid
        );
    }

    #[test]
    fn rejects_an_unknown_revision() {
        let fixture = Fixture::new("main");
        fixture.commit("init", &[]);

        let error = resolve(&fixture.repo, Some("ghost")).err();

        assert!(matches!(error, Some(GitError::NotFound(_))));
    }

    #[test]
    fn rejects_a_repository_without_commits() {
        let fixture = Fixture::new("main");

        let error = resolve(&fixture.repo, None).err();

        assert!(matches!(error, Some(GitError::NotFound(_))));
    }
}
