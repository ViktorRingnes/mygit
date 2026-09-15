use tokio::net::TcpListener;

use server::git::GitService;
use server::settings::Settings;
use server::state::AppState;

#[tokio::main]
async fn main() {
    if let Err(error) = run().await {
        eprintln!("fatal: {error}");
        std::process::exit(1);
    }
}

async fn run() -> Result<(), Box<dyn std::error::Error>> {
    let settings = Settings::load()?;

    let git = GitService::open(&settings.repo)?;

    let listener = TcpListener::bind(settings.bind_addr()).await?;
    eprintln!(
        "serving {} on http://{}",
        settings.repo.display(),
        listener.local_addr()?
    );

    axum::serve(listener, server::app(&settings, AppState { git })?).await?;

    Ok(())
}
