mod utils;

use client_api::client_api_server::{ClientApi, ClientApiServer};
use client_api::{HostDescription, ListHostsRequest, ListHostsResponse};
use tonic::transport::Server;
use tonic::{Request, Response, Status};

pub mod client_api {
    tonic::include_proto!("client.api");
}

#[derive(Debug, Default)]
pub struct ClientApiImpl {}

#[tonic::async_trait]
impl ClientApi for ClientApiImpl {
    async fn list_hosts(
        &self,
        _: Request<ListHostsRequest>,
    ) -> Result<Response<ListHostsResponse>, Status> {
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
    println!("Hello, world!");
    let addr = "[::1]:50051".parse()?;
    let client_api_instance = ClientApiImpl::default();

    Server::builder()
        .add_service(ClientApiServer::new(client_api_instance))
        .serve(addr)
        .await?;

    return Ok(());
}
