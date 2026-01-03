use std::path::PathBuf;

use dsync_server::server::config::{Config, keys::ENV_FILE};

use crate::config::provider::PartialConfigProvider;

pub mod provider;

#[derive(Debug, Clone, Default)]
pub struct PartialConfig {
    pub port: Option<u16>,
    pub database_url: Option<PathBuf>,
}

impl PartialConfig {
    pub fn new(port: Option<u16>, database_url: Option<PathBuf>) -> Self {
        Self { port, database_url }
    }

    /// Merge two PartialConfig instances, prioritizing values from self.
    pub fn merge(self, other: PartialConfig) -> Self {
        Self {
            port: self.port.or(other.port),
            database_url: self.database_url.or(other.database_url),
        }
    }
}

impl TryInto<Config> for PartialConfig {
    type Error = anyhow::Error;

    fn try_into(self) -> Result<Config, Self::Error> {
        let port = self
            .port
            .ok_or_else(|| anyhow::anyhow!("Port is required"))?;
        let database_url = self
            .database_url
            .ok_or_else(|| anyhow::anyhow!("Database URL is required"))?;

        Ok(Config { port, database_url })
    }
}

pub fn load_env(maybe_env_file: Option<PathBuf>) -> Result<PathBuf, dotenvy::Error> {
    let env_file = maybe_env_file.or_else(|| dotenvy::var(ENV_FILE).ok().map(PathBuf::from));

    log::info!("Loading env...");
    if let Some(ref env_file) = env_file {
        dotenvy::from_path(env_file)?;
        log::info!("Env loaded from {:?}", env_file);
        Ok(env_file.to_owned())
    } else {
        let env_file = dotenvy::dotenv()?;
        log::info!("Env loaded from {:?}", env_file);
        Ok(env_file)
    }
}

/// Load configuration from multiple providers. Providers are tried in order,
/// therefore lower-indexed providers take precedence.
pub fn load_config(providers: &[&dyn PartialConfigProvider]) -> Result<Config, anyhow::Error> {
    let mut config = PartialConfig::default();

    for provider in providers {
        config = config.merge(provider.load_partial_config()?);
    }

    config.try_into()
}
