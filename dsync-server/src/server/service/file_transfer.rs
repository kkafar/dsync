#![allow(unused)]

pub(crate) mod session;
pub(crate) mod session_factory;
pub(crate) mod session_registry;

use std::sync::Arc;

use dsync_proto::file_transfer::{
    TransferChunkRequest, TransferChunkResponse, TransferInitRequest, TransferInitResponse,
    file_transfer_service_server::FileTransferService,
};
use tokio::{
    fs::{File, OpenOptions},
    io::{AsyncWriteExt, BufWriter},
};
use tokio_stream::StreamExt;
use tonic::IntoRequest;

use crate::server::{
    global_context::GlobalContext,
    service::file_transfer::{
        session::{FileTransferSession, SessionId},
        session_factory::FileTransferSessionFactory,
        session_registry::FileTransferSessionRegistry,
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
        let request_inner = request.into_inner();

        let session = {
            let mut sf_guard = self.session_factory.lock().await;
            sf_guard.create_session(request_inner)
        };

        let session_id = {
            let session_id_raw = session.session_id.0;
            let mut sr_guard = self.session_registry.lock().await;
            sr_guard.register(session);
            session_id_raw
        };

        Ok(tonic::Response::new(TransferInitResponse {
            session_id: session_id.into(),
        }))
    }

    async fn transfer_chunk(
        &self,
        request_stream: tonic::Request<tonic::Streaming<TransferChunkRequest>>,
    ) -> Result<tonic::Response<TransferChunkResponse>, tonic::Status> {
        let mut stream = request_stream.into_inner().peekable();

        let first_chunk = stream.peek().await.unwrap().as_ref().unwrap();
        let session_id = SessionId(first_chunk.session_id);
        assert!(session_id.is_valid());

        let session_opt = {
            let mut sreg = self.session_registry.lock().await;
            sreg.get_session(session_id)
        };

        let Some(session) = session_opt else {
            return Err(tonic::Status::failed_precondition(
                "session-not-estabilished",
            ));
        };

        // Prepare writer
        let file_handle = OpenOptions::new()
            .write(true)
            .create(true)
            .open("transfer_file_text.txt")
            .await
            .unwrap();

        let mut writer = BufWriter::new(file_handle);

        while let Some(payload_result) = stream.next().await {
            match payload_result {
                Ok(payload) => {
                    assert_eq!(session_id.0, payload.session_id);

                    // let data_buf = bytes

                    assert_eq!(
                        payload.data_buffer.len(),
                        session.transfer_init_request.chunk_size as usize
                    );
                    self.write_chunk_to_file(&mut writer, payload.data_buffer);
                }
                Err(status) => {
                    // TODO: Handle error
                }
            }
        }

        // Make sure the buffer is flushed
        let _flush_res = writer.flush().await;

        // Stream ended

        Ok(tonic::Response::new(TransferChunkResponse {}))
    }
}

impl FileTransferServiceImpl {
    async fn write_chunk_to_file(&self, writer: &mut BufWriter<File>, data_buf: Vec<u8>) {
        let n_bytes = data_buf.len();
        let mut n_bytes_written = 0usize;

        while let Ok(write_size) = writer.write_buf(&mut data_buf.as_ref()).await {
            n_bytes_written += write_size;
            if n_bytes_written >= n_bytes {
                break;
            }
        }
    }
}
