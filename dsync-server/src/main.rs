mod api;
mod logging;
mod utils;

mod models;
mod schema;

use std::env;
use std::process::Command;

use diesel::{Connection, QueryDsl, RunQueryDsl, SelectableHelper, SqliteConnection};
use dotenvy::dotenv;
use dsync_proto::client_api::client_api_server::ClientApiServer;
use models::ThisServerInfoRow;
use tonic::transport::Server;
use uuid::Uuid;

fn ensure_db_record_exists(connection: &mut SqliteConnection) {
    use self::schema::this_server_info::dsl::*;

    let results = this_server_info
        .select(ThisServerInfoRow::as_select())
        .load(connection)
        .expect("Error while loading configuration");

    if results.is_empty() {
        log::info!("Server info table empty - generating server info");
        let server_info = create_this_server_info("main".to_owned());
        save_this_server_info_to_db(connection, server_info);
    } else if results.len() == 1 {
        log::trace!("Server info exists");
    } else {
        log::error!("Corrupted state of this_server_info! More than single record present!");
        panic!("Corrupted state of this_server_info! More than single record present!");
    }
}

fn save_this_server_info_to_db(connection: &mut SqliteConnection, server_info: ThisServerInfoRow) {
    use self::schema::this_server_info;
    diesel::insert_into(this_server_info::table)
        .values(&server_info)
        .execute(connection)
        .expect("Failed to insert server info to db");
}

fn create_this_server_info(name: String) -> ThisServerInfoRow {
    return ThisServerInfoRow {
        uuid: Uuid::new_v4().to_string(),
        name,
        hostname: get_hostname().expect("Error while resolving hostname"),
    };
}

fn get_hostname() -> anyhow::Result<String> {
    let hostname_output = Command::new("hostname")
        .output()
        .expect("Error while running hostname command");
    let output_string = String::from_utf8(hostname_output.stdout)
        .expect("Failed to convert hostname command output to string");
    return anyhow::Ok(output_string);
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let _ = logging::configure_logging();

    log::trace!("dsync_server start");
    log::info!("Loading env...");
    dotenv().ok();

    let db_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    let mut connection =
        SqliteConnection::establish(&db_url).expect("Failed to open db connection");

    ensure_db_record_exists(&mut connection);

    let addr = "[::1]:50051".parse()?;
    let client_api_instance = api::ClientApiImpl::default();

    log::info!("Starting server at {:?}", addr);

    Server::builder()
        .add_service(ClientApiServer::new(client_api_instance))
        .serve(addr)
        .await?;

    return Ok(());
}
