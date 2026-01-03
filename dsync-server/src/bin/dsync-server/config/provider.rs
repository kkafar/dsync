use std::path::PathBuf;

use dsync_server::server;

use crate::{cli::Args, config::PartialConfig};

#[derive(Debug, thiserror::Error)]
pub enum ConfigLoadError {
    #[error("I/O error: {0}")]
    IoError(std::io::Error),

    #[error("Failed to parse config: {0}")]
    ParseError(String),

    #[error("Other error: {0}")]
    Other(String),
}

pub trait PartialConfigProvider {
    fn load_partial_config(&self) -> Result<PartialConfig, ConfigLoadError>;
}

/// EnvConfigProvider loads configuration from environment variables,
/// but it does not load the environment itself.
/// Assert that the environment variables are set before using this provider.
pub struct EnvConfigProvider {}

impl EnvConfigProvider {
    pub fn new() -> Self {
        EnvConfigProvider {}
    }
}

impl PartialConfigProvider for EnvConfigProvider {
    fn load_partial_config(&self) -> Result<PartialConfig, ConfigLoadError> {
        let mut config = PartialConfig::default();

        if let Ok(db_url) = dotenvy::var(server::config::keys::DATABASE_URL) {
            let db_path = PathBuf::from(db_url);
            config.database_url = Some(db_path);
        }

        if let Ok(server_port) = dotenvy::var(server::config::keys::SERVER_PORT) {
            let port = server_port
                .parse::<u16>()
                .map_err(|err| ConfigLoadError::ParseError(err.to_string()))?;
            config.port = Some(port);
        }

        Ok(config)
    }
}

pub struct CliArgsConfigProvider {
    args: Args,
}

impl CliArgsConfigProvider {
    pub fn new(args: Args) -> Self {
        CliArgsConfigProvider { args }
    }
}

impl PartialConfigProvider for CliArgsConfigProvider {
    fn load_partial_config(&self) -> Result<PartialConfig, ConfigLoadError> {
        let mut config = PartialConfig::default();

        if let Some(port) = self.args.port {
            config.port = Some(port);
        }

        if let Some(ref database_url) = self.args.db_file {
            let db_path = PathBuf::from(database_url);
            config.database_url = Some(db_path);
        }

        Ok(config)
    }
}
