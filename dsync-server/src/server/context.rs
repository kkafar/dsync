#![allow(unused)]
use std::sync::Arc;

use super::{config::Config, database::DatabaseProxy};

/// Context that might be shared between different services.
// This should be thread safe!!!
pub(crate) struct ServerContext {
    pub cfg: Config,
    pub db_proxy: Arc<DatabaseProxy>,
}
