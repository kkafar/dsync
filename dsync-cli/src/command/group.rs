use anyhow::{Context, bail};
use dsync_proto::user_agent::{
    GroupCreateRequest, user_agent_service_client::UserAgentServiceClient,
};

use crate::command::model::LOOPBACK_ADDR_V4;

use super::model::{GroupId, RemoteId};

pub(crate) async fn group_create(group_id: GroupId) -> anyhow::Result<()> {
    let request = tonic::Request::new(GroupCreateRequest {
        group_id: group_id.clone(),
    });

    let mut client = UserAgentServiceClient::connect(LOOPBACK_ADDR_V4)
        .await
        .context("Failed to connect to server")?;

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
                return Err(status.into());
            }
            _ => {
                log::error!("Received ERROR response from server\n{status:?}");
                bail!("Received ERROR response from server: {status:?}")
            }
        },
    }
}

pub(crate) async fn group_delete(_group_id: GroupId) -> anyhow::Result<()> {
    todo!()
}

pub(crate) async fn group_list(_remote_id: Option<RemoteId>) -> anyhow::Result<()> {
    todo!()
}
