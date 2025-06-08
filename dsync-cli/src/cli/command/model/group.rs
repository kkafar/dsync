use clap::Subcommand;

use super::{GroupId, RemoteId};

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
