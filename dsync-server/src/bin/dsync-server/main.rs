use anyhow::Context;
use clap::Parser;
use dsync_server::server;

mod cli;
mod config;
mod logging;

use crate::{
    cli::Args,
    config::{load_config, load_env},
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    let _ = logging::configure_logging(args.log_level.unwrap_or(log::LevelFilter::Warn));
    log::info!("dsync_server start");

    if let Err(err) = load_env(args.env_file.clone()) {
        log::warn!("Failed to load environment variables: {}", err);
    };

    let config = load_config(&[
        &config::provider::CliArgsConfigProvider::new(args.clone()),
        &config::provider::EnvConfigProvider::new(),
    ])
    .context("Failed to load configuration")?;

    let server_instance = server::Server::new(config);

    server_instance.run().await?;

    return Ok(());
}
