use crate::config::{PartialConfig, error::ConfigLoadError};

mod cli_provider;
mod env_provider;
mod xdg_provider;

pub use cli_provider::CliArgsConfigProvider;
pub use env_provider::EnvConfigProvider;
pub use xdg_provider::XdgConfigProvider;

pub trait PartialConfigProvider {
    fn load_partial_config(&self) -> Result<PartialConfig, ConfigLoadError>;
}
