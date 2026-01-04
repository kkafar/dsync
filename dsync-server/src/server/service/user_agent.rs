use std::net::{Ipv4Addr, SocketAddrV4};
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;

use crate::server::config::defaults;
use crate::server::database::error::{DeleteLocalGroupError, FileAddError, SaveLocalGroupError};
use crate::server::database::models::{FilesLocalFragmentInsert, HostsRow};
use crate::server::service::tools;

use anyhow::Context;
use dsync_proto::model::common::LocalFileDescription;
use dsync_proto::model::server::HostInfo;
use dsync_proto::services::user_agent::{
    HostAddRequest, HostAddResponse, HostRemoveRequest, HostRemoveResponse, HostSpec, host_spec,
};
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
use dsync_shared::DEFAULT_SERVER_PORT;
use dsync_shared::conn::{ChannelFactory, create_server_url};
use dsync_shared::model::FileSourceWrapper;
use tonic::transport::Uri;
use tonic::{Request, Response, Status};

use crate::server::context::ServerContext;

pub struct UserAgentServiceImpl {
    ctx: Arc<ServerContext>,
}

impl UserAgentServiceImpl {
    pub fn new(ctx: Arc<ServerContext>) -> Self {
        Self { ctx }
    }
}

#[tonic::async_trait]
impl UserAgentService for UserAgentServiceImpl {
    async fn file_add(
        &self,
        request: Request<FileAddRequest>,
    ) -> Result<Response<FileAddResponse>, Status> {
        let req_payload = request.into_inner();

        log::info!("Received FileAdd");
        log::debug!("Payload: {req_payload:?}");

        if req_payload.file_paths.is_empty() {
            return Ok(Response::new(FileAddResponse {}));
        }

        {
            let invalid_paths = req_payload
                .file_paths
                .iter()
                .filter_map(|path_str| {
                    let path = PathBuf::from(path_str);
                    if !path.is_absolute() || !path.is_file() {
                        Some(path_str)
                    } else {
                        None
                    }
                })
                .collect::<Vec<_>>();

            if invalid_paths.len() > 0 {
                return Err(Status::invalid_argument(format!(
                    "Some file paths are invalid (not absolute or not a file): {:?}",
                    invalid_paths
                )));
            }
        }

        // 2 - compute file hash
        // TODO: TBH - do we need these hashes? We don't use them anywhere yet + they're not kept
        // up to date in case the file changes (we need to observe inodes).
        // This could potentially be done in multithreaded manner in case of many paths.

        let hashes: Vec<String> = req_payload
            .file_paths
            .iter()
            .filter_map(
                |path_str| match tools::file::compute_sha1_hash_from_file(path_str, None) {
                    Ok(hash) => Some(hash),
                    Err(_) => None,
                },
            )
            .collect();

        if hashes.len() != req_payload.file_paths.len() {
            return Err(Status::internal("Failed to compute some hashes"));
        }

        let fragments = req_payload
            .file_paths
            .into_iter()
            .zip(hashes.into_iter())
            .map(|(file_path, hash)| FilesLocalFragmentInsert {
                file_path,
                hash_sha1: hash,
            });

        // 3 - save file to the db
        let result = self
            .ctx
            .repo
            .save_local_files(&fragments.collect::<Vec<FilesLocalFragmentInsert>>())
            .await;

        match result {
            Ok(_) => Ok(tonic::Response::new(FileAddResponse {})),
            Err(err) => match err {
                FileAddError::AlreadyExists { file_name } => Err(Status::already_exists(format!(
                    "File: {file_name} is already tracked"
                ))),
                FileAddError::OtherDatabaseError { kind } => Err(Status::failed_precondition(
                    format!("Some other database error: {kind:?}"),
                )),
                FileAddError::Other(err) => Err(Status::unknown(format!("Unknown error: {err}"))),
            },
        }
    }

    async fn file_remove(
        &self,
        request: Request<FileRemoveRequest>,
    ) -> Result<Response<FileRemoveResponse>, Status> {
        let payload = request.into_inner();

        if payload.group_id.is_some() {
            return Err(Status::unimplemented(
                "Removing files from groups is not yet supported",
            ));
        }

        match self.ctx.repo.delete_local_file(&payload.file_path).await {
            Ok(_) => Ok(Response::new(FileRemoveResponse {})),
            Err(err) => {
                let message = format!("Error while attampting to remove a file: {err}");
                log::warn!("{message}");
                Err(Status::internal(message))
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

        match self.ctx.repo.fetch_local_files().await {
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

        log::info!("Received file_copy request");
        log::debug!("{:?}", &request);

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
                .repo
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

        let host_dst_ipv4_addr = Ipv4Addr::from_str(&host_src_info.ipv4_addr).map_err(|err| {
            Status::failed_precondition(format!(
                "failed-to-parse-dst-address: {} with error: {err}",
                &host_src_info.ipv4_addr
            ))
        })?;
        let host_dst_addr = SocketAddrV4::new(host_dst_ipv4_addr, defaults::SERVER_PORT);

        let channel = ChannelFactory::channel_with_timeout(
            create_server_url(host_dst_addr),
            Duration::from_secs(5),
        )
        .await?;
        let mut transfer_client = FileTransferServiceClient::new(channel);

        let _transfer_response = match transfer_client.transfer_submit(transfer_request).await {
            Ok(response) => response.into_inner(),
            Err(status) => {
                log::warn!("Dest host rejected TransferInitRequest: {}", status);
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

        let servers_info = match self.ctx.repo.fetch_hosts().await {
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

    async fn host_add(
        &self,
        request: Request<HostAddRequest>,
    ) -> Result<Response<HostAddResponse>, Status> {
        // FIXME: This function has few problems:
        // 1. We do not check whether requested server is exactly the same one
        // as instance serving the request.
        // 2. We need to actually store the port together with the address...
        // Even if autodiscovery is not able to find remotes running on different
        // ports than default, now it is possible to manually add them.

        let payload = request.into_inner();

        let host_ipv4 = match Ipv4Addr::from_str(&payload.ipv4_addr) {
            Ok(addr) => addr,
            Err(err) => {
                return Err(Status::invalid_argument(format!(
                    "failed-to-parse-host-ipv4: {err}"
                )));
            }
        };

        let port = if let Some(port) = payload.port {
            match u16::try_from(port) {
                Ok(port) => port,
                Err(err) => {
                    return Err(Status::invalid_argument(format!(
                        "failed-to-parse-host-port: {err}"
                    )));
                }
            }
        } else {
            DEFAULT_SERVER_PORT
        };

        let host_addr = SocketAddrV4::new(host_ipv4, port);

        let mut client = HostDiscoveryServiceClient::new(
            ChannelFactory::channel_with_timeout(
                create_server_url(host_addr),
                Duration::from_secs(5),
            )
            .await?,
        );

        let localhost_info = self.ctx.repo.fetch_local_server_info().await?;

        let result = client
            .hello_there(HelloThereRequest {
                host_info: Some(HostInfo {
                    uuid: localhost_info.uuid,
                    name: localhost_info.name,
                    hostname: localhost_info.hostname,
                    address: String::from(""),
                }),
            })
            .await;

        let response_payload = match result {
            Ok(response) => response.into_inner(),
            Err(err) => {
                return Err(Status::unavailable(format!(
                    "received-error-response: {err}"
                )));
            }
        };

        let host_info = response_payload.host_info.ok_or_else(|| {
            Status::aborted("Remote host response had missing required field - host_info")
        })?;

        let host_row = HostsRow {
            uuid: host_info.uuid.clone(),
            name: host_info.name.clone(),
            hostname: host_info.hostname.clone(),
            is_remote: true,
            ipv4_addr: host_addr.ip().to_string(),
            discovery_time: tools::time::get_current_timestamp(),
        };

        let _ = self.ctx.repo.insert_hosts(&[host_row]).await;
        Ok(Response::new(HostAddResponse {
            host_info: Some(host_info),
        }))
    }

    async fn host_remove(
        &self,
        request: Request<HostRemoveRequest>,
    ) -> Result<Response<HostRemoveResponse>, Status> {
        let payload = request.into_inner();
        let host_spec = payload
            .host_spec
            .ok_or_else(|| Status::invalid_argument("missing-host-spec"))?;

        let host_info = self
            .resolve_host_info_by_spec(&host_spec)
            .await
            .map_err(|err| {
                Status::unknown(format!(
                    "Failed to fetch requested remote info, likely it doesn't exist - {err}"
                ))
            })?;

        if !host_info.is_remote {
            return Err(Status::invalid_argument(
                "Can not remove current server instance",
            ));
        }

        let _ = self.ctx.repo.delete_host_with_uuid(&host_info.uuid).await;

        Ok(Response::new(HostRemoveResponse {}))
    }

    async fn group_create(
        &self,
        request: Request<GroupCreateRequest>,
    ) -> Result<Response<GroupCreateResponse>, Status> {
        log::info!("Received GroupCreateRequest");

        let payload = request.into_inner();
        match self.ctx.repo.save_local_group(&payload.group_id).await {
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
        let payload = request.into_inner();

        match self.ctx.repo.delete_group_by_name(&payload.group_id).await {
            Ok(_) => Ok(tonic::Response::new(GroupDeleteResponse {})),
            Err(error) => match error {
                DeleteLocalGroupError::DoesNotExist => Err(tonic::Status::invalid_argument(
                    format!("Group `{}` does not exist", &payload.group_id),
                )),
                DeleteLocalGroupError::Other(error) => {
                    Err(tonic::Status::internal(error.to_string()))
                }
            },
        }
    }

    async fn group_list(
        &self,
        _request: Request<GroupListRequest>,
    ) -> Result<Response<GroupListResponse>, Status> {
        let group_list = match self.ctx.repo.fetch_local_groups().await {
            Ok(groups) => groups,
            Err(err) => return Err(tonic::Status::internal(format!("Failed with error: {err}"))),
        };

        Ok(tonic::Response::new(GroupListResponse { group_list }))
    }
}

impl UserAgentServiceImpl {
    async fn check_hello(&self, ipv4_addr: &str) -> Option<HostInfo> {
        let remote_ip_addr = Ipv4Addr::from_str(ipv4_addr)
            .with_context(|| {
                format!(
                    "Failed to resolve host ipv4 addr from string: {}",
                    ipv4_addr
                )
            })
            .ok()?;

        // Try to connect with the host
        let remote_service_socket = SocketAddrV4::new(remote_ip_addr, defaults::SERVER_PORT);
        let remote_service_uri = Uri::builder()
            .scheme("http")
            .authority(remote_service_socket.clone().to_string())
            .path_and_query("/")
            .build()
            .with_context(|| {
                format!(
                    "Failed to build remote service uri from socket addr: {}",
                    &remote_service_socket
                )
            })
            .ok()?;

        let Ok(mut endpoint) = tonic::transport::Endpoint::new(remote_service_uri) else {
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

        let server_info = self.ctx.repo.fetch_local_server_info().await.ok()?;

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
            remote_server_info.address.is_empty(),
            "Unexpected payload, expected empty address"
        );

        // Fill up the address, because we actually have this information here
        remote_server_info.address = remote_service_socket.ip().to_string();

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
        let peer_base_info: Vec<HostsRow> = serial_responses
            .iter()
            // TODO: Could use only references in this struct, avoiding all the copies
            .map(|info| HostsRow::from_host_info(&info, true, Some(discovery_time)))
            .collect();

        let _ = self.ctx.repo.insert_hosts(&peer_base_info).await;

        Ok(serial_responses)
    }

    async fn resolve_host_info_by_spec(&self, host_spec: &HostSpec) -> anyhow::Result<HostsRow> {
        match host_spec.kind.as_ref().expect("Required field") {
            host_spec::Kind::LocalHost(_) => {
                let local_host_info = self.ctx.repo.fetch_local_server_info().await?;
                Ok(local_host_info)
            }
            host_spec::Kind::Name(name) => {
                let host_info = self.ctx.repo.fetch_host_by_name(name).await?;
                Ok(host_info)
            }
            host_spec::Kind::LocalId(id) => {
                let host_info = self.ctx.repo.fetch_host_by_local_id(*id).await?;
                Ok(host_info)
            }
        }
    }
}
