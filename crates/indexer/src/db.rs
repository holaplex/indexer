//! Support module for running Diesel operations in an async context.

use indexer_core::{
    db,
    db::{ConnectResult, PooledConnection},
};

use crate::prelude::*;

/// Handle to a database pool used by an indexer consumer
#[repr(transparent)]
#[derive(Debug)]
pub struct Pool(ConnectResult);

impl Pool {
    pub(crate) fn new(res: ConnectResult) -> Self {
        Self(res)
    }

    /// Get the connection-type hint for this database connection
    #[must_use]
    pub fn ty(&self) -> db::ConnectionType {
        self.0.ty
    }

    /// Get whether migrations were run upon connecting
    #[must_use]
    pub fn migrated(&self) -> bool {
        self.0.migrated
    }

    /// Spawn a blocking thread to perform operations on the database.
    ///
    /// # Errors
    /// This function fails if `r2d2` cannot acquire a database connection or
    /// the provided callback returns an error.
    #[allow(dead_code)]
    pub(crate) async fn run<T: 'static + Send, E: 'static + Into<indexer_core::error::Error>>(
        &self,
        f: impl FnOnce(&PooledConnection) -> Result<T, E> + Send + 'static,
    ) -> Result<T> {
        let db = self
            .0
            .pool
            .get()
            .context("Failed to acquire database connection");

        tokio::task::spawn_blocking(|| f(&db?).map_err(Into::into))
            .await
            .context("Blocking task failed")?
    }
}
