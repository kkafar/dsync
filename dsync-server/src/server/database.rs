use std::ops::DerefMut;

use anyhow::Context;
use diesel::{QueryDsl, RunQueryDsl, SelectableHelper, SqliteConnection};

use crate::models::ThisServerInfoRow;

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
    pub async fn fetch_this_server_info(&self) -> anyhow::Result<ThisServerInfoRow> {
        use crate::schema::this_server_info::dsl::*;
        let mut db_conn = self.conn.lock().await;

        let results = this_server_info
            .select(ThisServerInfoRow::as_select())
            .load(db_conn.deref_mut())
            .context("Error while loading configuration")?;

        if results.len() != 1 {
            return Err(anyhow::anyhow!(
                "Expected this_server_info to be populated with exactly one record"
            ));
        }

        // Unwrap asserted above
        let server_info = results[0].clone();

        return anyhow::Ok(server_info);
    }
}
