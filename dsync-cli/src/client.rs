use std::net::{Ipv4Addr, SocketAddrV4};

use dsync_proto::services::user_agent::user_agent_service_client::UserAgentServiceClient;
use dsync_shared::DEFAULT_SERVER_PORT;
use tonic::transport::Channel;

pub fn local_server_url() -> tonic::transport::Uri {
    let authority = SocketAddrV4::new(Ipv4Addr::LOCALHOST, DEFAULT_SERVER_PORT);
    tonic::transport::Uri::builder()
        .scheme("http")
        .authority(authority.to_string())
        .path_and_query("/")
        .build()
        .unwrap_or_else(|error| {
            panic!("Failed to build local server URI: {}", error);
        })
}

pub async fn user_agent_service_conn_factory()
-> Result<UserAgentServiceClient<Channel>, tonic::transport::Error> {
    UserAgentServiceClient::connect(local_server_url()).await
}
