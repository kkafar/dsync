mod file;
mod group;
mod host;

use std::path::PathBuf;

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

    ListHosts,

    AddFile {
        file_path: PathBuf,
    },
    ListLocalFiles,
    DiscoverHosts,
}

impl Commands {
    pub(crate) async fn handle(self) -> anyhow::Result<()> {
        match self {
            Self::ListHosts => self.handle_list_hosts().await,
            Self::DiscoverHosts => self.handle_discover_hosts().await,
            Self::AddFile { ref file_path } => self.handle_add_file(file_path.clone()).await,
            Self::ListLocalFiles => self.handle_list_local_files().await,
            _ => {
                log::warn!("Unimplemented command!");
                Err(anyhow::anyhow!("Unimplemented command"))
            }
        }
    }
}
