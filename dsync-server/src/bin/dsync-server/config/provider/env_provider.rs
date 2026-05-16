use std::path::PathBuf;

use dsync_server::server;

use crate::config::{
    PartialConfig,
    provider::{ConfigLoadError, PartialConfigProvider},
};

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
