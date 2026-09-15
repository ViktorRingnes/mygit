use std::sync::OnceLock;

#[derive(Debug)]
pub struct MyGitData {
    pub repo_path: String,
}

static MY_GIT_DATA: OnceLock<MyGitData> = OnceLock::new();

pub fn init(repo_path: impl Into<String>) -> Result<(), MyGitData> {
    MY_GIT_DATA.set(MyGitData {
        repo_path: repo_path.into(),
    })
}

pub fn get() -> &'static MyGitData {
    MY_GIT_DATA
        .get()
        .expect("not inited")
}
