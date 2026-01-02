use anyhow::{Context, bail};
use dsync_proto::services::user_agent::{
    GroupCreateRequest, GroupDeleteRequest, GroupListRequest,
    user_agent_service_client::UserAgentServiceClient,
};
use dsync_shared::conn::{local_server_url, user_agent_service_conn_factory};

use crate::command::{model::LOOPBACK_ADDR_V4, utils};

use super::model::{GroupId, RemoteId};

pub(crate) async fn group_create(group_id: GroupId) -> anyhow::Result<()> {
    let request = tonic::Request::new(GroupCreateRequest {
        group_id: group_id.clone(),
    });

    let mut client = user_agent_service_conn_factory(local_server_url(None)).await?;

    log::info!("Sending request to server");
    log::debug!("{request:?}");

    let response = client.group_create(request).await;
    match response {
        Ok(payload) => {
            log::info!("Received OK response from server");
            log::debug!("{payload:?}");
            anyhow::Ok(())
        }
        Err(status) => match status.code() {
            tonic::Code::AlreadyExists => {
                log::warn!("Received ERROR response from server\n{status:?}");
                println!(
                    "Seems that group with id {group_id} already exists. Use it or create a group with different id."
                );
                Err(status.into())
            }
            _ => {
                log::error!("Received ERROR response from server\n{status:?}");
                bail!("Received ERROR response from server: {status:?}")
            }
        },
    }
}

pub(crate) async fn group_delete(group_id: GroupId) -> Result<(), anyhow::Error> {
    let request = tonic::Request::new(GroupDeleteRequest {
        group_id: group_id.clone(),
    });

    let mut client = user_agent_service_conn_factory(local_server_url(None)).await?;

    log::trace!("Sending group_delete request to server: {:?}", &request);

    let req_result = client.group_delete(request).await;

    log::trace!(
        "Received group_delete response from server: {:?}",
        &req_result
    );

    match req_result {
        Ok(_) => {
            println!("Group '{}' deleted successfully", &group_id);
        }
        Err(status) => match status.code() {
            tonic::Code::InvalidArgument | tonic::Code::NotFound => {
                println!("Group '{}' does not exist", &group_id);
            }
            _ => {
                println!(
                    "Failed to delete group '{}' with error: {}",
                    &group_id, &status
                );
            }
        },
    };

    Ok(())
}

pub(crate) async fn group_list(remote_id: Option<RemoteId>) -> anyhow::Result<()> {
    let request = tonic::Request::new(GroupListRequest { remote_id });

    let mut client = user_agent_service_conn_factory(local_server_url(None)).await?;

    log::info!("Sending request to server");
    log::debug!("{request:?}");

    let response = client.group_list(request).await;
    match response {
        Ok(payload) => {
            log::info!("Received OK response from server");
            log::debug!("{payload:?}");
            utils::print_local_group_info(&payload.into_inner().group_list);
            anyhow::Ok(())
        }
        Err(status) => {
            log::error!("Received ERROR response from server\n{status:?}");
            bail!("Received ERROR response from server: {status:?}")
        }
    }
}
