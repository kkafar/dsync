mod cli;
mod command;
mod config;
mod logging;

use clap::Parser;
use config::Config;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = cli::Cli::parse();

    let _ = logging::configure_logging(&args);
    log::info!("dsync-client start");

    let cfg = Config::from_cli(&args);

    if let Err(err) = args.command.handle(&cfg).await {
        log::error!("Command failed with error: {err}");
        return Err(err.into());
    };

    Ok(())
}
