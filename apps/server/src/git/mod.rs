use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};

use git2::{Error, Repository};
use tokio::task::spawn_blocking;

pub mod branch;

#[derive(Clone)]
pub struct GitService(Arc<Pool>);

struct Pool {
    path: PathBuf,
    idle: Mutex<Vec<Repository>>,
}

impl GitService {
    pub fn open(path: &Path) -> Result<Self, Error> {
        Ok(Self(Arc::new(Pool {
            path: path.to_owned(),
            idle: Mutex::new(vec![Repository::open(path)?]),
        })))
    }

    pub async fn read<T: Send + 'static>(
        &self,
        read: impl FnOnce(&Repository) -> Result<T, Error> + Send + 'static,
    ) -> Result<T, Error> {
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
        .map_err(|error| Error::from_str(&error.to_string()))?
    }
}

impl Pool {
    fn idle(&self) -> MutexGuard<'_, Vec<Repository>> {
        self.idle.lock().unwrap_or_else(PoisonError::into_inner)
    }
}
