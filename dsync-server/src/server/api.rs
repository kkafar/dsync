use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use super::database::models::{PeerAddrV4Row, PeerServerBaseInfoRow};
use crate::utils;

use dsync_proto::cli::client_api_server::ClientApi;
use dsync_proto::cli::{
    self, DiscoverHostsRequest, DiscoverHostsResponse, HostDescription, ListHostsRequest,
    ListHostsResponse,
};
use dsync_proto::p2p::peer_service_client::PeerServiceClient;
use dsync_proto::p2p::{HelloThereRequest, ServerInfo};
use tonic::{Request, Response, Status};

use super::global_context::GlobalContext;

pub struct ClientApiImpl {
    ctx: Arc<GlobalContext>,
}

impl ClientApiImpl {
    pub fn new(ctx: Arc<GlobalContext>) -> Self {
        Self { ctx }
    }
}

#[tonic::async_trait]
impl ClientApi for ClientApiImpl {
    async fn list_hosts(
        &self,
        _request: Request<ListHostsRequest>,
    ) -> Result<Response<ListHostsResponse>, Status> {
        log::info!("Received ListHostsRequest");

        let serial_responses = self.host_discovery_impl().await.unwrap();

        let host_descriptions: Vec<HostDescription> = serial_responses
            .into_iter()
            .map(|sinfo| HostDescription {
                ipv4_addr: sinfo.address,
            })
            .collect();

        return Ok(Response::new(ListHostsResponse { host_descriptions }));
    }

    async fn discover_hosts(
        &self,
        _request: Request<DiscoverHostsRequest>,
    ) -> Result<Response<DiscoverHostsResponse>, Status> {
        log::info!("Received DiscoverHostsRequest");

        let discovered_servers_info = self.host_discovery_impl().await.unwrap();

        return Ok(Response::new(DiscoverHostsResponse {
            server_info: discovered_servers_info
                .into_iter()
                .map(|info| cli::ServerInfo {
                    uuid: info.uuid,
                    name: info.name,
                    hostname: info.hostname,
                    address: info.address,
                })
                .collect(),
        }));
    }
}

impl ClientApiImpl {
    async fn check_hello(&self, ipv4_addr: &str) -> Option<ServerInfo> {
        // Try to connect with the host
        let remote_service_socket = format!("http://{ipv4_addr}:{}", self.ctx.run_config.port);
        let mut client_conn = match PeerServiceClient::connect(remote_service_socket.clone()).await
        {
            Ok(conn) => conn,
            Err(error) => {
                log::debug!("Failed to connect with {remote_service_socket} with error: {error}");
                return None;
            }
        };

        let server_info = self.ctx.db_proxy.fetch_local_server_info().await.ok()?;

        let request = tonic::Request::new(HelloThereRequest {
            server_info: Some(ServerInfo {
                uuid: server_info.uuid,
                name: server_info.name,
                hostname: server_info.hostname,
                address: "".to_owned(),
            }),
        });

        let response = client_conn.hello_there(request).await.ok()?.into_inner();

        if response.server_info.is_none() {
            log::warn!(target: "pslog", "Invalid response from peer, server info must not be none");
            return None;
        }

        let mut remote_server_info = response.server_info.unwrap();

        assert!(
            remote_server_info.address == "".to_owned(),
            "Unexpected payload, expected empty address"
        );

        // Fill up the address, because we actually have this information here
        remote_server_info.address = remote_service_socket;

        Some(remote_server_info)
    }

    async fn host_discovery_impl(&self) -> Result<Vec<ServerInfo>, Status> {
        // TODO: this could be done once, on server start.
        if !utils::check_binary_exists("nmap") {
            return Err(tonic::Status::internal("Missing binary: nmap"));
        }

        if !utils::check_binary_exists("arp") {
            return Err(tonic::Status::internal("Missing binary: arp"));
        }

        let Some(ipv4_addrs) = utils::discover_hosts_in_local_network() else {
            return Err(tonic::Status::internal(
                "Failed to find hosts in local network",
            ));
        };

        let mut serial_responses: Vec<ServerInfo> = Vec::new();

        // This could be definitely improved, however it's fine for now.
        for addr in ipv4_addrs.iter() {
            match self.check_hello(&addr.to_string()).await {
                Some(server_info) => serial_responses.push(server_info),
                None => {
                    log::trace!(target: "pslog", "Have not found deamon at {addr}");
                }
            }
        }

        let discovery_time: i64 = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
            .try_into()
            .unwrap();

        // Cache discovered hosts locally
        {
            let peer_base_info: Vec<PeerServerBaseInfoRow> = serial_responses
                .iter()
                .map(|info| PeerServerBaseInfoRow {
                    // TODO: Could use only references in this struct, avoiding all the copies
                    uuid: info.uuid.clone(),
                    name: info.name.clone(),
                    hostname: info.hostname.clone(),
                })
                .collect();

            self.ctx
                .db_proxy
                .save_peer_server_base_info(&peer_base_info)
                .await;
        }

        {
            let peer_addr_info: Vec<PeerAddrV4Row> = serial_responses
                .iter()
                .map(|info| PeerAddrV4Row {
                    uuid: info.uuid.clone(),
                    ipv4_addr: info.address.clone(),
                    discovery_time,
                })
                .collect();

            self.ctx
                .db_proxy
                .save_peer_server_addr_info(&peer_addr_info)
                .await;
        }

        Ok(serial_responses)
    }
}
