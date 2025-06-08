use crate::cli::command::model::{GroupId, RemoteId};

pub(crate) async fn group_create(_group_id: GroupId) -> anyhow::Result<()> {
    unimplemented!()
}

pub(crate) async fn group_delete(_group_id: GroupId) -> anyhow::Result<()> {
    unimplemented!()
}

pub(crate) async fn group_list(_remote_id: Option<RemoteId>) -> anyhow::Result<()> {
    unimplemented!()
}
