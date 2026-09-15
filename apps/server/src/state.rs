use crate::git::GitService;

#[derive(Clone)]
pub struct AppState {
    pub git: GitService,
}
