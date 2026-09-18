use std::fs;
use std::path::Path;

use git2::{Oid, Repository, RepositoryInitOptions, Signature};
use tempfile::TempDir;

pub(crate) struct Fixture {
    pub dir: TempDir,
    pub repo: Repository,
}

impl Fixture {
    pub(crate) fn new(head: &str) -> Self {
        Self::init(head, false)
    }

    pub(crate) fn bare(head: &str) -> Self {
        Self::init(head, true)
    }

    fn init(head: &str, bare: bool) -> Self {
        let dir = TempDir::new().unwrap();

        let mut options = RepositoryInitOptions::new();
        options.initial_head(head).bare(bare);

        let repo = Repository::init_opts(dir.path(), &options).unwrap();

        Self { dir, repo }
    }

    pub(crate) fn remote_head(&self, remote: &str, branch: &str, oid: Oid) {
        self.repo
            .remote(remote, "https://example.com/repo.git")
            .ok();
        self.repo
            .reference(&format!("refs/remotes/{remote}/{branch}"), oid, true, "")
            .unwrap();
        self.repo
            .reference_symbolic(
                &format!("refs/remotes/{remote}/HEAD"),
                &format!("refs/remotes/{remote}/{branch}"),
                true,
                "",
            )
            .unwrap();
    }

    pub(crate) fn commit(&self, message: &str, files: &[(&str, &[u8])]) -> Oid {
        let mut index = self.repo.index().unwrap();

        for (path, content) in files {
            let full = self.dir.path().join(path);

            if let Some(parent) = full.parent() {
                fs::create_dir_all(parent).unwrap();
            }

            fs::write(&full, content).unwrap();
            index.add_path(Path::new(path)).unwrap();
        }

        index.write().unwrap();

        let tree = self.repo.find_tree(index.write_tree().unwrap()).unwrap();
        let signature = Signature::now("test", "test@example.com").unwrap();

        let head = self
            .repo
            .head()
            .ok()
            .and_then(|head| head.peel_to_commit().ok());

        let parents: Vec<_> = head.iter().collect();

        self.repo
            .commit(
                Some("HEAD"),
                &signature,
                &signature,
                message,
                &tree,
                &parents,
            )
            .unwrap()
    }

    #[cfg(unix)]
    pub(crate) fn symlink(&self, link: &str, target: &str) {
        std::os::unix::fs::symlink(target, self.dir.path().join(link)).unwrap();

        let mut index = self.repo.index().unwrap();
        index.add_path(Path::new(link)).unwrap();
        index.write().unwrap();
    }

    pub(crate) fn branch(&self, name: &str, oid: Oid) {
        self.repo
            .branch(name, &self.repo.find_commit(oid).unwrap(), false)
            .unwrap();
    }
}
