mod cli;
mod logging;
mod models;
mod schema;
mod server;
mod utils;

use std::env;

use clap::Parser;
use cli::Args;

fn configure_env(
    maybe_env_file: Option<&std::path::PathBuf>,
) -> anyhow::Result<std::path::PathBuf> {
    log::info!("Loading env...");
    if let Some(env_file) = maybe_env_file {
        let _ = dotenvy::from_path(env_file)?;
        log::info!("Env loaded from {:?}", env_file);
        return Ok(env_file.to_owned());
    } else {
        let env_file = dotenvy::dotenv()?;
        log::info!("Env loaded from {:?}", env_file);
        return Ok(env_file);
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let _ = logging::configure_logging();
    log::trace!("dsync_server start");

    let args = Args::parse();

    let env_file_path =
        configure_env(args.env_file.as_ref()).expect("Failure while environment initialization");

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL env var must be set");

    let server_instance = server::Server::new(server::config::RunConfiguration {
        port: 50051,
        database_url: std::path::PathBuf::from(database_url),
        env_file_path,
    });

    server_instance.run().await?;

    return Ok(());
}
