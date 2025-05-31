use std::sync::Arc;

use diesel::SqliteConnection;

use super::config::RunConfiguration;

/// Context that might be shared between different services.
// This should be thread safe!!!
pub(crate) struct GlobalContext {
    pub run_config: RunConfiguration,
    pub db_conn: Arc<tokio::sync::Mutex<SqliteConnection>>,
}
