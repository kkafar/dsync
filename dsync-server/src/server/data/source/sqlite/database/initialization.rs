use crate::server::data::source::SqliteDataSource;

use diesel_migrations::{EmbeddedMigrations, MigrationHarness, embed_migrations};

pub(super) const MIGRATIONS: EmbeddedMigrations = embed_migrations!();

impl SqliteDataSource {
    pub(crate) async fn init_if_needed(&mut self) -> Result<(), anyhow::Error> {
        log::trace!("Initializing SqliteDataSource");
        let mut db_connection = self.conn.lock().await;

        db_connection
            .run_pending_migrations(MIGRATIONS)
            .map_err(|err| anyhow::anyhow!("Failed to run migrations with err: {}", err))?;

        log::info!("Database initialized");
        Ok(())
    }
}
