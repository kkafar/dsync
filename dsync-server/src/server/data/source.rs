mod sqlite;

use dsync_proto::model::server::{GroupInfo, HostInfo};
pub use sqlite::SqliteDataSource;

use crate::server::database::{
    error::{DeleteLocalGroupError, FileAddError, LocalServerBaseInfoError, SaveLocalGroupError},
    models::{FilesLocalFragmentInsert, FilesLocalRow, HostsRow},
};

/// The data source interface that `MainRepository` depends on.
/// This mirrors the `DataRepository` methods but is meant to be implemented
/// by concrete data source types (e.g., `SqliteLocalMainDataSource`).
#[async_trait::async_trait]
pub(crate) trait DataSource: Send + Sync {
    // Hosts (local and remote)
    async fn fetch_local_server_info(&self) -> Result<HostsRow, LocalServerBaseInfoError>;
    async fn insert_hosts(&self, hosts_rows: &[HostsRow]) -> anyhow::Result<()>;
    async fn fetch_hosts(&self) -> anyhow::Result<Vec<HostInfo>>;
    async fn fetch_host_by_uuid(&self, uuid: &str) -> anyhow::Result<HostsRow>;
    async fn fetch_host_by_name(&self, name: &str) -> anyhow::Result<HostsRow>;
    #[allow(unused)]
    async fn fetch_host_by_local_id(&self, local_id: i32) -> anyhow::Result<HostsRow>;
    async fn delete_host_with_uuid(&self, host_uuid: &str) -> anyhow::Result<()>;

    // Files (local)
    async fn save_local_files(
        &self,
        local_files: &[FilesLocalFragmentInsert],
    ) -> Result<(), FileAddError>;
    async fn fetch_local_files(&self) -> anyhow::Result<Vec<FilesLocalRow>>;
    async fn delete_local_file(&self, file_path: &str) -> anyhow::Result<usize>;

    // Groups (local)
    async fn save_local_group(&self, group_id: &str) -> Result<usize, SaveLocalGroupError>;
    async fn delete_group_by_name(&self, group_name: &str) -> Result<(), DeleteLocalGroupError>;
    async fn fetch_local_groups(&self) -> anyhow::Result<Vec<GroupInfo>>;
}
