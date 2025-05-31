use dsync_proto::p2p::peer_service_server::PeerService;
use dsync_proto::p2p::{self, HelloThereRequest, HelloThereResponse};
use tonic::{Request, Response, Status};

#[derive(Debug, Default)]
pub struct PeerServiceImpl {}

#[tonic::async_trait]
impl PeerService for PeerServiceImpl {
    async fn hello_there(
        &self,
        request: Request<HelloThereRequest>,
    ) -> Result<Response<HelloThereResponse>, Status> {
        Ok(Response::new(HelloThereResponse {
            server_info: Some(p2p::ServerInfo {
                uuid: "uuid".to_string(),
                name: "name".to_string(),
                hostname: "hostname".to_string(),
                address: "address".to_string(),
            }),
        }))
    }
}
