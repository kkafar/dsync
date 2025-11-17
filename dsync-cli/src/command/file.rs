use std::path::Path;

use crate::command::utils;
use anyhow::Context;
use dsync_proto::services::user_agent::{
    FileAddRequest, FileCopyRequest, FileListRequest, FileRemoveRequest,
    user_agent_service_client::UserAgentServiceClient,
};
use dsync_shared::model::{FileSourceWrapper, parse_file_source_spec};

use crate::command::model::LOOPBACK_ADDR_V4;

use super::model::{GroupId, RemoteId};

pub(crate) async fn file_add(file_path: impl AsRef<Path>) -> anyhow::Result<()> {
    let file_path = file_path.as_ref();
    let file_path_abs = match file_path.canonicalize() {
        Ok(abs_path) => abs_path,
        Err(err) => {
            let message = format!(
                "Failed to turn file_path: ${file_path:?} into absolute path with err: {err}"
            );
            log::error!("{message}");
            return Err(anyhow::anyhow!(message));
        }
    };

    let path_as_string = file_path_abs
        .to_str()
        .context("Looks like the specified path is not a valid unicode")?
        .to_string();

    let request = tonic::Request::new(FileAddRequest {
        file_path: path_as_string,
        group_id: None,
    });

    let mut client = UserAgentServiceClient::connect(LOOPBACK_ADDR_V4)
        .await
        .context("Failed to connect to server")?;

    log::info!("Sending request to server");
    log::debug!("{request:?}");

    let response = client.file_add(request).await?;

    log::info!("Received response from server");
    log::debug!("{response:?}");

    anyhow::Ok(())
}

pub(crate) async fn file_remove(file_path: impl AsRef<Path>) -> anyhow::Result<()> {
    let file_path = file_path.as_ref();

    let file_path_abs = match file_path.canonicalize() {
        Ok(abs_path) => abs_path,
        Err(err) => {
            let message = format!(
                "Failed to turn file_path: ${file_path:?} into absolute path with err: {err}"
            );
            log::error!("{message}");
            return Err(anyhow::anyhow!(message));
        }
    };

    let path_as_string = file_path_abs
        .to_str()
        .context("Looks like the specified path is not a valid unicode")?
        .to_string();

    let request = tonic::Request::new(FileRemoveRequest {
        file_path: path_as_string,
        group_id: None,
    });

    let mut client = UserAgentServiceClient::connect(LOOPBACK_ADDR_V4)
        .await
        .context("Failed to connect to server")?;

    log::info!("Sending request to server");
    log::debug!("{request:?}");

    let response = client.file_remove(request).await?;

    log::info!("Received response from server");
    log::debug!("{response:?}");

    anyhow::Ok(())
}

pub(crate) async fn file_list(
    remote_id: Option<RemoteId>,
    group_id: Option<GroupId>,
) -> anyhow::Result<()> {
    if remote_id.is_some() {
        todo!("Remote file list is not supported yet");
    }

    if group_id.is_some() {
        todo!("Group file list is not supported yet");
    }

    let request = tonic::Request::new(FileListRequest {
        remote_id: None,
        group_id: None,
    });

    let mut client = UserAgentServiceClient::connect(LOOPBACK_ADDR_V4)
        .await
        .context("Failed to connect to server")?;

    log::info!("Sending request to server");
    log::debug!("{request:?}");

    let response = client.file_list(request).await?;

    log::info!("Received response from server");
    log::debug!("{response:?}");

    let payload = response.into_inner();

    utils::print_local_files_desc(&payload.file_list);

    anyhow::Ok(())
}

pub(crate) async fn file_copy(source: String, destination: String) -> anyhow::Result<()> {
    let mut file_source_src: FileSourceWrapper = parse_file_source_spec(&source)?.into();
    let mut file_source_dst: FileSourceWrapper = parse_file_source_spec(&destination)?.into();

    if let Some(is_localhost) = file_source_src.host_spec.try_is_localhost() {
        log::trace!("Source is_localhost: {}", is_localhost);
        if is_localhost {
            file_source_src.path_spec = file_source_src.path_spec.try_into_abs_path_spec()?;
        }
    };

    if let Some(is_localhost) = file_source_dst.host_spec.try_is_localhost() {
        log::trace!("Dest is_localhost: {}", is_localhost);
        if is_localhost {
            file_source_dst.path_spec = file_source_dst.path_spec.try_into_abs_path_spec()?;
        }
    };

    let request = tonic::Request::new(FileCopyRequest {
        src_spec: Some(file_source_src.into()),
        dst_spec: Some(file_source_dst.into()),
    });

    let mut client = UserAgentServiceClient::connect(LOOPBACK_ADDR_V4)
        .await
        .context("Failed to connect to server")?;

    log::info!("Sending request to server");
    log::debug!("{request:?}");

    let response = client.file_copy(request).await?;

    log::info!("Received response from server");
    log::debug!("{response:?}");

    anyhow::Ok(())
}

pub(crate) async fn file_sync() -> anyhow::Result<()> {
    todo!()
}

pub(crate) async fn file_unsync() -> anyhow::Result<()> {
    todo!()
}
