use dsync_proto::cli::{
    DiscoverHostsRequest, ListHostsRequest, client_api_client::ClientApiClient,
};

use crate::command::{model::LOOPBACK_ADDR_V4, utils};

pub(crate) async fn host_list() -> anyhow::Result<()> {
    let mut client = ClientApiClient::connect(LOOPBACK_ADDR_V4).await?;

    let request = tonic::Request::new(ListHostsRequest {});

    log::info!("Sending request to server");
    log::debug!("{request:?}");

    let response = client.list_hosts(request).await?;

    log::info!("Received response from server");
    log::debug!("{response:?}");

    let response_payload = response.into_inner();
    utils::print_servers_info(&response_payload.servers_info);

    anyhow::Ok(())
}

pub(crate) async fn host_discover() -> anyhow::Result<()> {
    let mut client = ClientApiClient::connect(LOOPBACK_ADDR_V4).await?;

    let request = tonic::Request::new(DiscoverHostsRequest {});

    log::info!("Sending request to server");
    log::debug!("{request:?}");

    let response = client.discover_hosts(request).await?;

    log::info!("Received response from server");
    log::debug!("{response:?}");
    let response_payload = response.into_inner();
    utils::print_servers_info(&response_payload.servers_info);

    anyhow::Ok(())
}
