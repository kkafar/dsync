mod logging;
mod utils;

mod models;
mod schema;

use std::env;

use diesel::{Connection, SqliteConnection};
use dotenvy::dotenv;
use dsync_proto::client_api::client_api_server::{ClientApi, ClientApiServer};
use dsync_proto::client_api::{HostDescription, ListHostsRequest, ListHostsResponse};
use tonic::transport::Server;
use tonic::{Request, Response, Status};

#[derive(Debug, Default)]
pub struct ClientApiImpl {}

#[tonic::async_trait]
impl ClientApi for ClientApiImpl {
    async fn list_hosts(
        &self,
        _: Request<ListHostsRequest>,
    ) -> Result<Response<ListHostsResponse>, Status> {
        log::info!("Received ListHostsRequest");

        // TODO: this could be done once, on server start.
        if !utils::check_binary_exists("nmap") {
            return Err(tonic::Status::internal("Missing binary: nmap"));
        }

        if !utils::check_binary_exists("arp") {
            return Err(tonic::Status::internal("Missing binary: arp"));
        }

        let Some(ipv4_addrs) = utils::discover_hosts_in_local_network(true) else {
            return Err(tonic::Status::internal(
                "Failed to find hosts in local network",
            ));
        };

        let host_descriptions: Vec<HostDescription> = ipv4_addrs
            .into_iter()
            .map(|addr| HostDescription {
                ipv4_addr: addr.to_string(),
            })
            .collect();

        return Ok(Response::new(ListHostsResponse { host_descriptions }));
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let _ = logging::configure_logging();

    log::trace!("dsync_server start");
    log::info!("Loading env...");
    dotenv().ok();

    let db_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    let connection = SqliteConnection::establish(&db_url).expect("Failed to open db connection");

    let addr = "[::1]:50051".parse()?;
    let client_api_instance = ClientApiImpl::default();

    log::info!("Starting server at {:?}", addr);

    Server::builder()
        .add_service(ClientApiServer::new(client_api_instance))
        .serve(addr)
        .await?;

    return Ok(());
}
