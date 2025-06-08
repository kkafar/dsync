use clap::Subcommand;

use std::path::PathBuf;

use crate::command::model::{GroupId, RemoteId};

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
