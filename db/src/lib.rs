use std::path::PathBuf;

use thiserror::Error;
use tracing::info;

pub type DbResult<T> = Result<T, DbError>;

#[derive(Debug, Clone)]
pub struct LocalDbConfig {
    pub data_dir: PathBuf,
}

impl LocalDbConfig {
    pub fn new<P: Into<PathBuf>>(data_dir: P) -> Self {
        Self {
            data_dir: data_dir.into(),
        }
    }
}

pub struct LocalDb {
    config: LocalDbConfig,
}

impl LocalDb {
    pub fn new(config: LocalDbConfig) -> Self {
        Self { config }
    }

    /// Initialize the local database. Placeholder for LanceDB hookup.
    pub fn init(&self) -> DbResult<()> {
        info!(path = ?self.config.data_dir, "Initializing local DB (placeholder for LanceDB)");
        Ok(())
    }
}

#[derive(Debug, Error)]
pub enum DbError {
    #[error("io error: {0}")]
    Io(String),
}
