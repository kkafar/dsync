use clap::Subcommand;

use std::path::PathBuf;

use crate::command::model::{GroupId, RemoteId};

#[derive(Subcommand, Debug)]
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
    /// Copy files between destinations.
    Copy {
        /// Source file path from the local host.
        source: String,
        /// Destination specification given in appropriat format.
        /// [HOST_IDENTIFIER@]FILE_PATH. If HOST_IDENTIFIER is ommited,
        /// the path is treated as local to the origin host.
        destination: String,
    },
    Sync,
    Unsync,
}
