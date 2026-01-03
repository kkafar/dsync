// Centralized configuration for the CLI application.
// This module provides a `Config` struct that is derived from the parsed CLI options,
// with sensible defaults applied where needed.

use dsync_shared::DEFAULT_SERVER_PORT;

/// Application configuration derived from CLI inputs with defaults.
#[derive(Debug, Clone)]
pub(crate) struct Config {
    /// Port the dsync-server is running on. Defaults to `DEFAULT_SERVER_PORT` when not provided by the user.
    pub(crate) server_port: u16,
}

impl Config {
    /// Construct `Config` from the parsed CLI.
    pub(crate) fn from_cli(cli: &crate::cli::Cli) -> Self {
        Self {
            server_port: cli.server_port.unwrap_or(DEFAULT_SERVER_PORT),
        }
    }
}
