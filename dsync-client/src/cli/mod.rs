use clap::Parser;
use command::Commands;

pub(crate) mod command;

#[derive(Parser)]
#[command(about)]
pub(crate) struct Cli {
    #[command(subcommand)]
    pub(crate) command: Commands,
}
