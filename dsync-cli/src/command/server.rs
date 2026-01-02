use anyhow::{Context, bail};
use dsync_proto::services::server_control::{
    ShutdownRequest, server_control_service_client::ServerControlServiceClient,
};

use crate::command::model::LOOPBACK_ADDR_V4;

pub(crate) async fn server_shutdown() -> Result<(), anyhow::Error> {
    let request = tonic::Request::new(ShutdownRequest {});

    let mut client = ServerControlServiceClient::connect(LOOPBACK_ADDR_V4)
        .await
        .context("Failed to connect to server")?;

    log::trace!("Sending request to server");
    log::debug!("{request:?}");

    let response = client.shutdown(request).await;
    match response {
        Ok(_) => {
            log::info!("Server ACK");
            Ok(())
        }
        Err(status) => {
            log::error!("Received ERROR response from server\n{status:?}");
            bail!("Received ERROR response from server: {status:?}")
        }
    }
}
