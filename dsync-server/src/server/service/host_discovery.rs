use dsync_proto::model::server::HostInfo;
use dsync_proto::services::host_discovery::host_discovery_service_server::HostDiscoveryService;
use dsync_proto::services::host_discovery::{GeneralKenobiResponse, HelloThereRequest};
use std::sync::Arc;
use tonic::{Request, Response, Status};

use crate::server::database::models::HostsRow;
use crate::server::global_context::GlobalContext;

// #[derive(Debug)]
pub struct HostDiscoveryServiceImpl {
    ctx: Arc<GlobalContext>,
}

impl HostDiscoveryServiceImpl {
    pub fn new(ctx: Arc<GlobalContext>) -> Self {
        Self { ctx }
    }

    async fn retrieve_local_server_info(&self) -> anyhow::Result<HostsRow> {
        anyhow::Ok(self.ctx.db_proxy.fetch_local_server_info().await?)
    }
}

#[tonic::async_trait]
impl HostDiscoveryService for HostDiscoveryServiceImpl {
    async fn hello_there(
        &self,
        request: Request<HelloThereRequest>,
    ) -> Result<Response<GeneralKenobiResponse>, Status> {
        log::info!(target: "pslog", "Received hello_there rpc");

        let Some(peer_info) = request.into_inner().host_info else {
            log::trace!(target: "pslog", "Rejecting request due to missing peer info");
            return Err(tonic::Status::invalid_argument("Missing peer info"));
        };

        log::debug!(target: "pslog", "Connectinng peer: {}", peer_info.hostname);

        let server_info = match self.retrieve_local_server_info().await {
            Ok(data) => data,
            Err(err) => {
                log::error!(target: "pslog", "Error while fetching this server info {err}");
                return Err(tonic::Status::internal(format!(
                    "Failed to fetch peer info with error: {err}"
                )));
            }
        };

        Ok(Response::new(GeneralKenobiResponse {
            host_info: Some(HostInfo {
                uuid: server_info.uuid,
                name: server_info.name,
                hostname: server_info.hostname,
                address: "".to_string(),
            }),
        }))
    }
}
