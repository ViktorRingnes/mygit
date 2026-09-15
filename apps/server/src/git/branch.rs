use git2::{BranchType, Error, Repository};

use super::GitService;

impl GitService {
    pub async fn branches(&self) -> Result<Vec<String>, Error> {
        self.read(branches).await
    }
}

fn branches(repo: &Repository) -> Result<Vec<String>, Error> {
    let mut names = Vec::new();

    for result in repo.branches(None)? {
        let (branch, kind) = result?;

        if branch.get().symbolic_target()?.is_some() {
            continue;
        }

        let name = branch
            .name()?
            .ok_or_else(|| Error::from_str("branch name is not valid utf8"))?;

        names.push(
            match kind {
                BranchType::Remote => name.split_once('/').map_or(name, |(_, short)| short),
                BranchType::Local => name,
            }
            .to_owned(),
        );
    }

    names.sort_unstable();
    names.dedup();

    Ok(names)
}
