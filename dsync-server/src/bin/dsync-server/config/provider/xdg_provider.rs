use std::path::PathBuf;

use crate::config::{PartialConfig, error::ConfigLoadError, provider::PartialConfigProvider};

/// This one provides only path for the database
pub struct XdgConfigProvider {}

impl XdgConfigProvider {
    pub fn new() -> Self {
        Self {}
    }
}

impl PartialConfigProvider for XdgConfigProvider {
    fn load_partial_config(&self) -> Result<PartialConfig, ConfigLoadError> {
        let xdg_dirs = xdg::BaseDirectories::with_prefix("dsync");

        let relative_db_path = PathBuf::from("main.db");

        let db_path = xdg_dirs.place_state_file(&relative_db_path);

        Ok(PartialConfig::new(None, db_path.ok()))
    }
}
