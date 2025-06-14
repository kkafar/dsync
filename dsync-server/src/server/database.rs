use std::ops::DerefMut;

use anyhow::{Context, anyhow};
use diesel::{ExpressionMethods, QueryDsl, RunQueryDsl, SelectableHelper, SqliteConnection};
use dsync_proto::shared;

use models::{
    LocalFilesWoIdRow, LocalGroupQueryRow, LocalGroupWoIdInsertRow, LocalServerBaseInfoRow,
    PeerAddrV4Row, PeerServerBaseInfoRow,
};

pub(crate) mod models;
mod schema;

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
    pub async fn fetch_local_server_info(
        &self,
    ) -> Result<LocalServerBaseInfoRow, LocalServerBaseInfoError> {
        use schema::local_server_base_info::dsl::*;
        let mut db_conn = self.conn.lock().await;

        let results = local_server_base_info
            .select(LocalServerBaseInfoRow::as_select())
            .load(db_conn.deref_mut())
            .context("Error while loading configuration");

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
        return Ok(records.pop().unwrap());
    }

    pub async fn ensure_db_record_exists(
        &self,
        server_info_factory: impl FnOnce() -> LocalServerBaseInfoRow,
    ) {
        let result = self.fetch_local_server_info().await;

        match result {
            Ok(lcl_srv_info) => {
                log::info!("Server info exists: {:?}", lcl_srv_info);
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

    pub async fn save_local_server_info(&self, server_info: LocalServerBaseInfoRow) {
        use schema::local_server_base_info;

        log::trace!("Saving local server info");

        let mut connection = self.conn.lock().await;

        diesel::insert_into(local_server_base_info::table)
            .values(&server_info)
            .execute(connection.deref_mut())
            .expect("Failed to insert server info to db");
    }

    pub async fn fetch_peer_server_info(&self) -> anyhow::Result<Vec<shared::ServerInfo>> {
        use schema::peer_addr_v4 as pa;
        use schema::peer_server_base_info as psbi;

        let mut connection = self.conn.lock().await;

        let query_result = psbi::table
            .inner_join(pa::table)
            .select((
                PeerServerBaseInfoRow::as_select(),
                PeerAddrV4Row::as_select(),
            ))
            .load::<(PeerServerBaseInfoRow, PeerAddrV4Row)>(connection.deref_mut());

        std::mem::drop(connection);

        match query_result {
            Ok(data) => {
                return Ok(data
                    .into_iter()
                    .map(|(srv_base_info, srv_addr_info)| shared::ServerInfo {
                        uuid: srv_base_info.uuid,
                        name: srv_base_info.name,
                        hostname: srv_base_info.hostname,
                        address: srv_addr_info.ipv4_addr,
                    })
                    .collect());
            }
            Err(error) => {
                log::error!("Error while fetching peer server information: {error}");
                return Err(error.into());
            }
        }
    }

    pub async fn save_peer_server_base_info(&self, peer_info: &[PeerServerBaseInfoRow]) {
        use schema::peer_server_base_info as psbi;

        let mut connection = self.conn.lock().await;
        let conn_ref_mut = connection.deref_mut();

        // There is a weird bug, similar to https://github.com/diesel-rs/diesel/issues/1930
        // but for sqlite db, so that I can not add `on_conflict_do_nothing` and execute this
        // in single call. However, since in sqlite batch insert gets resolved into multiple queries
        // doing it in loop should be fine (https://diesel.rs/guides/all-about-inserts.html#batch-insert).
        // diesel::insert_into(psbi::table)
        //     .values(peer_info)
        //     .execute(connection.deref_mut())
        //     .expect("Failed to insert peer info to db");
        for info in peer_info {
            diesel::insert_into(psbi::table)
                .values(info)
                .on_conflict_do_nothing()
                .execute(conn_ref_mut)
                .expect("Failed to insert peer info to db");
        }
    }

    pub async fn save_peer_server_addr_info(&self, peer_addr_info: &[PeerAddrV4Row]) {
        use schema::peer_addr_v4 as pa;

        let mut connection = self.conn.lock().await;
        let conn_ref_mut = connection.deref_mut();
        for info in peer_addr_info {
            diesel::insert_into(pa::table)
                .values(info)
                .on_conflict_do_nothing()
                .execute(conn_ref_mut)
                .expect("Failed to insert peer addr info to db");
        }
    }

    pub async fn save_local_file(&self, local_file: LocalFilesWoIdRow) {
        use schema::local_files as lf;

        let mut connection = self.conn.lock().await;
        let conn_ref_mut = connection.deref_mut();
        diesel::insert_into(lf::table)
            .values(local_file)
            .execute(conn_ref_mut)
            .expect("Failed to register local file as tracked");
    }

    pub async fn fetch_local_files(&self) -> anyhow::Result<Vec<models::LocalFilesRow>> {
        use schema::local_files as lf;

        let mut connection = self.conn.lock().await;
        let conn_ref_mut = connection.deref_mut();

        let result = lf::table
            .select(models::LocalFilesRow::as_select())
            .load(conn_ref_mut)
            .context("Failed to fetch local files from db")?;

        anyhow::Ok(result)
    }

    pub async fn delete_local_file(&self, file_path: &str) -> anyhow::Result<usize> {
        use schema::local_files as lf;

        let mut connection = self.conn.lock().await;
        let conn_ref_mut = &mut *connection;

        let result = diesel::delete(lf::table)
            .filter(lf::dsl::file_path.eq(file_path))
            .execute(conn_ref_mut)?;

        anyhow::Ok(result)
    }

    pub async fn save_local_group(&self, group_id: &str) -> Result<usize, SaveLocalGroupError> {
        use schema::local_groups as lg;

        let mut connection = self.conn.lock().await;
        let result = diesel::insert_into(lg::table)
            .values(LocalGroupWoIdInsertRow { name: group_id })
            .execute(&mut *connection);

        match result {
            Ok(aff_rows) => Ok(aff_rows),
            Err(err) => match err {
                diesel::result::Error::DatabaseError(db_err_kind, _) => match db_err_kind {
                    diesel::result::DatabaseErrorKind::UniqueViolation => {
                        Err(SaveLocalGroupError::AlreadyExists {
                            group_id: group_id.into(),
                        })
                    }
                    _ => Err(SaveLocalGroupError::Other),
                },
                _ => Err(SaveLocalGroupError::Other),
            },
        }
    }

    pub async fn fetch_local_groups(&self) -> Result<Vec<shared::GroupInfo>, anyhow::Error> {
        use schema::local_groups as lg;

        let mut connection = self.conn.lock().await;
        let result = lg::table
            .select(LocalGroupQueryRow::as_select())
            .load::<LocalGroupQueryRow>(&mut *connection)?;

        Ok(result
            .into_iter()
            .map(|row| shared::GroupInfo {
                local_id: row.id,
                name: row.name,
            })
            .collect())
    }
}

#[derive(thiserror::Error, Debug)]
pub enum LocalServerBaseInfoError {
    #[error("No local server base info present")]
    Uninitialized,

    #[error("Invalid record count: `{0}`")]
    InvalidRecordCount(usize),

    #[error("Other error `{0}`")]
    Other(anyhow::Error),
}

#[derive(thiserror::Error, Debug)]
pub enum SaveLocalGroupError {
    #[error("Group with id: {group_id} already exists")]
    AlreadyExists { group_id: String },

    #[error("Other error when saving local group")]
    Other,
}

// pub enum PeerServerInfoFetchError {
//     NoExistingAddressRecord
// }
