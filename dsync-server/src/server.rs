use std::{
    net::{Ipv4Addr, SocketAddrV4},
    process::Command,
    sync::Arc,
};

use config::RunConfiguration;
use context::ServerContext;
use database::DatabaseProxy;
use diesel::{Connection, SqliteConnection};
use dsync_proto::services::{
    file_transfer::file_transfer_service_server::FileTransferServiceServer,
    host_discovery::host_discovery_service_server::HostDiscoveryServiceServer,
    user_agent::user_agent_service_server::UserAgentServiceServer,
};
use uuid::Uuid;

use database::models::HostsRow;

pub mod config;
pub mod context;
pub mod database;
pub(crate) mod service;

pub struct Server {
    run_config: RunConfiguration,
}

impl Server {
    pub fn new(run_config: RunConfiguration) -> Self {
        Self { run_config }
    }

    pub async fn run(self) -> anyhow::Result<()> {
        log::info!("Starting the server instance");

        let connection =
            SqliteConnection::establish(self.run_config.database_url.to_str().unwrap())
                .expect("Failed to open db connection");

        let db_proxy = DatabaseProxy::new(connection);
        db_proxy
            .ensure_db_record_exists(|| self.create_this_server_info())
            .await;

        let server_addr = self.get_server_addr();

        let g_ctx = Arc::new(ServerContext {
            run_config: self.run_config,
            db_proxy: Arc::new(db_proxy),
        });

        let user_agent_service_instance =
            service::user_agent::UserAgentServiceImpl::new(g_ctx.clone());
        let peer_service_instance =
            service::host_discovery::HostDiscoveryServiceImpl::new(g_ctx.clone());
        let file_transfer_service =
            service::file_transfer::FileTransferServiceImpl::new(g_ctx.clone());

        log::info!("Starting server at {:?}", &server_addr);

        tonic::transport::Server::builder()
            .add_service(UserAgentServiceServer::new(user_agent_service_instance))
            .add_service(HostDiscoveryServiceServer::new(peer_service_instance))
            .add_service(FileTransferServiceServer::new(file_transfer_service))
            .serve(server_addr.into())
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
        SocketAddrV4::new(Ipv4Addr::UNSPECIFIED, self.run_config.port)
    }
}
