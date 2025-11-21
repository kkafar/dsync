use std::sync::Arc;

use super::{config::RunConfiguration, database::DatabaseProxy};

/// Context that might be shared between different services.
// This should be thread safe!!!
pub(crate) struct ServerContext {
    pub run_config: RunConfiguration,
    pub db_proxy: Arc<DatabaseProxy>,
}
