mod file;
mod group;
mod host;
mod server;

use crate::command;
use crate::config::Config;

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

    /// Manage & configure server instance
    #[command(subcommand)]
    Server(server::ServerCommand),
}

impl Commands {
    pub(crate) async fn handle(self, cfg: &Config) -> anyhow::Result<()> {
        match self {
            Self::Host(subcmd) => match subcmd {
                host::HostCommand::List { discover: _ } => command::host::host_list(cfg).await,
                host::HostCommand::Discover => command::host::host_discover(cfg).await,
                host::HostCommand::Add { ipv4_addr } => {
                    command::host::host_add(cfg, ipv4_addr).await
                }
                host::HostCommand::Remove { host_spec } => {
                    command::host::host_remove(cfg, host_spec).await
                }
            },
            Self::File(subcmd) => match subcmd {
                file::FileCommand::Add { paths, group_id } => {
                    command::file::file_add(cfg, paths, group_id).await
                }
                file::FileCommand::Remove { path, group_id: _ } => {
                    command::file::file_remove(cfg, &path).await
                }
                file::FileCommand::List {
                    remote_id,
                    group_id,
                } => command::file::file_list(cfg, remote_id, group_id).await,
                file::FileCommand::Copy {
                    source,
                    destination,
                } => command::file::file_copy(cfg, source, destination).await,
                file::FileCommand::Sync => command::file::file_sync(cfg).await,
                file::FileCommand::Unsync => command::file::file_unsync(cfg).await,
            },
            Self::Group(subcmd) => match subcmd {
                group::GroupCommand::Create { group_id } => {
                    command::group::group_create(cfg, group_id).await
                }
                group::GroupCommand::Delete { group_id } => {
                    command::group::group_delete(cfg, group_id).await
                }
                group::GroupCommand::List { remote_id } => {
                    command::group::group_list(cfg, remote_id).await
                }
            },
            Self::Server(subcmd) => match subcmd {
                server::ServerCommand::Shutdown {} => command::server::server_shutdown(cfg).await,
            },
        }
    }
}
