use std::fmt;

use indexer_core::{db, db::PooledConnection};

use crate::prelude::*;

/// Handle to a database pool used by an indexer consumer
pub struct Pool(db::Pool);

impl std::panic::UnwindSafe for Pool {}
impl std::panic::RefUnwindSafe for Pool {}

impl Pool {
    pub(crate) fn new(pool: db::Pool) -> Self {
        Self(pool)
    }

    /// Spawn a blocking thread to perform operations on the database.
    ///
    /// # Errors
    /// This function fails if `r2d2` cannot acquire a database connection or
    /// the provided callback returns an error.
    pub(crate) async fn run<T: 'static + Send, E: 'static + Into<indexer_core::error::Error>>(
        &self,
        f: impl FnOnce(&PooledConnection) -> Result<T, E> + Send + 'static,
    ) -> Result<T> {
        let db = self
            .0
            .get()
            .context("Failed to acquire database connection");

        tokio::task::spawn_blocking(|| f(&db?).map_err(Into::into))
            .await
            .context("Blocking task failed")?
    }
}

impl fmt::Debug for Pool {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Pool").finish_non_exhaustive()
    }
}
