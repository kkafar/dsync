mod cli;
mod command;
mod logging;

use clap::Parser;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = cli::Cli::parse();

    let _ = logging::configure_logging(&args);
    log::info!("dsync-client start");

    if let Err(err) = args.command.handle().await {
        log::error!("Command failed with error: {err}");
        return Err(err.into());
    };

    Ok(())
}
