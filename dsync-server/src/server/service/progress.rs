use dsync_proto::services::progress::{
    AttachToProgressStreamRequest, AttachToProgressStreamResponse,
    progress_service_server::ProgressService,
};

pub(crate) struct ProgressServiceImpl;

#[tonic::async_trait]
impl ProgressService for ProgressServiceImpl {
    type AttachToProgressStreamStream = tokio_stream::wrappers::ReceiverStream<
        std::result::Result<AttachToProgressStreamResponse, tonic::Status>,
    >;

    async fn attach_to_progress_stream(
        &self,
        request: tonic::Request<AttachToProgressStreamRequest>,
    ) -> std::result::Result<tonic::Response<Self::AttachToProgressStreamStream>, tonic::Status>
    {
        let session_id = request.into_inner().session_id;
        todo!()
    }
}
