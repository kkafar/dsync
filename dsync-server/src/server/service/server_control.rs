use std::sync::{Arc, Mutex};

use dsync_proto::services::server_control::{
    ShutdownRequest, ShutdownResponse, server_control_service_server::ServerControlService,
};
use tokio::sync::oneshot;

use crate::server::context::ServerContext;

pub struct ServerControlServiceImpl {
    _ctx: Arc<ServerContext>,
    shutdown_sig: Mutex<Option<oneshot::Sender<()>>>,
}

impl ServerControlServiceImpl {
    pub fn new(ctx: Arc<ServerContext>, shutdown_sig: oneshot::Sender<()>) -> Self {
        Self {
            _ctx: ctx,
            shutdown_sig: Mutex::new(Some(shutdown_sig)),
        }
    }
}

#[tonic::async_trait]
impl ServerControlService for ServerControlServiceImpl {
    async fn shutdown(
        &self,
        _request: tonic::Request<ShutdownRequest>,
    ) -> Result<tonic::Response<ShutdownResponse>, tonic::Status> {
        log::info!("Received shutdown request");

        // Extract the sender while holding the lock
        let shutdown_sig = {
            let mut sig_guard = self
                .shutdown_sig
                .lock()
                .expect("Failed to acquire shutdown signal lock");

            sig_guard.take().ok_or_else(|| {
                tonic::Status::failed_precondition("Server has already been asked to shutdown")
            })?
        };

        // Prepare response
        let response = Ok(tonic::Response::new(ShutdownResponse {}));

        // Schedule shutdown AFTER the response future completes
        tokio::spawn(async move {
            // Yield once to ensure the response is flushed
            tokio::task::yield_now().await;
            let _ = shutdown_sig.send(());
        });

        response
    }
}
