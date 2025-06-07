mod command_impl;

use std::path::PathBuf;

use clap::Subcommand;

type GroupId = String;
type RemoteId = String;

#[derive(Subcommand)]
pub(crate) enum Commands {
    /// Manage local & remote hosts (peers)
    #[command(subcommand)]
    Host(HostCommand),

    /// Operations on files
    #[command(subcommand)]
    File(FileCommand),

    /// Manage & display file groups.
    #[command(subcommand)]
    Group(GroupCommand),

    ListHosts,
    DiscoverHosts,
    AddFile {
        file_path: PathBuf,
    },
    ListLocalFiles,
}

#[derive(Subcommand)]
pub(crate) enum HostCommand {
    List {
        /// Run host discovery before returning the result
        #[arg(short = 'd', long = "discover")]
        discover: Option<bool>,
    },
    Discover,
}

#[derive(Subcommand)]
pub(crate) enum FileCommand {
    /// Add file to the track list. When added,
    /// this file can be synced by any remote peer.
    Add {
        /// Path to a local file
        path: PathBuf,

        /// Add the file to given group
        #[arg(short = 'g', long = "group")]
        group_id: Option<GroupId>,
    },
    /// Remove given from from the tracked-files list.
    /// When removed this file won't be synced between peers
    /// anymore.
    Remove {
        /// Path to a local file
        path: PathBuf,

        /// Remove file from the given group
        #[arg(short = 'g', long = "group")]
        group_id: Option<GroupId>,
    },
    /// List files. Specify options to select source of files.
    List {
        /// Identifier of remote peer.
        remote_id: Option<RemoteId>,

        /// Group id. When specified the results will be limited
        /// to paths belonging to the group.
        #[arg(short = 'g', long = "group")]
        group_id: Option<GroupId>,
    },
    Sync,
    Unsync,
}

#[derive(Subcommand)]
pub(crate) enum GroupCommand {
    /// Create group with given ID. Groups must be unique across local host.
    /// They might however repeat across different peers.
    Create { group_id: GroupId },

    /// Delete group with given ID.
    Delete { group_id: GroupId },

    /// List groups from local host or a remote peer.
    List {
        /// Identifier of remote peer.
        remote_id: Option<RemoteId>,
    },
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
