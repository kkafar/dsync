use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use crate::server::config::defaults;
use crate::server::database::error::{FileAddError, SaveLocalGroupError};
use crate::server::database::models::{FilesLocalFragmentInsert, HostsRow};
use crate::server::service::tools;

use dsync_proto::model::common::LocalFileDescription;
use dsync_proto::model::server::HostInfo;
use dsync_proto::services::user_agent::file_source::{HostSpec, PathSpec};
use dsync_proto::services::{
    file_transfer::{
        TransferSubmitRequest, file_transfer_service_client::FileTransferServiceClient,
    },
    host_discovery::{
        HelloThereRequest, host_discovery_service_client::HostDiscoveryServiceClient,
    },
    user_agent::{
        FileAddRequest, FileAddResponse, FileCopyRequest, FileCopyResponse, FileListRequest,
        FileListResponse, FileRemoveRequest, FileRemoveResponse, GroupCreateRequest,
        GroupCreateResponse, GroupDeleteRequest, GroupDeleteResponse, GroupListRequest,
        GroupListResponse, HostDiscoverRequest, HostDiscoverResponse, HostListRequest,
        HostListResponse, user_agent_service_server::UserAgentService,
    },
};
use dsync_shared::model::FileSourceWrapper;
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
        let hash = match tools::file::compute_sha1_hash_from_file(&file_path, None) {
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
        let result = self
            .ctx
            .db_proxy
            .save_local_file(FilesLocalFragmentInsert {
                file_path: file_abs_path_string,
                hash_sha1: hash,
            })
            .await;

        match result {
            Ok(_) => Ok(tonic::Response::new(FileAddResponse {})),
            Err(err) => match err {
                FileAddError::AlreadyExists { file_name } => Err(tonic::Status::already_exists(
                    format!("File: {file_name} is already tracked"),
                )),
                FileAddError::OtherDatabaseError { kind } => {
                    Err(tonic::Status::failed_precondition(format!(
                        "Some other database error: {kind:?}"
                    )))
                }
                FileAddError::Other(err) => {
                    Err(tonic::Status::unknown(format!("Unknown error: {err}")))
                }
            },
        }
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

    async fn file_copy(
        &self,
        request: Request<FileCopyRequest>,
    ) -> Result<Response<FileCopyResponse>, Status> {
        let request = request.into_inner();

        if request.src_spec.is_none() {
            return Err(tonic::Status::invalid_argument("missing-src-spec"));
        }

        if request.dst_spec.is_none() {
            return Err(tonic::Status::invalid_argument("missing-dst-spec"));
        }

        let file_src_spec: FileSourceWrapper = request.src_spec.unwrap().into();
        let file_dst_spec: FileSourceWrapper = request.dst_spec.unwrap().into();

        let host_dst_info = match self
            .resolve_host_info_by_spec(&file_dst_spec.host_spec.0)
            .await
        {
            Ok(info) => info,
            Err(err) => {
                return Err(tonic::Status::invalid_argument(format!(
                    "failed-to-resolve-dest-host-spec: {err}"
                )));
            }
        };

        let host_src_info = match self
            .resolve_host_info_by_spec(&file_src_spec.host_spec.0)
            .await
        {
            Ok(info) => info,
            Err(err) => {
                return Err(tonic::Status::invalid_argument(format!(
                    "failed-to-resolve-src-host-spec: {err}"
                )));
            }
        };

        let host_local_info = if !host_src_info.is_remote {
            host_src_info.clone()
        } else if !host_dst_info.is_remote {
            host_dst_info.clone()
        } else {
            self.ctx
                .db_proxy
                .fetch_local_server_info()
                .await
                .map_err(|err| {
                    tonic::Status::internal(format!("failed-to-resolve-local-host-info: {err}"))
                })?
        };

        let transfer_request = TransferSubmitRequest {
            file_path_src: file_src_spec.path_spec.into_direct_string(),
            file_path_dst: file_dst_spec.path_spec.into_direct_string(),
            host_org_uuid: host_local_info.uuid,
            host_dst_uuid: host_dst_info.uuid,
        };

        let Ok(mut transfer_client) = FileTransferServiceClient::connect(
            tools::net::ipv4_into_connection_addr(&host_src_info.ipv4_addr, defaults::SERVER_PORT),
        )
        .await
        else {
            return Err(tonic::Status::unavailable("remote-server-unavailable"));
        };

        let _transfer_response = match transfer_client.transfer_submit(transfer_request).await {
            Ok(response) => response.into_inner(),
            Err(status) => {
                log::trace!("Dest host rejected TransferInitRequest");
                // TODO: Handle this correctly, instead of forwarding the status
                return Err(status);
            }
        };

        Ok(tonic::Response::new(FileCopyResponse {}))
    }

    async fn host_list(
        &self,
        _request: Request<HostListRequest>,
    ) -> Result<Response<HostListResponse>, Status> {
        log::info!("Received ListHostsRequest");

        let servers_info = match self.ctx.db_proxy.fetch_hosts().await {
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
        _request: Request<HostDiscoverRequest>,
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
        _request: Request<GroupDeleteRequest>,
    ) -> Result<Response<GroupDeleteResponse>, Status> {
        Err(tonic::Status::unimplemented("Not yet implemented"))
    }

    async fn group_list(
        &self,
        _request: Request<GroupListRequest>,
    ) -> Result<Response<GroupListResponse>, Status> {
        let group_list = match self.ctx.db_proxy.fetch_local_groups().await {
            Ok(groups) => groups,
            Err(err) => return Err(tonic::Status::internal(format!("Failed with error: {err}"))),
        };

        Ok(tonic::Response::new(GroupListResponse { group_list }))
    }
}

impl UserAgentServiceImpl {
    async fn check_hello(&self, ipv4_addr: &str) -> Option<HostInfo> {
        // Try to connect with the host
        let remote_service_socket = format!("http://{ipv4_addr}:{}", self.ctx.run_config.port);

        let Ok(mut endpoint) = tonic::transport::Endpoint::new(remote_service_socket.clone())
        else {
            log::warn!(target: "pslog", "Failed to create endpoint for addr: {}", &remote_service_socket);
            return None;
        };

        endpoint = endpoint.connect_timeout(Duration::from_secs(10));

        let channel = match endpoint.connect().await {
            Ok(ch) => ch,
            Err(error) => {
                log::warn!("Failed to connect with {remote_service_socket} with error: {error}");
                return None;
            }
        };

        let mut client_conn = HostDiscoveryServiceClient::new(channel);

        // let mut client_conn = HostDiscoveryServiceClient::connect(remote_service_socket.clone())
        //     .await
        //     .ok()?;

        let server_info = self.ctx.db_proxy.fetch_local_server_info().await.ok()?;

        let request = tonic::Request::new(HelloThereRequest {
            host_info: Some(HostInfo {
                uuid: server_info.uuid,
                name: server_info.name,
                hostname: server_info.hostname,
                address: "".to_owned(),
            }),
        });

        let response = client_conn.hello_there(request).await.ok()?.into_inner();

        let Some(mut remote_server_info) = response.host_info else {
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

    async fn host_discovery_impl(&self) -> Result<Vec<HostInfo>, Status> {
        // TODO: this could be done once, on server start.
        if !tools::file::check_binary_exists("nmap") {
            return Err(tonic::Status::internal("Missing binary: nmap"));
        }

        let Some(ipv4_addrs) = tools::net::addr_discovery::discover_hosts_in_local_network() else {
            return Err(tonic::Status::internal(
                "Failed to find hosts in local network",
            ));
        };

        log::debug!("Resolved addrs: {:?}", &ipv4_addrs);

        let mut serial_responses: Vec<HostInfo> = Vec::new();

        // This could be definitely improved, however it's fine for now.
        for addr in ipv4_addrs.iter() {
            match self.check_hello(&addr.to_string()).await {
                Some(server_info) => serial_responses.push(server_info),
                None => {
                    log::trace!(target: "pslog", "Have not found deamon at {addr}");
                }
            }
        }

        let discovery_time = tools::time::get_current_timestamp();

        // Cache discovered hosts locally
        {
            let peer_base_info: Vec<HostsRow> = serial_responses
                .iter()
                .map(|info| HostsRow {
                    // TODO: Could use only references in this struct, avoiding all the copies
                    uuid: info.uuid.clone(),
                    name: info.name.clone(),
                    hostname: info.hostname.clone(),
                    is_remote: true,
                    ipv4_addr: info.address.clone(),
                    discovery_time,
                })
                .collect();

            self.ctx.db_proxy.insert_hosts(&peer_base_info).await;
        }

        Ok(serial_responses)
    }

    async fn resolve_host_info_by_spec(&self, host_spec: &HostSpec) -> anyhow::Result<HostsRow> {
        match host_spec {
            HostSpec::LocalHost(_) => {
                let local_host_info = self.ctx.db_proxy.fetch_local_server_info().await?;
                Ok(local_host_info)
            }
            HostSpec::Name(name) => {
                let host_info = self.ctx.db_proxy.fetch_host_by_name(name).await?;
                Ok(host_info)
            }
            HostSpec::LocalId(id) => {
                let host_info = self.ctx.db_proxy.fetch_host_by_local_id(*id).await?;
                Ok(host_info)
            }
        }
    }
}
