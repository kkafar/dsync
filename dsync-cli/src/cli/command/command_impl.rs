use dsync_proto::cli::{ListHostsRequest, client_api_client::ClientApiClient};

use super::Commands;

impl Commands {
    pub(super) async fn handle_list_hosts(self) -> anyhow::Result<()> {
        self.handle_discover_hosts().await
    }

    pub(super) async fn handle_discover_hosts(self) -> anyhow::Result<()> {
        let mut client = ClientApiClient::connect("http://127.0.0.1:50051").await?;

        let request = tonic::Request::new(ListHostsRequest {});

        let response = client.list_hosts(request).await?;

        log::info!("Received response from server");
        log::debug!("{:?}", response);

        let response_payload = response.into_inner();
        response_payload
            .servers_info
            .into_iter()
            .enumerate()
            .for_each(|(i, info)| {
                println!(
                    "{} {}@{} ({})",
                    i + 1,
                    info.uuid,
                    info.address,
                    info.hostname
                );
            });

        anyhow::Ok(())
    }
}
