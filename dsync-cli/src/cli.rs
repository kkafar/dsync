//! Definition of CLI.

use clap::Parser;

use command::Commands;

pub(crate) mod command;

#[derive(Parser)]
#[command(about)]
pub(crate) struct Cli {
    #[arg(short = 'l', long, help = "Configure logging level of the client")]
    pub(crate) log_level: Option<log::LevelFilter>,

    #[arg(short = 'p', long, help = "Port the dsync-server is running on")]
    pub(crate) server_port: Option<u16>,

    #[command(subcommand)]
    pub(crate) command: Commands,
}
