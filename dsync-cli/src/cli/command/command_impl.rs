use dsync_proto::cli::{
    self, DiscoverHostsRequest, ListHostsRequest, client_api_client::ClientApiClient,
};

use super::Commands;

const LOOPBACK_ADDR_V4: &str = "http://127.0.0.1:50051";

impl Commands {
    pub(super) async fn handle_list_hosts(self) -> anyhow::Result<()> {
        let mut client = ClientApiClient::connect(LOOPBACK_ADDR_V4).await?;

        let request = tonic::Request::new(ListHostsRequest {});

        log::info!("Sending request to server");
        log::debug!("{request:?}");

        let response = client.list_hosts(request).await?;

        log::info!("Received response from server");
        log::debug!("{response:?}");

        let response_payload = response.into_inner();
        print_servers_info(&response_payload.servers_info);

        anyhow::Ok(())
    }

    pub(super) async fn handle_discover_hosts(self) -> anyhow::Result<()> {
        let mut client = ClientApiClient::connect(LOOPBACK_ADDR_V4).await?;

        let request = tonic::Request::new(DiscoverHostsRequest {});

        log::info!("Sending request to server");
        log::debug!("{request:?}");

        let response = client.discover_hosts(request).await?;

        log::info!("Received response from server");
        log::debug!("{response:?}");

        let response_payload = response.into_inner();
        print_servers_info(&response_payload.servers_info);

        anyhow::Ok(())
    }
}

fn print_servers_info(server_info_coll: &[cli::ServerInfo]) -> () {
    server_info_coll
        .into_iter()
        .enumerate()
        .for_each(|(i, info)| {
            println!("{} {}", i + 1, format_server_info(&info));
        });
}

fn format_server_info(server_info: &cli::ServerInfo) -> String {
    format!(
        "{} {} {}",
        server_info.name, server_info.hostname, server_info.hostname
    )
}
