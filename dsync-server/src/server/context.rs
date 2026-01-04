#![allow(unused)]

use std::sync::Arc;

use super::config::Config;
use super::data::repo::DataRepository;

/// Context that might be shared between different services.
/// This should be thread-safe.
pub(crate) struct ServerContext {
    pub cfg: Config,
    pub repo: Arc<dyn DataRepository>,
}

impl ServerContext {
    pub fn new(cfg: Config, repo: Arc<dyn DataRepository>) -> Self {
        Self { cfg, repo }
    }
}
