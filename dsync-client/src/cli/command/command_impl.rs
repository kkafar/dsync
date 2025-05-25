use dsync_proto::client_api::{ListHostsRequest, client_api_client::ClientApiClient};

use super::Commands;

impl Commands {
    pub(super) async fn handle_list_hosts(self) -> anyhow::Result<()> {
        let mut client = ClientApiClient::connect("http://[::1]:50051").await?;

        let request = tonic::Request::new(ListHostsRequest {});

        let response = client.list_hosts(request).await?;

        log::info!("RESPONSE={:?}", response);
        anyhow::Ok(())
    }
}
