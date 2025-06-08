pub(crate) mod file;
pub(crate) mod group;
pub(crate) mod host;
pub(crate) mod model;
pub(self) mod utils;

use std::path::PathBuf;

pub(super) async fn handle_list_hosts() -> anyhow::Result<()> {
    host::host_list().await
}

pub(super) async fn handle_discover_hosts() -> anyhow::Result<()> {
    host::host_discover().await
}

// This captures self by reference to avoid problems with dispatching,
// since the command itself owns the file_path.
pub(super) async fn handle_add_file(file_path: PathBuf) -> anyhow::Result<()> {
    file::file_add(file_path).await
}

pub(super) async fn handle_list_local_files() -> anyhow::Result<()> {
    file::file_list(None, None).await
}
