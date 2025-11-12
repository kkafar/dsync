use std::{process::Command, sync::Arc};

use config::RunConfiguration;
use database::DatabaseProxy;
use diesel::{Connection, SqliteConnection};
use dsync_proto::services::{
    file_transfer::file_transfer_service_server::FileTransferServiceServer,
    host_discovery::host_discovery_service_server::HostDiscoveryServiceServer,
    user_agent::user_agent_service_server::UserAgentServiceServer,
};
use global_context::GlobalContext;
use uuid::Uuid;

use database::models::HostsRow;

use crate::utils;

pub mod config;
pub mod database;
pub mod global_context;
pub(crate) mod service;

pub(self) mod util;

pub(crate) struct Server {
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

        let addr_str = format!("127.0.0.1:{}", self.run_config.port);

        let addr = addr_str.parse()?;

        let g_ctx = Arc::new(GlobalContext {
            run_config: self.run_config,
            db_proxy: Arc::new(db_proxy),
        });

        let user_agent_service_instance =
            service::user_agent::UserAgentServiceImpl::new(g_ctx.clone());
        let peer_service_instance =
            service::host_discovery::HostDiscoveryServiceImpl::new(g_ctx.clone());
        let file_transfer_service =
            service::file_transfer::FileTransferServiceImpl::new(g_ctx.clone());

        log::info!("Starting server at {:?}", addr);

        tonic::transport::Server::builder()
            .add_service(UserAgentServiceServer::new(user_agent_service_instance))
            .add_service(HostDiscoveryServiceServer::new(peer_service_instance))
            .add_service(FileTransferServiceServer::new(file_transfer_service))
            .serve(addr)
            .await?;

        anyhow::Ok(())
    }

    fn create_this_server_info(&self) -> HostsRow {
        let hostname = self.get_hostname().expect("Error while resolving hostname");

        return HostsRow {
            uuid: Uuid::new_v4().to_string(),
            name: hostname.clone(),
            hostname: hostname,
            is_remote: false,
            ipv4_addr: String::from("127.0.0.1"),
            discovery_time: utils::time::get_current_timestamp(),
        };
    }

    fn get_hostname(&self) -> anyhow::Result<String> {
        let hostname_output = Command::new("hostname")
            .output()
            .expect("Error while running hostname command");
        let output_string = String::from_utf8(hostname_output.stdout)
            .expect("Failed to convert hostname command output to string")
            .trim()
            .to_string();
        return anyhow::Ok(output_string);
    }
}
