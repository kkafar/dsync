use std::ops::DerefMut;

use anyhow::Context;
use diesel::{QueryDsl, RunQueryDsl, SelectableHelper, SqliteConnection};

use crate::models::LocalServerBaseInfoRow;

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
            self.save_this_server_info(server_info).await;
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

    pub async fn save_this_server_info(&self, server_info: LocalServerBaseInfoRow) {
        use crate::schema::local_server_base_info;

        let mut connection = self.conn.lock().await;

        diesel::insert_into(local_server_base_info::table)
            .values(&server_info)
            .execute(connection.deref_mut())
            .expect("Failed to insert server info to db");
    }
}
