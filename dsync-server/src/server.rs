use std::{process::Command, sync::Arc};

use config::RunConfiguration;
use database::DatabaseProxy;
use diesel::{Connection, SqliteConnection};
use dsync_proto::{
    server::peer_service_server::PeerServiceServer,
    user_agent::user_agent_service_server::UserAgentServiceServer,
};
use global_context::GlobalContext;
use uuid::Uuid;

use database::models::LocalServerBaseInfoRow;

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
            .ensure_db_record_exists(|| self.create_this_server_info("main".to_owned()))
            .await;

        let addr_str = format!("0.0.0.0:{}", self.run_config.port);

        let addr = addr_str.parse()?;

        let g_ctx = Arc::new(GlobalContext {
            run_config: self.run_config,
            db_proxy: Arc::new(db_proxy),
        });

        let user_agent_service_instance =
            service::user_agent::UserAgentServiceImpl::new(g_ctx.clone());
        let peer_service_instance = service::peer::PeerServiceImpl::new(g_ctx.clone());

        log::info!("Starting server at {:?}", addr);

        tonic::transport::Server::builder()
            .add_service(UserAgentServiceServer::new(user_agent_service_instance))
            .add_service(PeerServiceServer::new(peer_service_instance))
            .serve(addr)
            .await?;

        anyhow::Ok(())
    }

    fn create_this_server_info(&self, name: String) -> LocalServerBaseInfoRow {
        return LocalServerBaseInfoRow {
            uuid: Uuid::new_v4().to_string(),
            name,
            hostname: self.get_hostname().expect("Error while resolving hostname"),
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
