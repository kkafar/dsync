use dsync_proto::client_api::{ListHostsRequest, client_api_client::ClientApiClient};

use super::Commands;

impl Commands {
    pub(super) async fn handle_list_hosts(self) -> anyhow::Result<()> {
        let mut client = ClientApiClient::connect("http://[::1]:50051").await?;

        let request = tonic::Request::new(ListHostsRequest {});

        let response = client.list_hosts(request).await?;

        log::info!("Received response from server");
        log::debug!("{:?}", response);

        let response_payload = response.into_inner();
        response_payload
            .host_descriptions
            .into_iter()
            .for_each(|desc| {
                println!("Host at: {}, name: <not-implemented>", desc.ipv4_addr);
            });

        anyhow::Ok(())
    }
}
