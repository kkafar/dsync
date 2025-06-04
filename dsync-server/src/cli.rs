//! Cli setup for the server

use std::path::PathBuf;

use clap::Parser;

#[derive(Parser)]
#[command(about)]
pub(crate) struct Args {
    #[arg(
        long,
        help = "Path to file defining the environment variables. Please note that this will NOT override existing environment variables in case of collisions."
    )]
    pub env_file: Option<PathBuf>,

    #[arg(
        long,
        help = "Path to local db. This will overwrite DATABASE_URL env variable if set. Might be necessary to use in case of running the server binary outside of dsync-server workspace dir."
    )]
    pub db_file: Option<PathBuf>,

    #[arg(
        long,
        help = "Port number for the server to listen on. This will overwrite SERVER_PORT env variable if set."
    )]
    pub port: Option<i32>,

    #[arg(short = 'l', long, help = "Configure logging level of the client.")]
    pub log_level: Option<log::LevelFilter>,
}
