#![allow(unused)]

pub(crate) mod session;
pub(crate) mod session_factory;
pub(crate) mod session_registry;

use std::{
    path::{Path, PathBuf},
    str::FromStr,
    sync::Arc,
};

use async_stream::stream;
use dsync_proto::services::file_transfer::{
    TransferChunkRequest, TransferChunkResponse, TransferInitRequest, TransferInitResponse,
    TransferSubmitRequest, TransferSubmitResponse,
    file_transfer_service_client::FileTransferServiceClient,
    file_transfer_service_server::FileTransferService,
};
use tokio::{
    fs::{File, OpenOptions, metadata},
    io::{AsyncRead, AsyncReadExt, AsyncWriteExt, BufWriter},
};
use tokio_stream::StreamExt;
use tonic::{IntoRequest, transport::Channel};

use crate::server::{
    config::defaults,
    global_context::GlobalContext,
    service::{
        file_transfer::{
            session::{FileTransferSession, SessionId},
            session_factory::FileTransferSessionFactory,
            session_registry::FileTransferSessionRegistry,
        },
        tools,
    },
};

// #[derive(Debug)]
pub struct FileTransferServiceImpl {
    global_ctx: Arc<GlobalContext>,
    session_registry: tokio::sync::Mutex<FileTransferSessionRegistry>,
    session_factory: tokio::sync::Mutex<FileTransferSessionFactory>,
}

impl FileTransferServiceImpl {
    pub fn new(ctx: Arc<GlobalContext>) -> Self {
        Self {
            global_ctx: ctx,
            session_registry: tokio::sync::Mutex::new(FileTransferSessionRegistry::new()),
            session_factory: tokio::sync::Mutex::new(FileTransferSessionFactory::new()),
        }
    }
}

#[tonic::async_trait]
impl FileTransferService for FileTransferServiceImpl {
    async fn transfer_submit(
        &self,
        request: tonic::Request<TransferSubmitRequest>,
    ) -> Result<tonic::Response<TransferSubmitResponse>, tonic::Status> {
        log::trace!("Received TransferSubmitRequest");
        // Here we received request to transfer a file from this server, to another one pointed
        // by the request.
        // We need to:
        //  * verify that the file path is absolute & exists,
        //  * extract the file information (size, hash),
        //  * send init request to destination host,
        //  * on success, schedule data transfer.

        let request_inner = request.into_inner();

        // Step 1
        // Verify that the file path is absolute & the file exists.

        let file_path_src: PathBuf = PathBuf::from_str(&request_inner.file_path_src)
            .expect("Path conversion should never fail");

        if !file_path_src.is_absolute() {
            return Err(tonic::Status::invalid_argument("src-path-not-absolute"));
        }

        // TODO: Support not only files, but also directories
        if !file_path_src.is_file() {
            return Err(tonic::Status::invalid_argument("src-path-not-a-file"));
        }

        // Step 2
        // Extract necessary file information

        let Ok(file_metadata) = file_path_src.metadata() else {
            return Err(tonic::Status::internal("src-path-failed-metadata-fetch"));
        };

        let Ok(file_size_bytes) = i64::try_from(file_metadata.len()) else {
            return Err(tonic::Status::internal("file-size-conversion-fail"));
        };

        let Ok(file_sha1) =
            tools::file::compute_sha1_hash_from_file_async(&file_path_src, None).await
        else {
            return Err(tonic::Status::internal("file-sh1-comput-fail"));
        };

        log::debug!(
            "File info - size: {} bytes, sha1: {}",
            file_size_bytes,
            file_sha1
        );

        // Step 3
        // Send init message to destination host
        let Ok(host_data) = self
            .global_ctx
            .db_proxy
            .fetch_host_by_uuid(&request_inner.host_dst_uuid)
            .await
        else {
            return Err(tonic::Status::internal("host-dst-addr-missing"));
        };

        // FIXME: host_dst_addr most likely does not have port information attached
        let Ok(mut fts_client) = FileTransferServiceClient::connect(
            tools::net::ipv4_into_connection_addr(&host_data.ipv4_addr, defaults::SERVER_PORT),
        )
        .await
        else {
            return Err(tonic::Status::failed_precondition("fts-connection-fail"));
        };

        let transfer_init_request = TransferInitRequest {
            file_path_src: request_inner.file_path_src,
            file_path_dst: request_inner.file_path_dst,
            file_sha1: file_sha1,
            file_size_bytes: file_size_bytes,
            chunk_size: 1024 * 8,
        };

        let result = fts_client
            .transfer_init(transfer_init_request.clone())
            .await;

        let transfer_init_response = match result {
            Ok(response) => {
                let response = response.into_inner();
                response
            }
            Err(status) => {
                // log::error!(format!("FTS at {} rejected transfer request: {}", &request_inner.host_dst_uuid, status));
                return Err(tonic::Status::failed_precondition("fts-rejected"));
            }
        };

        // Step 4
        // Schedule data transfer
        tokio::spawn(Self::transfer_file_impl(
            fts_client,
            transfer_init_request,
            transfer_init_response,
        ));

        Ok(tonic::Response::new(TransferSubmitResponse {}))
    }

    async fn transfer_init(
        &self,
        request: tonic::Request<TransferInitRequest>,
    ) -> Result<tonic::Response<TransferInitResponse>, tonic::Status> {
        log::trace!("Received TransferInitRequest");
        // This message means that some other server (or the very same) wants to tranfser file
        // to us.
        // We need to either decline the request & provide a reason,
        // or accept the request & prepare for follow-up data transfer.

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
        log::trace!("Received TransferChunkRequest");
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
            .open(&session.transfer_init_request.file_path_dst) // TODO: Sanitize path
            .await
            .unwrap();

        let mut writer = BufWriter::new(file_handle);
        let mut hasher = sha1_smol::Sha1::new();

        while let Some(payload_result) = stream.next().await {
            match payload_result {
                Ok(payload) => {
                    assert_eq!(session_id.0, payload.session_id);

                    // let data_buf = bytes

                    log::debug!("Received chunk of size: {}", payload.data_buffer.len());
                    assert!(
                        payload.data_buffer.len()
                            <= session.transfer_init_request.chunk_size as usize
                    );
                    hasher.update(&payload.data_buffer);
                    self.write_chunk_to_file(&mut writer, payload.data_buffer)
                        .await;
                }
                Err(status) => {} // TODO: Handle error
            }
        }

        // Make sure the buffer is flushed
        let _flush_res = writer.flush().await;

        // Stream ended -> remove session

        {
            let mut sreg = self.session_registry.lock().await;
            assert!(sreg.unregister(session_id));
        }

        // Compare hashes
        let file_hash = hasher.digest().to_string();
        if file_hash != session.transfer_init_request.file_sha1 {
            log::error!(
                "File hash mismatch! Expected: {}, got: {}",
                session.transfer_init_request.file_sha1,
                file_hash
            );
            return Err(tonic::Status::invalid_argument("file-hash-mismatch"));
        }

        Ok(tonic::Response::new(TransferChunkResponse {}))
    }
}

impl FileTransferServiceImpl {
    async fn write_chunk_to_file(&self, writer: &mut BufWriter<File>, data_buf: Vec<u8>) {
        if data_buf.is_empty() {
            log::debug!("Return because buffer is empty");
            return;
        }

        let n_bytes = data_buf.len();
        let mut n_bytes_written = 0usize;

        log::debug!("Write to buffer - start");
        while let Ok(write_size) = writer.write_buf(&mut data_buf.as_ref()).await {
            log::debug!("Wrote {} bytes", write_size);
            n_bytes_written += write_size;
            if n_bytes_written >= n_bytes {
                break;
            }
        }
    }

    async fn transfer_file_impl(
        mut client: FileTransferServiceClient<Channel>,
        init_request: TransferInitRequest,
        init_response: TransferInitResponse,
    ) {
        log::trace!("Sending TransferChunkRequest");

        let file_path = PathBuf::from(&init_request.file_path_src);

        debug_assert!(file_path.is_file());

        let mut file_handle = OpenOptions::new()
            .read(true)
            .open(&file_path)
            .await
            .expect("Failed to open the file");

        let mut buffer = bytes::BytesMut::with_capacity(init_request.chunk_size as usize);

        let stream = stream! {
            let mut chunk_id = 0;
            while let Some(read_count) = file_handle.read_buf(&mut buffer).await.ok() {
                if read_count == 0 {
                    break;
                } else {
                    yield TransferChunkRequest {
                        session_id: init_response.session_id,
                        chunk_id: chunk_id,
                        data_buffer: buffer.to_vec(), // FIXME: WE COPY HERE HARD
                    };
                    chunk_id += 1;
                    buffer.clear();
                }
            }
        };

        client.transfer_chunk(stream).await;
    }
}
