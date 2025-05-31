//! Cli setup for the server

use std::path::PathBuf;

use clap::Parser;

#[derive(Parser)]
#[command(about)]
pub(crate) struct Args {
    pub env_file: Option<PathBuf>,
}
