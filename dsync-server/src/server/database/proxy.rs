use anyhow::Context;
use diesel::{
    ExpressionMethods, QueryDsl, RunQueryDsl, SelectableHelper, SqliteConnection,
    result::DatabaseErrorKind,
};
use dsync_proto::model::server::{GroupInfo, HostInfo};
use std::ops::DerefMut;

use super::error::{FileAddError, LocalServerBaseInfoError, SaveLocalGroupError};

use crate::server::database::{
    error::DeleteLocalGroupError,
    models::{
        FilesLocalFragmentInsert, FilesLocalRow, GroupsLocalFragmentInsert, GroupsLocalRow,
        HostsRow,
    },
    schema,
};

pub(crate) struct DatabaseProxy {
    conn: tokio::sync::Mutex<SqliteConnection>,
}

impl DatabaseProxy {
    pub fn new(conn: SqliteConnection) -> Self {
        Self {
            conn: tokio::sync::Mutex::new(conn),
        }
    }
}

impl DatabaseProxy {
    pub async fn fetch_local_server_info(&self) -> Result<HostsRow, LocalServerBaseInfoError> {
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

        // Unwrap asserted above
        Ok(records.pop().unwrap())
    }

    pub async fn ensure_db_record_exists(&self, server_info_factory: impl FnOnce() -> HostsRow) {
        let result = self.fetch_local_server_info().await;

        match result {
            Ok(lcl_srv_info) => {
                log::info!("Server info exists: {:?}", &lcl_srv_info);
            }
            Err(err) => match err {
                LocalServerBaseInfoError::InvalidRecordCount(count) => {
                    log::error!(
                        "Corrupted state of local server info db table! More than single record present! Count: {count}."
                    );
                    panic!(
                        "Corrupted state of local server info db table! More than single record present! Count: {count}."
                    );
                }
                LocalServerBaseInfoError::Other(err) => {
                    log::error!("An error occured while fetching local server data: {err}");
                    panic!("An error occured while fetching local server data: {err}");
                }
                LocalServerBaseInfoError::Uninitialized => {
                    log::warn!("Server info table empty - generating server info");
                    let server_info = server_info_factory();
                    self.save_local_server_info(server_info).await;
                }
            },
        }
    }

    pub async fn save_local_server_info(&self, server_info: HostsRow) {
        use schema::hosts::dsl::*;

        log::trace!("Saving local server info");

        let mut connection = self.conn.lock().await;

        diesel::insert_into(hosts)
            .values(&server_info)
            .execute(connection.deref_mut())
            .expect("Failed to insert server info to db");
    }

    pub async fn fetch_host_by_name(&self, name: impl AsRef<str>) -> anyhow::Result<HostsRow> {
        use schema::hosts::dsl as ht;

        let name = name.as_ref();

        let qr_result = {
            let mut connection = self.conn.lock().await;
            QueryDsl::filter(ht::hosts, ht::name.eq(name))
                .select(HostsRow::as_select())
                .first(connection.deref_mut())
        };

        let row = match qr_result {
            Ok(row) => row,
            Err(err) => {
                anyhow::bail!("Failed to fetch the row: {err}");
            }
        };

        Ok(row)
    }

    pub async fn fetch_host_by_local_id(&self, _local_id: i32) -> anyhow::Result<HostsRow> {
        // use schema::hosts::dsl as ht;
        //
        // let qr_result = {
        //     let mut connection = self.conn.lock().await;
        //     QueryDsl::filter(ht::hosts, ht::id.eq(local_id))
        //         .select(HostsRow::as_select())
        //         .first(connection.deref_mut())
        // };
        //
        // let row = match qr_result {
        //     Ok(row) => row,
        //     Err(err) => {
        //         anyhow::bail!("Failed to fetch the row: {err}");
        //     }
        // };
        //
        // return Ok(row);
        anyhow::bail!("Not implemented yet");
    }

    pub async fn fetch_hosts(&self) -> anyhow::Result<Vec<HostInfo>> {
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

    pub async fn fetch_host_by_uuid(&self, uuid: impl AsRef<str>) -> anyhow::Result<HostsRow> {
        use schema::hosts::dsl as ht;

        let uuid = uuid.as_ref();

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

    pub async fn insert_hosts(&self, hosts_rows: &[HostsRow]) {
        use schema::hosts::dsl as ht;

        let mut connection = self.conn.lock().await;
        let conn_ref_mut: &mut SqliteConnection = &mut connection;

        // There is a weird bug, similar to https://github.com/diesel-rs/diesel/issues/1930
        // but for sqlite db, so that I can not add `on_conflict_do_nothing` and execute this
        // in single call. However, since in sqlite batch insert gets resolved into multiple queries
        // doing it in loop should be fine (https://diesel.rs/guides/all-about-inserts.html#batch-insert).
        // diesel::insert_into(psbi::table)
        //     .values(peer_info)
        //     .execute(connection.deref_mut())
        //     .expect("Failed to insert peer info to db");
        for info in hosts_rows {
            diesel::insert_into(ht::hosts)
                .values(info)
                .on_conflict_do_nothing()
                .execute(conn_ref_mut)
                .expect("Failed to insert peer info to db");
        }
    }

    pub async fn delete_host_with_uuid(&self, host_uuid: &str) {
        use schema::hosts::dsl as ht;

        let mut connection = self.conn.lock().await;
        let conn_ref_mut: &mut SqliteConnection = &mut connection;
        let filtered_table = QueryDsl::filter(ht::hosts, ht::uuid.eq(host_uuid));
        let _ = diesel::delete(filtered_table)
            .execute(conn_ref_mut)
            .expect("Failed to delete host from db");
    }

    #[allow(unused)]
    pub async fn save_local_file(
        &self,
        local_file: FilesLocalFragmentInsert,
    ) -> Result<(), FileAddError> {
        use schema::files_local as fl;

        let mut connection = self.conn.lock().await;
        let conn_ref_mut = connection.deref_mut();
        match diesel::insert_into(fl::table)
            .values(&local_file)
            .execute(conn_ref_mut)
        {
            Ok(_) => Ok(()),
            Err(error) => match error {
                diesel::result::Error::DatabaseError(kind, _extras) => match kind {
                    DatabaseErrorKind::UniqueViolation => Err(FileAddError::AlreadyExists {
                        file_name: local_file.file_path,
                    }),
                    _ => Err(FileAddError::OtherDatabaseError { kind }),
                },
                _ => Err(FileAddError::Other(error.into())),
            },
        }
    }

    pub async fn save_local_files(
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

    pub async fn fetch_local_files(&self) -> anyhow::Result<Vec<FilesLocalRow>> {
        use schema::files_local as fl;

        let mut connection = self.conn.lock().await;
        let conn_ref_mut = connection.deref_mut();

        let result = fl::table
            .select(FilesLocalRow::as_select())
            .load(conn_ref_mut)
            .context("Failed to fetch local files from db")?;

        anyhow::Ok(result)
    }

    pub async fn delete_local_file(&self, file_path: &str) -> anyhow::Result<usize> {
        use schema::files_local::dsl as fl;

        let mut connection = self.conn.lock().await;
        let conn_ref_mut = &mut *connection;

        let result = diesel::delete(fl::files_local)
            .filter(fl::file_path.eq(file_path))
            .execute(conn_ref_mut)?;

        anyhow::Ok(result)
    }

    pub async fn delete_group_by_name(
        &self,
        group_name: &str,
    ) -> Result<(), DeleteLocalGroupError> {
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

    pub async fn save_local_group(&self, group_id: &str) -> Result<usize, SaveLocalGroupError> {
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

    pub async fn fetch_local_groups(&self) -> Result<Vec<GroupInfo>, anyhow::Error> {
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
