mod utils;

use hello_world::greeter_server::{Greeter, GreeterServer};
use hello_world::{HelloReply, HelloRequest};
use tonic::transport::Server;
use tonic::{Request, Response, Status};

pub mod hello_world {
    tonic::include_proto!("helloworld");
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

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Hello, world!");
    // let addr = "[::1]:50051".parse()?;
    // let greeter = MyGreeter::default();

    if !utils::check_binary_exists("nmap") {
        Err("nmap is not installed")?;
    }

    if !utils::check_binary_exists("arp") {
        Err("arp is not installed")?;
    }

    let Some(ipv4_addrs) = utils::discover_hosts_in_local_network() else {
        return Err("Failed to find hosts in local network")?;
    };

    println!("{:?}", ipv4_addrs);

    // Server::builder()
    //     .add_service(GreeterServer::new(greeter))
    //     .serve(addr)
    //     .await?;

    return Ok(());
}
