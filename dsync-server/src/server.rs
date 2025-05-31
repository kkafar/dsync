use std::{process::Command, sync::Arc};

use config::RunConfiguration;
use diesel::{Connection, QueryDsl, RunQueryDsl, SelectableHelper, SqliteConnection};
use dsync_proto::{
    client_api::client_api_server::ClientApiServer, p2p::peer_service_server::PeerServiceServer,
};
use global_context::GlobalContext;
use uuid::Uuid;

use crate::models::ThisServerInfoRow;

pub mod api;
pub mod config;
pub mod database;
pub mod global_context;
pub mod peer_service;

pub(crate) struct Server {
    run_config: RunConfiguration,
}

impl Server {
    pub fn new(run_config: RunConfiguration) -> Self {
        Self { run_config }
    }

    pub async fn run(self) -> anyhow::Result<()> {
        log::info!("Starting the server instance");

        let mut connection =
            SqliteConnection::establish(self.run_config.database_url.to_str().unwrap())
                .expect("Failed to open db connection");

        self.ensure_db_record_exists(&mut connection);

        let addr_str = format!("[::1]:{}", self.run_config.port);

        let addr = addr_str.parse()?;
        let client_api_instance = api::ClientApiImpl::default();

        let g_ctx = Arc::new(GlobalContext {
            run_config: self.run_config,
            db_conn: Arc::new(tokio::sync::Mutex::new(connection)),
        });

        let peer_service_instance = peer_service::PeerServiceImpl::new(g_ctx.clone());

        log::info!("Starting server at {:?}", addr);

        tonic::transport::Server::builder()
            .add_service(ClientApiServer::new(client_api_instance))
            .add_service(PeerServiceServer::new(peer_service_instance))
            .serve(addr)
            .await?;

        anyhow::Ok(())
    }

    fn ensure_db_record_exists(&self, connection: &mut SqliteConnection) {
        use crate::schema::this_server_info::dsl::*;

        let results = this_server_info
            .select(ThisServerInfoRow::as_select())
            .load(connection)
            .expect("Error while loading configuration");

        if results.is_empty() {
            log::info!("Server info table empty - generating server info");
            let server_info = self.create_this_server_info("main".to_owned());
            self.save_this_server_info_to_db(connection, server_info);
        } else if results.len() == 1 {
            log::trace!("Server info exists");
        } else {
            log::error!("Corrupted state of this_server_info! More than single record present!");
            panic!("Corrupted state of this_server_info! More than single record present!");
        }
    }

    fn save_this_server_info_to_db(
        &self,
        connection: &mut SqliteConnection,
        server_info: ThisServerInfoRow,
    ) {
        use crate::schema::this_server_info;
        diesel::insert_into(this_server_info::table)
            .values(&server_info)
            .execute(connection)
            .expect("Failed to insert server info to db");
    }

    fn create_this_server_info(&self, name: String) -> ThisServerInfoRow {
        return ThisServerInfoRow {
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
            .expect("Failed to convert hostname command output to string");
        return anyhow::Ok(output_string);
    }
}
