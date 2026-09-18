use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};

use git2::Repository;
use tokio::task::spawn_blocking;

pub mod branch;
pub mod commit;
pub mod error;
pub mod file;
pub mod rev;
pub mod tree;

#[cfg(test)]
pub(crate) mod fixture;

use error::GitError;

#[derive(Clone)]
pub struct GitService(Arc<Pool>);

struct Pool {
    path: PathBuf,
    idle: Mutex<Vec<Repository>>,
}

impl GitService {
    pub fn open(path: &Path) -> Result<Self, GitError> {
        Ok(Self(Arc::new(Pool {
            path: path.to_owned(),
            idle: Mutex::new(vec![Repository::open(path)?]),
        })))
    }

    pub async fn read<T: Send + 'static>(
        &self,
        read: impl FnOnce(&Repository) -> Result<T, GitError> + Send + 'static,
    ) -> Result<T, GitError> {
        let pool = Arc::clone(&self.0);

        spawn_blocking(move || {
            let repo = match pool.idle().pop() {
                Some(repo) => repo,
                None => Repository::open(&pool.path)?,
            };

            let result = read(&repo);
            pool.idle().push(repo);

            result
        })
        .await
        .map_err(|error| GitError::Internal(error.to_string()))?
    }
}

impl Pool {
    fn idle(&self) -> MutexGuard<'_, Vec<Repository>> {
        self.idle.lock().unwrap_or_else(PoisonError::into_inner)
    }
}

pub(crate) fn remotes(repo: &Repository) -> Result<Vec<String>, GitError> {
    let list = repo.remotes()?;

    let mut remotes: Vec<String> = list
        .iter()
        .filter_map(|remote| remote.ok().flatten())
        .map(ToOwned::to_owned)
        .collect();

    remotes.sort_unstable_by_key(|remote| remote != "origin");

    Ok(remotes)
}
