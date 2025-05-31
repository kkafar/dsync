use std::path::PathBuf;

/// Running configuration for the server.
pub(crate) struct RunConfiguration {
    /// Port the server should listen on.
    pub port: i32,

    /// Path to local storage database.
    pub database_url: PathBuf,

    /// Environment file that was used to load-up the envvars.
    pub env_file_path: PathBuf,
}
