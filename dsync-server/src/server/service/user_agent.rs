use std::path::PathBuf;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::server::database::SaveLocalGroupError;
use crate::server::database::models::{LocalFilesWoIdRow, PeerAddrV4Row, PeerServerBaseInfoRow};
use crate::server::util;
use crate::utils;

use dsync_proto::server::HelloThereRequest;
use dsync_proto::server::peer_service_client::PeerServiceClient;
use dsync_proto::shared;
use dsync_proto::user_agent::user_agent_service_server::UserAgentService;
use dsync_proto::user_agent::{
    FileAddRequest, FileAddResponse, FileListRequest, FileListResponse, FileRemoveRequest,
    FileRemoveResponse, GroupCreateRequest, GroupCreateResponse, GroupDeleteRequest,
    GroupDeleteResponse, GroupListRequest, GroupListResponse, HostDiscoverRequest,
    HostDiscoverResponse, HostListRequest, HostListResponse, LocalFileDescription,
};
use tonic::{Request, Response, Status};

use crate::server::global_context::GlobalContext;

pub struct UserAgentServiceImpl {
    ctx: Arc<GlobalContext>,
}

impl UserAgentServiceImpl {
    pub fn new(ctx: Arc<GlobalContext>) -> Self {
        Self { ctx }
    }
}

#[tonic::async_trait]
impl UserAgentService for UserAgentServiceImpl {
    async fn file_add(
        &self,
        request: Request<FileAddRequest>,
    ) -> Result<Response<FileAddResponse>, Status> {
        let request_payload = request.into_inner();

        log::info!("Received FileAdd");
        log::debug!("Payload: {request_payload:?}");

        let file_path_string = request_payload.file_path;
        let file_path = PathBuf::from(file_path_string);

        if !file_path.is_absolute() {
            return Err(tonic::Status::invalid_argument(
                "File path must be absolute",
            ));
        }

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

        return Ok(tonic::Response::new(FileAddResponse {}));
    }

    async fn file_remove(
        &self,
        request: Request<FileRemoveRequest>,
    ) -> Result<Response<FileRemoveResponse>, Status> {
        let payload = request.into_inner();

        if payload.group_id.is_some() {
            return Err(tonic::Status::unimplemented(
                "Removing files from groups is not yet supported",
            ));
        }

        match self
            .ctx
            .db_proxy
            .delete_local_file(&payload.file_path)
            .await
        {
            Ok(_) => Ok(Response::new(FileRemoveResponse {})),
            Err(err) => {
                let message = format!("Error while attampting to remove a file: {err}");
                log::warn!("{message}");
                Err(tonic::Status::internal(message))
            }
        }
    }

    async fn file_list(
        &self,
        request: Request<FileListRequest>,
    ) -> Result<Response<FileListResponse>, Status> {
        let request_payload = request.into_inner();

        log::info!("Received FileList");
        log::debug!("Payload: {request_payload:?}");

        match self.ctx.db_proxy.fetch_local_files().await {
            Ok(local_files) => Ok(tonic::Response::new(FileListResponse {
                file_list: local_files
                    .into_iter()
                    .map(|f| LocalFileDescription {
                        local_id: f.id.to_string(),
                        file_path: f.file_path,
                        hash_sha1: f.hash_sha1,
                    })
                    .collect(),
            })),
            Err(err) => {
                let message = format!("Failed to fetch local files with err: {err}");
                log::warn!("{message}");
                Err(tonic::Status::internal(message))
            }
        }
    }

    async fn host_list(
        &self,
        request: Request<HostListRequest>,
    ) -> Result<Response<HostListResponse>, Status> {
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

        return Ok(Response::new(HostListResponse {
            host_list: servers_info,
        }));
    }

    async fn host_discover(
        &self,
        request: Request<HostDiscoverRequest>,
    ) -> Result<Response<HostDiscoverResponse>, Status> {
        log::info!("Received DiscoverHostsRequest");

        let discovered_servers_info = self.host_discovery_impl().await.unwrap();

        return Ok(Response::new(HostDiscoverResponse {
            host_list: discovered_servers_info,
        }));
    }

    async fn group_create(
        &self,
        request: Request<GroupCreateRequest>,
    ) -> Result<Response<GroupCreateResponse>, Status> {
        log::info!("Received GroupCreateRequest");

        let payload = request.into_inner();
        match self.ctx.db_proxy.save_local_group(&payload.group_id).await {
            Ok(_) => Ok(tonic::Response::new(GroupCreateResponse {})),
            Err(err) => {
                log::warn!("Failed to save local group with error: {err}");
                match err {
                    SaveLocalGroupError::AlreadyExists { group_id } => {
                        Err(tonic::Status::already_exists(format!(
                            "Group with id: \"{group_id}\" already exists"
                        )))
                    }
                    SaveLocalGroupError::Other => Err(tonic::Status::unknown(format!(
                        "Unknown error raised while saving local group: {err}",
                    ))),
                }
            }
        }
    }

    async fn group_delete(
        &self,
        request: Request<GroupDeleteRequest>,
    ) -> Result<Response<GroupDeleteResponse>, Status> {
        Err(tonic::Status::unimplemented("Not yet implemented"))
    }

    async fn group_list(
        &self,
        request: Request<GroupListRequest>,
    ) -> Result<Response<GroupListResponse>, Status> {
        let group_list = match self.ctx.db_proxy.fetch_local_groups().await {
            Ok(groups) => groups,
            Err(err) => return Err(tonic::Status::internal(format!("Failed with error: {err}"))),
        };

        Ok(tonic::Response::new(GroupListResponse { group_list }))
    }
}

impl UserAgentServiceImpl {
    async fn check_hello(&self, ipv4_addr: &str) -> Option<shared::ServerInfo> {
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
            server_info: Some(shared::ServerInfo {
                uuid: server_info.uuid,
                name: server_info.name,
                hostname: server_info.hostname,
                address: "".to_owned(),
            }),
        });

        let response = client_conn.hello_there(request).await.ok()?.into_inner();

        let Some(mut remote_server_info) = response.server_info else {
            log::warn!(target: "pslog", "Invalid response from peer, server info must not be none");
            return None;
        };

        assert!(
            remote_server_info.address == "".to_owned(),
            "Unexpected payload, expected empty address"
        );

        // Fill up the address, because we actually have this information here
        remote_server_info.address = remote_service_socket;

        Some(remote_server_info)
    }

    async fn host_discovery_impl(&self) -> Result<Vec<shared::ServerInfo>, Status> {
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

        let mut serial_responses: Vec<shared::ServerInfo> = Vec::new();

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
