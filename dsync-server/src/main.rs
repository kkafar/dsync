mod cli;
mod logging;
mod server;

use std::{env, path::PathBuf};

use clap::Parser;
use cli::Args;

use crate::server::config::defaults;

fn load_env(maybe_env_file: Option<&std::path::PathBuf>) -> anyhow::Result<std::path::PathBuf> {
    log::info!("Loading env...");
    if let Some(env_file) = maybe_env_file {
        dotenvy::from_path(env_file)?;
        log::info!("Env loaded from {:?}", env_file);
        Ok(env_file.to_owned())
    } else {
        let env_file = dotenvy::dotenv()?;
        log::info!("Env loaded from {:?}", env_file);
        Ok(env_file)
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    let _ = logging::configure_logging(args.log_level.unwrap_or(log::LevelFilter::Warn));
    log::info!("dsync_server start");

    let env_file_path_from_env = env::var(server::config::keys::ENV_FILE)
        .ok()
        .map(PathBuf::from);

    let env_file_path_input = args.env_file.as_ref().or(env_file_path_from_env.as_ref());

    let env_file_path =
        load_env(env_file_path_input).expect("Failure while environment initialization");

    let database_url = env::var(server::config::keys::DATABASE_URL).unwrap_or_else(|_| {
        panic!(
            "{} env variable must be set",
            server::config::keys::DATABASE_URL
        )
    });

    let server_port_env = env::var(server::config::keys::SERVER_PORT)
        .ok()
        .map(|port_string| port_string.parse::<u16>().unwrap());

    let server_instance = server::Server::new(server::config::RunConfiguration {
        port: args
            .port
            .unwrap_or(server_port_env.unwrap_or(defaults::SERVER_PORT)),
        database_url: PathBuf::from(database_url),
        env_file_path,
    });

    server_instance.run().await?;

    return Ok(());
}
