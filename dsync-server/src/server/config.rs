#![allow(unused)]

use std::path::PathBuf;

pub(crate) mod defaults {
    pub const SERVER_PORT: u16 = 50051;
    pub const ADDR_LOOPBACK_V4: &str = "127.0.0.1";
    pub const ADDR_ALL_INTERFACES_V4: &str = "0.0.0.0";
}

pub(crate) mod keys {
    pub const DATABASE_URL: &str = "DATABASE_URL";
    pub const ENV_FILE: &str = "ENV_FILE";
    pub const SERVER_PORT: &str = "SERVER_PORT";
}

/// Running configuration for the server.
#[derive(Debug)]
pub(crate) struct RunConfiguration {
    /// Port the server should listen on.
    pub port: u16,

    /// Path to local storage database.
    pub database_url: PathBuf,

    /// Environment file that was used to load-up the envvars.
    pub env_file_path: PathBuf,
}
