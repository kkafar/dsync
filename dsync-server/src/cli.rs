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
}
