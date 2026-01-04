use anyhow::Context;
use async_trait::async_trait;
use diesel::{
    ExpressionMethods, QueryDsl, RunQueryDsl, SelectableHelper, SqliteConnection,
    result::DatabaseErrorKind,
};
use dsync_proto::model::server::{GroupInfo, HostInfo};
use std::ops::DerefMut;

pub(crate) mod database;

use crate::server::data::source::{
    DataSource,
    sqlite::database::{
        error::{
            DeleteLocalGroupError, FileAddError, LocalServerBaseInfoError, SaveLocalGroupError,
        },
        models::{
            FilesLocalFragmentInsert, FilesLocalRow, GroupsLocalFragmentInsert, GroupsLocalRow,
            HostsRow,
        },
        schema,
    },
};

/// SqliteLocalMainDataSource is a concrete data source backed by a single-threaded Diesel
/// `SqliteConnection` wrapped with an async-aware `tokio::sync::Mutex`.
///
/// It ports methods that were previously implemented on `DatabaseProxy`.
pub struct SqliteDataSource {
    conn: tokio::sync::Mutex<SqliteConnection>,
}

impl SqliteDataSource {
    pub async fn new(
        conn: SqliteConnection,
        server_info_factory: impl FnOnce() -> HostsRow,
    ) -> anyhow::Result<Self> {
        let ds = Self {
            conn: tokio::sync::Mutex::new(conn),
        };

        match ds.fetch_local_server_info().await {
            Ok(_) => {}
            Err(LocalServerBaseInfoError::Uninitialized) => {
                let this_server = server_info_factory();
                ds.insert_hosts(&[this_server]).await?;
            }
            Err(LocalServerBaseInfoError::InvalidRecordCount(count)) => {
                anyhow::bail!(
                    "Corrupted state of local server info db table! More than single record present! Count: {count}."
                );
            }
            Err(LocalServerBaseInfoError::Other(err)) => {
                anyhow::bail!(err);
            }
        };

        Ok(ds)
    }
}

#[async_trait]
impl DataSource for SqliteDataSource {
    // Hosts (local and remote)
    async fn fetch_local_server_info(&self) -> Result<HostsRow, LocalServerBaseInfoError> {
        use schema::hosts::dsl::*;

        let results = {
            let mut db_conn = self.conn.lock().await;
            QueryDsl::filter(hosts, is_remote.eq(false))
                .select(HostsRow::as_select())
                .load(db_conn.deref_mut())
                .context(
                    "Error while loading configuration. Maybe you forgot to initiate the database?",
                )
        };

        if let Err(err) = results {
            return Err(LocalServerBaseInfoError::Other(err));
        }

        let mut records = results.unwrap();

        if records.is_empty() {
            return Err(LocalServerBaseInfoError::Uninitialized);
        }

        if records.len() > 1 {
            return Err(LocalServerBaseInfoError::InvalidRecordCount(records.len()));
        }

        Ok(records.pop().unwrap())
    }

    async fn insert_hosts(&self, hosts_rows: &[HostsRow]) -> anyhow::Result<()> {
        use schema::hosts::dsl as ht;

        let mut connection = self.conn.lock().await;
        let conn_ref_mut: &mut SqliteConnection = &mut connection;

        // See Diesel note in original implementation: batch insert resolves to multiple queries in SQLite.
        for info in hosts_rows {
            diesel::insert_into(ht::hosts)
                .values(info)
                .on_conflict_do_nothing()
                .execute(conn_ref_mut)
                .context("Failed to insert peer info to db")?;
        }

        Ok(())
    }

    async fn fetch_hosts(&self) -> anyhow::Result<Vec<HostInfo>> {
        use schema::hosts::dsl::*;

        let qr_result = {
            let mut connection = self.conn.lock().await;
            hosts.select(HostsRow::as_select()).load(&mut *connection)
        };

        match qr_result {
            Ok(data) => Ok(data
                .into_iter()
                .map(|host_info| HostInfo {
                    uuid: host_info.uuid,
                    name: host_info.name,
                    hostname: host_info.hostname,
                    address: host_info.ipv4_addr,
                })
                .collect()),
            Err(error) => {
                log::error!("Error while fetching peer server information: {error}");
                Err(error.into())
            }
        }
    }

    async fn fetch_host_by_uuid(&self, uuid: &str) -> anyhow::Result<HostsRow> {
        use schema::hosts::dsl as ht;

        let qr_result = {
            let mut connection = self.conn.lock().await;
            QueryDsl::filter(ht::hosts, ht::uuid.eq(uuid))
                .select(HostsRow::as_select())
                .first(connection.deref_mut())
        };

        let Ok(row) = qr_result else {
            anyhow::bail!("Failed to fetch the row");
        };

        Ok(row)
    }

    async fn fetch_host_by_name(&self, name: &str) -> anyhow::Result<HostsRow> {
        use schema::hosts::dsl as ht;

        let qr_result = {
            let mut connection = self.conn.lock().await;
            QueryDsl::filter(ht::hosts, ht::name.eq(name))
                .select(HostsRow::as_select())
                .first(connection.deref_mut())
        };

        let Ok(row) = qr_result else {
            anyhow::bail!("Failed to fetch the row by name");
        };

        Ok(row)
    }

    async fn fetch_host_by_local_id(&self, _local_id: i32) -> anyhow::Result<HostsRow> {
        anyhow::bail!("fetch_host_by_local_id is not implemented yet")
    }

    async fn delete_host_with_uuid(&self, host_uuid: &str) -> anyhow::Result<()> {
        use schema::hosts::dsl as ht;

        let mut connection = self.conn.lock().await;
        let conn_ref_mut: &mut SqliteConnection = &mut connection;
        let filtered_table = QueryDsl::filter(ht::hosts, ht::uuid.eq(host_uuid));
        let _ = diesel::delete(filtered_table)
            .execute(conn_ref_mut)
            .context("Failed to delete host from db")?;

        Ok(())
    }

    // Files (local)
    async fn save_local_files(
        &self,
        local_files: &[FilesLocalFragmentInsert],
    ) -> Result<(), FileAddError> {
        use schema::files_local as fl;

        let query_res = {
            let mut connection = self.conn.lock().await;
            diesel::insert_or_ignore_into(fl::table)
                .values(local_files)
                .execute(&mut *connection)
        };

        match query_res {
            Ok(_) => Ok(()),
            Err(error) => match error {
                diesel::result::Error::DatabaseError(kind, _extras) => match kind {
                    DatabaseErrorKind::UniqueViolation => Err(FileAddError::AlreadyExists {
                        file_name: String::from("unknown"),
                    }),
                    _ => Err(FileAddError::OtherDatabaseError { kind }),
                },
                _ => Err(FileAddError::Other(error.into())),
            },
        }
    }

    async fn fetch_local_files(&self) -> anyhow::Result<Vec<FilesLocalRow>> {
        use schema::files_local as fl;

        let mut connection = self.conn.lock().await;
        let conn_ref_mut = connection.deref_mut();

        let result = fl::table
            .select(FilesLocalRow::as_select())
            .load(conn_ref_mut)
            .context("Failed to fetch local files from db")?;

        anyhow::Ok(result)
    }

    async fn delete_local_file(&self, file_path: &str) -> anyhow::Result<usize> {
        use schema::files_local::dsl as fl;

        let mut connection = self.conn.lock().await;
        let conn_ref_mut = &mut *connection;

        let result = diesel::delete(fl::files_local)
            .filter(fl::file_path.eq(file_path))
            .execute(conn_ref_mut)?;

        anyhow::Ok(result)
    }

    // Groups (local)
    async fn save_local_group(&self, group_id: &str) -> Result<usize, SaveLocalGroupError> {
        use schema::groups_local as gl;

        let mut connection = self.conn.lock().await;
        let result = diesel::insert_into(gl::table)
            .values(GroupsLocalFragmentInsert {
                name: group_id.to_owned(),
            })
            .execute(&mut *connection);

        std::mem::drop(connection);

        match result {
            Ok(aff_rows) => Ok(aff_rows),
            Err(err) => match err {
                diesel::result::Error::DatabaseError(db_err_kind, _) => {
                    if let diesel::result::DatabaseErrorKind::UniqueViolation = db_err_kind {
                        Err(SaveLocalGroupError::AlreadyExists {
                            group_id: group_id.into(),
                        })
                    } else {
                        Err(SaveLocalGroupError::Other)
                    }
                }
                _ => Err(SaveLocalGroupError::Other),
            },
        }
    }

    async fn delete_group_by_name(&self, group_name: &str) -> Result<(), DeleteLocalGroupError> {
        use schema::groups_local::dsl as gl;

        let query_res = {
            let mut conn = self.conn.lock().await;
            diesel::delete(QueryDsl::filter(gl::groups_local, gl::name.eq(group_name)))
                .execute(&mut *conn)
                .map_err(|error| DeleteLocalGroupError::Other(error.into()))
        }?;

        if query_res == 0 {
            Err(DeleteLocalGroupError::DoesNotExist)
        } else {
            Ok(())
        }
    }

    async fn fetch_local_groups(&self) -> anyhow::Result<Vec<GroupInfo>> {
        use schema::groups_local as gl;

        let mut connection = self.conn.lock().await;
        let result = gl::table
            .select(GroupsLocalRow::as_select())
            .load(&mut *connection)?;

        std::mem::drop(connection);

        Ok(result
            .into_iter()
            .map(|row| GroupInfo {
                local_id: row.id,
                name: row.name,
            })
            .collect())
    }
}
