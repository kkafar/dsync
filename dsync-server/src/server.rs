use std::{
    net::{Ipv4Addr, SocketAddrV4},
    process::Command,
    sync::Arc,
};

use config::Config;
use context::ServerContext;
use data::repo::{DataRepository, MainDataRepository};
use data::source::SqliteDataSource;
use diesel::{Connection, SqliteConnection};
use dsync_proto::services::{
    file_transfer::file_transfer_service_server::FileTransferServiceServer,
    host_discovery::host_discovery_service_server::HostDiscoveryServiceServer,
    server_control::server_control_service_server::ServerControlServiceServer,
    user_agent::user_agent_service_server::UserAgentServiceServer,
};
use tokio::sync::oneshot;
use uuid::Uuid;

use crate::server::data::source::sqlite::database::models::HostsRow;

pub mod config;
pub mod context;
pub(crate) mod data;
pub(crate) mod service;

pub struct Server {
    cfg: Config,
}

impl Server {
    pub fn new(cfg: Config) -> Self {
        Self { cfg }
    }

    pub async fn run(self) -> anyhow::Result<()> {
        log::info!("Starting the server instance");

        let connection = SqliteConnection::establish(self.cfg.database_url.to_str().unwrap())
            .expect("Failed to open db connection");

        let sqlite_ds =
            SqliteDataSource::new(connection, || self.create_this_server_info()).await?;
        let repo_arc: Arc<dyn DataRepository> =
            Arc::new(MainDataRepository::new(Box::new(sqlite_ds)));

        let server_addr = self.get_server_addr();

        let srv_ctx = Arc::new(ServerContext::new(self.cfg, repo_arc));

        let user_agent_service_instance =
            service::user_agent::UserAgentServiceImpl::new(srv_ctx.clone());
        let peer_service_instance =
            service::host_discovery::HostDiscoveryServiceImpl::new(srv_ctx.clone());
        let file_transfer_service =
            service::file_transfer::FileTransferServiceImpl::new(srv_ctx.clone());

        let (signal_tx, signal_rx) = tokio::sync::oneshot::channel::<()>();

        let server_control_service =
            service::server_control::ServerControlServiceImpl::new(srv_ctx.clone(), signal_tx);

        log::info!("Starting server at {:?}", &server_addr);

        tonic::transport::Server::builder()
            .add_service(UserAgentServiceServer::new(user_agent_service_instance))
            .add_service(HostDiscoveryServiceServer::new(peer_service_instance))
            .add_service(FileTransferServiceServer::new(file_transfer_service))
            .add_service(ServerControlServiceServer::new(server_control_service))
            .serve_with_shutdown(server_addr.into(), Self::shutdown_feature(signal_rx))
            .await?;

        anyhow::Ok(())
    }

    fn create_this_server_info(&self) -> HostsRow {
        let hostname = self.get_hostname().expect("Error while resolving hostname");

        HostsRow {
            uuid: Uuid::new_v4().to_string(),
            name: hostname.clone(),
            hostname,
            is_remote: false,
            ipv4_addr: String::from("127.0.0.1"),
            discovery_time: service::tools::time::get_current_timestamp(),
        }
    }

    fn get_hostname(&self) -> anyhow::Result<String> {
        let hostname_output = Command::new("hostname")
            .output()
            .expect("Error while running hostname command");
        let output_string = String::from_utf8(hostname_output.stdout)
            .expect("Failed to convert hostname command output to string")
            .trim()
            .to_string();
        anyhow::Ok(output_string)
    }

    fn get_server_addr(&self) -> SocketAddrV4 {
        SocketAddrV4::new(Ipv4Addr::UNSPECIFIED, self.cfg.port)
    }

    async fn shutdown_feature(signal_rx: oneshot::Receiver<()>) -> () {
        if signal_rx.await.is_err() {
            log::warn!(
                "Shutdown signal sender has most likely been dropped w/o sending a message. This might mean that the shutdown mechanism is impaired."
            );
        }
        log::info!("Requesting runtime shutdown");
    }
}
