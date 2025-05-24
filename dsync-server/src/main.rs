mod utils;

use client_api::client_api_server::{ClientApi, ClientApiServer};
use client_api::{HostDescription, ListHostsRequest, ListHostsResponse};
use hello_world::greeter_server::{Greeter, GreeterServer};
use hello_world::{HelloReply, HelloRequest};
use tonic::transport::Server;
use tonic::{Request, Response, Status};

pub mod hello_world {
    tonic::include_proto!("helloworld");
}

pub mod client_api {
    tonic::include_proto!("client.api");
}

#[derive(Debug, Default)]
pub struct MyGreeter {}

#[tonic::async_trait]
impl Greeter for MyGreeter {
    async fn say_hello(
        &self,
        request: Request<HelloRequest>,
    ) -> Result<Response<HelloReply>, Status> {
        println!("Got a request: {:?}", request);
        let reply = HelloReply {
            message: format!("Hello {}!", request.into_inner().name),
        };

        Ok(Response::new(reply))
    }
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
    let greeter = MyGreeter::default();
    let client_api_instance = ClientApiImpl::default();

    Server::builder()
        .add_service(GreeterServer::new(greeter))
        .add_service(ClientApiServer::new(client_api_instance))
        .serve(addr)
        .await?;

    return Ok(());
}
