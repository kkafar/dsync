use dsync_proto::model::server::{GroupInfo, HostInfo};

use crate::server::data::source::DataSource;
use crate::server::database::error::{
    DeleteLocalGroupError, FileAddError, LocalServerBaseInfoError, SaveLocalGroupError,
};
use crate::server::database::models::{FilesLocalFragmentInsert, FilesLocalRow, HostsRow};

/// DataRepository defines the data access API used by services.
/// It abstracts the underlying data source (e.g., SQLite) and exposes methods
/// currently used by the service layer.
///
/// NOTE: This trait should remain focused on service needs. If a service starts
/// using a new operation, add it here and implement it in the repository and data source.
#[async_trait::async_trait]
pub(crate) trait DataRepository: Send + Sync {
    // Hosts (local and remote)
    async fn fetch_local_server_info(&self) -> Result<HostsRow, LocalServerBaseInfoError>;
    async fn insert_hosts(&self, hosts_rows: &[HostsRow]) -> anyhow::Result<()>;
    async fn fetch_hosts(&self) -> anyhow::Result<Vec<HostInfo>>;
    async fn fetch_host_by_uuid(&self, uuid: &str) -> anyhow::Result<HostsRow>;
    async fn fetch_host_by_name(&self, name: &str) -> anyhow::Result<HostsRow>;
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

/// MainRepository is the concrete repository used by services.
/// It delegates to a data source that implements the same operations.
/// The data source is injected, enabling swapping implementations (e.g., SQLite, Postgres, mocks).
pub(crate) struct MainDataRepository<Ds> {
    data_source: Box<Ds>,
}

impl<Ds> MainDataRepository<Ds> {
    pub fn new(data_source: Box<Ds>) -> Self {
        Self { data_source }
    }
}

#[async_trait::async_trait]
impl<Ds> DataRepository for MainDataRepository<Ds>
where
    Ds: DataSource,
{
    // Hosts (local and remote)
    async fn fetch_local_server_info(&self) -> Result<HostsRow, LocalServerBaseInfoError> {
        self.data_source.fetch_local_server_info().await
    }

    async fn insert_hosts(&self, hosts_rows: &[HostsRow]) -> anyhow::Result<()> {
        self.data_source.insert_hosts(hosts_rows).await
    }

    async fn fetch_hosts(&self) -> anyhow::Result<Vec<HostInfo>> {
        self.data_source.fetch_hosts().await
    }

    async fn fetch_host_by_uuid(&self, uuid: &str) -> anyhow::Result<HostsRow> {
        self.data_source.fetch_host_by_uuid(uuid).await
    }

    async fn fetch_host_by_name(&self, name: &str) -> anyhow::Result<HostsRow> {
        self.data_source.fetch_host_by_name(name).await
    }

    async fn fetch_host_by_local_id(&self, _local_id: i32) -> anyhow::Result<HostsRow> {
        // Stub implementation to preserve behavior; data source should implement real logic later.
        anyhow::bail!("fetch_host_by_local_id is not implemented yet")
    }

    async fn delete_host_with_uuid(&self, host_uuid: &str) -> anyhow::Result<()> {
        self.data_source.delete_host_with_uuid(host_uuid).await
    }

    // Files (local)
    async fn save_local_files(
        &self,
        local_files: &[FilesLocalFragmentInsert],
    ) -> Result<(), FileAddError> {
        self.data_source.save_local_files(local_files).await
    }

    async fn fetch_local_files(&self) -> anyhow::Result<Vec<FilesLocalRow>> {
        self.data_source.fetch_local_files().await
    }

    async fn delete_local_file(&self, file_path: &str) -> anyhow::Result<usize> {
        self.data_source.delete_local_file(file_path).await
    }

    // Groups (local)
    async fn save_local_group(&self, group_id: &str) -> Result<usize, SaveLocalGroupError> {
        self.data_source.save_local_group(group_id).await
    }

    async fn delete_group_by_name(&self, group_name: &str) -> Result<(), DeleteLocalGroupError> {
        self.data_source.delete_group_by_name(group_name).await
    }

    async fn fetch_local_groups(&self) -> anyhow::Result<Vec<GroupInfo>> {
        self.data_source.fetch_local_groups().await
    }
}
