mod file;
mod group;
mod host;

use std::path::PathBuf;

use clap::Subcommand;

pub(crate) type GroupId = String;
pub(crate) type RemoteId = String;

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

    ListHosts,

    AddFile {
        file_path: PathBuf,
    },
    ListLocalFiles,
    DiscoverHosts,
}
