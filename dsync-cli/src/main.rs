mod cli;
mod logging;

use clap::Parser;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let _ = logging::configure_logging();
    log::info!("dsync-client start");

    let args = cli::Cli::parse();
    let _ = args.command.handle().await?;
    Ok(())
}
