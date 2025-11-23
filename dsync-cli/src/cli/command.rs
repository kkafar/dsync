mod file;
mod group;
mod host;

use crate::command;

use clap::Subcommand;

#[derive(Subcommand, Debug)]
pub(crate) enum Commands {
    /// Manage local & remote hosts (peers)
    #[command(subcommand)]
    Host(host::HostCommand),

    /// Operations on files
    #[command(subcommand)]
    File(file::FileCommand),

    /// Manage & display file groups.
    #[command(subcommand)]
    Group(group::GroupCommand),
}

impl Commands {
    pub(crate) async fn handle(self) -> anyhow::Result<()> {
        match self {
            Self::Host(subcmd) => match subcmd {
                host::HostCommand::List { discover: _ } => command::host::host_list().await,
                host::HostCommand::Discover => command::host::host_discover().await,
            },
            Self::File(subcmd) => match subcmd {
                file::FileCommand::Add { path, group_id: _ } => command::file::file_add(path).await,
                file::FileCommand::Remove { path, group_id: _ } => {
                    command::file::file_remove(&path).await
                }
                file::FileCommand::List {
                    remote_id,
                    group_id,
                } => command::file::file_list(remote_id, group_id).await,
                file::FileCommand::Copy {
                    source,
                    destination,
                } => command::file::file_copy(source, destination).await,
                file::FileCommand::Sync => command::file::file_sync().await,
                file::FileCommand::Unsync => command::file::file_unsync().await,
            },
            Self::Group(subcmd) => match subcmd {
                group::GroupCommand::Create { group_id } => {
                    command::group::group_create(group_id).await
                }
                group::GroupCommand::Delete { group_id } => {
                    command::group::group_delete(group_id).await
                }
                group::GroupCommand::List { remote_id } => {
                    command::group::group_list(remote_id).await
                }
            },
        }
    }
}
