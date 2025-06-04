use std::path::PathBuf;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use super::database::models::{LocalFilesWoIdRow, PeerAddrV4Row, PeerServerBaseInfoRow};
use super::util;
use crate::utils;

use dsync_proto::cli::client_api_server::ClientApi;
use dsync_proto::cli::{
    self, AddFileRequest, AddFileResponse, DiscoverHostsRequest, DiscoverHostsResponse,
    ListHostsRequest, ListHostsResponse,
};
use dsync_proto::p2p::peer_service_client::PeerServiceClient;
use dsync_proto::p2p::{self, HelloThereRequest};
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
    async fn add_file(
        &self,
        request: Request<AddFileRequest>,
    ) -> Result<Response<AddFileResponse>, Status> {
        let request_payload = request.into_inner();

        log::info!("Received AddFileRequest");
        log::debug!("Payload: {request_payload:?}");

        if request_payload.file_path.len() != 1 {
            return Err(tonic::Status::invalid_argument(format!(
                "Expected exactly 1 file. More / less is not supported right now."
            )));
        }

        let file_path_string = request_payload
            .file_path
            .first()
            .expect("Single file in array asserted above");

        let file_path = PathBuf::from(file_path_string);

        if !file_path.is_absolute() {
            return Err(tonic::Status::invalid_argument(
                "File path must be absolute",
            ));
        }

        // let file_path = match PathBuf::from(file_path_string).canonicalize() {
        //     Ok(absolute_path) => absolute_path,
        //     Err(err) => {
        //         let message = format!(
        //             "Failed to turn path: {file_path_string} into an absolute path with err: {err}"
        //         );
        //         log::warn!("{message}");
        //         return Err(tonic::Status::invalid_argument(message));
        //     }
        // };

        // 1 - verify that the file exists on the host
        // directories are yet unsupported.
        if !file_path.is_file() {
            return Err(tonic::Status::invalid_argument(format!(
                "Not a file! File from request either does not exist or is not a regular file."
            )));
        }

        // 2 - compute file hash
        let hash = match util::compute_sha1_hash_from_file(&file_path, None) {
            Ok(hash) => hash,
            Err(err) => {
                log::warn!("Failed to compute hash for requested file with err: {err}");
                return Err(tonic::Status::internal(format!(
                    "Failed to compute hash for requested file with err: {err}"
                )));
            }
        };

        log::debug!("Hash computed for file: {hash}");

        let file_abs_path_string = match file_path.to_str() {
            Some(abs_path_str) => abs_path_str.to_string(),
            None => {
                log::warn!("Failed to convert absolute file path to string");
                return Err(tonic::Status::invalid_argument(
                    "Failed to convert absolute file path to string",
                ));
            }
        };

        // 3 - save file to the db
        self.ctx
            .db_proxy
            .save_local_file(LocalFilesWoIdRow {
                file_path: file_abs_path_string,
                hash_sha1: hash,
            })
            .await;

        return Ok(tonic::Response::new(AddFileResponse {}));
    }

    async fn list_hosts(
        &self,
        _request: Request<ListHostsRequest>,
    ) -> Result<Response<ListHostsResponse>, Status> {
        log::info!("Received ListHostsRequest");

        let servers_info = match self.ctx.db_proxy.fetch_peer_server_info().await {
            Ok(data) => data,
            Err(err) => {
                log::error!("Error while fetching peer server info: {err}");
                return Err(tonic::Status::internal(
                    "Error while fetching peer server info: {err}",
                ));
            }
        };

        return Ok(Response::new(ListHostsResponse {
            servers_info: servers_info
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

    async fn discover_hosts(
        &self,
        _request: Request<DiscoverHostsRequest>,
    ) -> Result<Response<DiscoverHostsResponse>, Status> {
        log::info!("Received DiscoverHostsRequest");

        let discovered_servers_info = self.host_discovery_impl().await.unwrap();

        return Ok(Response::new(DiscoverHostsResponse {
            servers_info: discovered_servers_info
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
    async fn check_hello(&self, ipv4_addr: &str) -> Option<p2p::ServerInfo> {
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
            server_info: Some(p2p::ServerInfo {
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

    async fn host_discovery_impl(&self) -> Result<Vec<p2p::ServerInfo>, Status> {
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

        let mut serial_responses: Vec<p2p::ServerInfo> = Vec::new();

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
