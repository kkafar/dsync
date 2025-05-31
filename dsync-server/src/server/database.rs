use std::ops::DerefMut;

use anyhow::Context;
use diesel::{QueryDsl, RunQueryDsl, SelectableHelper, SqliteConnection};

use crate::models::{LocalServerBaseInfoRow, PeerAddrV4Row, PeerServerBaseInfoRow};

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
    pub async fn fetch_local_server_info(&self) -> anyhow::Result<LocalServerBaseInfoRow> {
        use crate::schema::local_server_base_info::dsl::*;
        let mut db_conn = self.conn.lock().await;

        let results = local_server_base_info
            .select(LocalServerBaseInfoRow::as_select())
            .load(db_conn.deref_mut())
            .context("Error while loading configuration")?;

        if results.len() != 1 {
            return Err(anyhow::anyhow!(
                "Expected local server info table to be populated with exactly one record"
            ));
        }

        // Unwrap asserted above
        let server_info = results[0].clone();

        return anyhow::Ok(server_info);
    }

    pub async fn ensure_db_record_exists(
        &self,
        server_info_factory: impl FnOnce() -> LocalServerBaseInfoRow,
    ) {
        use crate::schema::local_server_base_info::dsl::*;

        let mut connection = self.conn.lock().await;

        let results = local_server_base_info
            .select(LocalServerBaseInfoRow::as_select())
            .load(connection.deref_mut())
            .expect("Error while loading configuration");

        if results.is_empty() {
            log::info!("Server info table empty - generating server info");
            let server_info = server_info_factory();
            self.save_local_server_info(server_info).await;
        } else if results.len() == 1 {
            log::trace!("Server info exists");
        } else {
            log::error!(
                "Corrupted state of local server info db table! More than single record present!"
            );
            panic!(
                "Corrupted state of local server info db table! More than single record present!"
            );
        }
    }

    pub async fn save_local_server_info(&self, server_info: LocalServerBaseInfoRow) {
        use crate::schema::local_server_base_info;

        let mut connection = self.conn.lock().await;

        diesel::insert_into(local_server_base_info::table)
            .values(&server_info)
            .execute(connection.deref_mut())
            .expect("Failed to insert server info to db");
    }

    pub async fn save_peer_server_base_info(&self, peer_info: &[PeerServerBaseInfoRow]) {
        use crate::schema::peer_server_base_info as psbi;

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
        use crate::schema::peer_addr_v4 as pa;

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
}
