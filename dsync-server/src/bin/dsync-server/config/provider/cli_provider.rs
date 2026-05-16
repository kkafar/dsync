use std::path::PathBuf;

use crate::{
    cli::Args,
    config::{
        PartialConfig,
        provider::{ConfigLoadError, PartialConfigProvider},
    },
};

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
