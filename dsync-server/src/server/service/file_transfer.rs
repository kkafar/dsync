use std::sync::Arc;

use dsync_proto::file_transfer::{
    InitTransferRequest, InitTransferResponse, file_transfer_service_server::FileTransferService,
};

use crate::server::global_context::GlobalContext;

// #[derive(Debug)]
pub struct FileTransferServiceImpl {
    ctx: Arc<GlobalContext>,
}

impl FileTransferServiceImpl {
    pub fn new(ctx: Arc<GlobalContext>) -> Self {
        Self { ctx }
    }
}

#[tonic::async_trait]
impl FileTransferService for FileTransferServiceImpl {
    async fn init_transfer(
        &self,
        _request: tonic::Request<InitTransferRequest>,
    ) -> std::result::Result<tonic::Response<InitTransferResponse>, tonic::Status> {
        Ok(tonic::Response::new(InitTransferResponse {
            is_accepted: true,
        }))
    }
}
