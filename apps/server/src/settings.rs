use std::net::{IpAddr, SocketAddr};
use std::path::{Path, PathBuf};

use config::{Config, ConfigError, Environment, File};
use serde::Deserialize;

const DEFAULT_PROFILE: &str = "development";

#[derive(Debug, Clone, Deserialize)]
pub struct Settings {
    pub repo: PathBuf,
    pub server: Server,
    pub cors: Cors,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Server {
    pub host: IpAddr,
    pub port: u16,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Cors {
    pub origins: Vec<String>,
}

impl Settings {
    pub fn load() -> Result<Self, ConfigError> {
        let dir = config_dir();
        let profile = std::env::var("MYGIT_PROFILE").unwrap_or_else(|_| DEFAULT_PROFILE.to_owned());

        let settings: Self = Config::builder()
            .add_source(File::from(dir.join("default.toml")))
            .add_source(File::from(dir.join(format!("{profile}.toml"))).required(false))
            .add_source(File::from(dir.join("local.toml")).required(false))
            .add_source(
                Environment::with_prefix("MYGIT")
                    .separator("_")
                    .try_parsing(true)
                    .list_separator(",")
                    .with_list_parse_key("cors.origins"),
            )
            .build()?
            .try_deserialize()?;

        Ok(settings.resolve(&dir))
    }

    pub fn bind_addr(&self) -> SocketAddr {
        SocketAddr::new(self.server.host, self.server.port)
    }

    fn resolve(mut self, dir: &Path) -> Self {
        let repo = dir.join(&self.repo);
        self.repo = repo.canonicalize().unwrap_or(repo);

        self
    }
}

fn config_dir() -> PathBuf {
    std::env::var_os("MYGIT_CONFIG_DIR").map_or_else(
        || PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("config"),
        PathBuf::from,
    )
}
