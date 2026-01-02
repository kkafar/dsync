use anyhow::bail;
use dsync_proto::services::server_control::ShutdownRequest;
use dsync_shared::conn::ServiceConnFactory;

pub(crate) async fn server_shutdown() -> Result<(), anyhow::Error> {
    let request = tonic::Request::new(ShutdownRequest {});

    let mut client = ServiceConnFactory::local_server_control_service(None).await?;

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
