pub(crate) mod file;
pub(crate) mod group;
pub(crate) mod host;
pub(crate) mod model;
pub(self) mod utils;

use std::path::PathBuf;

use crate::cli::command::Commands;

impl Commands {
    pub(super) async fn handle_list_hosts(self) -> anyhow::Result<()> {
        host::host_list().await
    }

    pub(super) async fn handle_discover_hosts(self) -> anyhow::Result<()> {
        host::host_discover().await
    }

    // This captures self by reference to avoid problems with dispatching,
    // since the command itself owns the file_path.
    pub(super) async fn handle_add_file(&self, file_path: PathBuf) -> anyhow::Result<()> {
        file::file_add(file_path).await
    }

    pub(super) async fn handle_list_local_files(&self) -> anyhow::Result<()> {
        file::file_list(None, None).await
    }
}
