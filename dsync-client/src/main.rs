mod logging;

use client_api::ListHostsRequest;
use client_api::client_api_client::ClientApiClient;

pub mod client_api {
    tonic::include_proto!("client.api");
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    log::info!("dsync-client start");

    let mut client = ClientApiClient::connect("http://[::1]:50051").await?;

    let request = tonic::Request::new(ListHostsRequest {});

    let response = client.list_hosts(request).await?;

    log::info!("RESPONSE={:?}", response);

    Ok(())
}
