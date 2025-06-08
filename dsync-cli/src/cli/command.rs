mod file;
mod group;
mod host;

use crate::command;

use clap::Subcommand;

#[derive(Subcommand)]
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
                host::HostCommand::List { discover } => command::handle_list_hosts().await,
                host::HostCommand::Discover => command::handle_discover_hosts().await,
            },
            Self::File(subcmd) => match subcmd {
                file::FileCommand::Add { path, group_id } => command::handle_add_file(path).await,
                file::FileCommand::Remove { path, group_id } => todo!(),
                file::FileCommand::List {
                    remote_id,
                    group_id,
                } => command::handle_list_local_files().await,
                file::FileCommand::Sync => todo!(),
                file::FileCommand::Unsync => todo!(),
            },
            Self::Group(subcmd) => match subcmd {
                group::GroupCommand::Create { group_id } => todo!(),
                group::GroupCommand::Delete { group_id } => todo!(),
                group::GroupCommand::List { remote_id } => todo!(),
            },
            _ => {
                log::error!("Unimplemented command!");
                Err(anyhow::anyhow!("Unimplemented command"))
            }
        }
    }
}
