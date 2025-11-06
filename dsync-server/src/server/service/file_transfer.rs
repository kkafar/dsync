#![allow(unused)]

pub(crate) mod session;
pub(crate) mod session_factory;
pub(crate) mod session_registry;

use std::sync::Arc;

use dsync_proto::file_transfer::{
    TransferChunkRequest, TransferChunkResponse, TransferInitRequest, TransferInitResponse,
    file_transfer_service_server::FileTransferService,
};
use tokio_stream::StreamExt;

use crate::server::{
    global_context::GlobalContext,
    service::file_transfer::{
        session_factory::FileTransferSessionFactory, session_registry::FileTransferSessionRegistry,
    },
};

// #[derive(Debug)]
pub struct FileTransferServiceImpl {
    ctx: Arc<GlobalContext>,
    session_registry: tokio::sync::Mutex<FileTransferSessionRegistry>,
    session_factory: tokio::sync::Mutex<FileTransferSessionFactory>,
}

impl FileTransferServiceImpl {
    pub fn new(ctx: Arc<GlobalContext>) -> Self {
        Self {
            ctx,
            session_registry: tokio::sync::Mutex::new(FileTransferSessionRegistry::new()),
            session_factory: tokio::sync::Mutex::new(FileTransferSessionFactory::new()),
        }
    }
}

#[tonic::async_trait]
impl FileTransferService for FileTransferServiceImpl {
    async fn transfer_init(
        &self,
        request: tonic::Request<TransferInitRequest>,
    ) -> Result<tonic::Response<TransferInitResponse>, tonic::Status> {
        let session = {
            let mut sf_guard = self.session_factory.lock().await;
            sf_guard.create_session()
        };

        let mut sr_guard = self.session_registry.lock().await;
        let session_id = session.session_id;
        sr_guard.register(session);

        Ok(tonic::Response::new(TransferInitResponse {
            session_id: session_id.into(),
        }))
    }

    async fn transfer_chunk(
        &self,
        request_stream: tonic::Request<tonic::Streaming<TransferChunkRequest>>,
    ) -> Result<tonic::Response<TransferChunkResponse>, tonic::Status> {
        let mut stream = request_stream.into_inner();

        while let Some(payload_result) = stream.next().await {
            match payload_result {
                Ok(payload) => {
                    // Store the data somewhere?
                    payload.data_buffer;
                }
                Err(status) => {
                    // Handle error
                }
            }
        }

        // Stream ended

        Ok(tonic::Response::new(TransferChunkResponse {}))
    }
}
