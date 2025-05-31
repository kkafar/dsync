use std::ops::DerefMut;
use std::sync::Arc;

use anyhow::anyhow;
use diesel::{QueryDsl, RunQueryDsl, SelectableHelper};
use dsync_proto::p2p::peer_service_server::PeerService;
use dsync_proto::p2p::{self, HelloThereRequest, HelloThereResponse};
use tonic::{Request, Response, Status};

use crate::models::ThisServerInfoRow;
use crate::schema::server_info;

use super::global_context::GlobalContext;

// #[derive(Debug)]
pub struct PeerServiceImpl {
    ctx: Arc<GlobalContext>,
}

impl PeerServiceImpl {
    pub fn new(ctx: Arc<GlobalContext>) -> Self {
        Self { ctx }
    }

    async fn retrieve_this_server_info(&self) -> anyhow::Result<ThisServerInfoRow> {
        use crate::schema::this_server_info::dsl::*;
        let mut db_conn = self.ctx.db_conn.lock().await;

        let results = this_server_info
            .select(ThisServerInfoRow::as_select())
            .load(db_conn.deref_mut())
            .expect("Error while loading configuration");

        if results.len() != 1 {
            return Err(anyhow!(
                "Expected this_server_info to be populated with exactly one record"
            ));
        }

        // Unwrap asserted above
        let server_info = results[0].clone();

        return anyhow::Ok(server_info);
    }
}

#[tonic::async_trait]
impl PeerService for PeerServiceImpl {
    async fn hello_there(
        &self,
        request: Request<HelloThereRequest>,
    ) -> Result<Response<HelloThereResponse>, Status> {
        log::info!(target: "pslog", "Received hello_there rpc");

        let Some(peer_info) = request.into_inner().server_info else {
            log::trace!(target: "pslog", "Rejecting request due to missing peer info");
            return Err(tonic::Status::invalid_argument("Missing peer info"));
        };

        log::debug!(target: "pslog", "Connectinng peer: {}", peer_info.hostname);

        let server_info = match self.retrieve_this_server_info().await {
            Ok(data) => data,
            Err(err) => {
                log::error!(target: "pslog", "Error while fetching this server info {err}");
                return Err(tonic::Status::internal(format!(
                    "Failed to fetch peer info with error: {err}"
                )));
            }
        };

        Ok(Response::new(HelloThereResponse {
            server_info: Some(p2p::ServerInfo {
                uuid: server_info.uuid,
                name: server_info.name,
                hostname: server_info.hostname,
                address: "".to_string(),
            }),
        }))
    }
}
