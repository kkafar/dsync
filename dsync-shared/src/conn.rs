use std::{
    net::{Ipv4Addr, SocketAddrV4},
    time::Duration,
};

use dsync_proto::services::{
    server_control::server_control_service_client::ServerControlServiceClient,
    user_agent::user_agent_service_client::UserAgentServiceClient,
};
use tonic::transport::Channel;

use crate::DEFAULT_SERVER_PORT;

pub fn local_server_url(port: Option<u16>) -> tonic::transport::Uri {
    let authority = SocketAddrV4::new(Ipv4Addr::LOCALHOST, port.unwrap_or(DEFAULT_SERVER_PORT));
    create_server_url(authority)
}

pub fn create_server_url(socket: SocketAddrV4) -> tonic::transport::Uri {
    tonic::transport::Uri::builder()
        .scheme("http")
        .authority(socket.to_string())
        .path_and_query("/")
        .build()
        .unwrap_or_else(|error| {
            panic!("Failed to build local server URI: {}", error);
        })
}

pub async fn user_agent_service_conn_factory(
    uri: tonic::transport::Uri,
) -> Result<UserAgentServiceClient<Channel>, tonic::transport::Error> {
    UserAgentServiceClient::connect(uri).await
}

pub struct ChannelFactory {}

impl ChannelFactory {
    pub async fn channel(
        uri: tonic::transport::Uri,
    ) -> Result<tonic::transport::Channel, tonic::Status> {
        ChannelFactory::create_channel(uri, None).await
    }

    pub async fn channel_with_timeout(
        uri: tonic::transport::Uri,
        timeout: Duration,
    ) -> Result<tonic::transport::Channel, tonic::Status> {
        ChannelFactory::create_channel(uri, Some(timeout)).await
    }

    async fn create_channel(
        uri: tonic::transport::Uri,
        timeout: Option<Duration>,
    ) -> Result<tonic::transport::Channel, tonic::Status> {
        let mut endpoint = match tonic::transport::Endpoint::new(uri.clone()) {
            Ok(endpoint) => endpoint,
            Err(err) => {
                return Err(tonic::Status::unavailable(format!(
                    "Failed to create endpoint from uri {uri} with error: {err}"
                )));
            }
        };

        endpoint = if let Some(timeout) = timeout {
            endpoint.connect_timeout(timeout)
        } else {
            endpoint
        };

        endpoint.connect().await.map_err(|err| {
            tonic::Status::unavailable(format!(
                "Failed to connect to an endpoint: {endpoint:?} with err: {err}"
            ))
        })
    }
}

pub struct ServiceConnFactory {}

impl ServiceConnFactory {
    pub async fn local_user_agent_service(
        port: Option<u16>,
    ) -> Result<UserAgentServiceClient<Channel>, tonic::Status> {
        let port = port.unwrap_or(DEFAULT_SERVER_PORT);
        let channel = ChannelFactory::channel(local_server_url(Some(port))).await?;
        Ok(UserAgentServiceClient::new(channel))
    }

    pub async fn local_server_control_service(
        port: Option<u16>,
    ) -> Result<ServerControlServiceClient<Channel>, tonic::Status> {
        let port = port.unwrap_or(DEFAULT_SERVER_PORT);
        let channel = ChannelFactory::channel(local_server_url(Some(port))).await?;
        Ok(ServerControlServiceClient::new(channel))
    }
}
