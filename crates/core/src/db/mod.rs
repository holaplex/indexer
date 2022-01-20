//! Interface with the indexer database

pub mod models;
pub mod queries;
#[allow(missing_docs, unused_imports)]
mod schema;

pub mod tables {
    //! Diesel schema DSLs

    pub use super::schema::{
        bids, editions, listing_metadatas, listings, master_editions, metadata_creators, metadatas,
        store_denylist, storefronts, token_accounts,
    };
}

use std::env;

pub use diesel::{insert_into, pg::upsert::excluded, select, update};
use diesel::{pg, r2d2};
pub use diesel_full_text_search::{
    websearch_to_tsquery, TsQuery, TsQueryExtensions, TsVector, TsVectorExtensions,
};

use crate::prelude::*;

embed_migrations!("migrations");

/// Postgres database connection
pub type Connection = pg::PgConnection;
/// R2D2 connection manager for Postgres
pub type ConnectionManager = r2d2::ConnectionManager<Connection>;
/// R2D2 connection pool for Postgres
pub type Pool = r2d2::Pool<ConnectionManager>;
/// Pooled Postgres connection
pub type PooledConnection = r2d2::PooledConnection<ConnectionManager>;

/// Hint indicating how the database should be connected
#[derive(Debug, Clone, Copy)]
pub enum ConnectMode {
    /// Open the database for reading
    ///
    /// This will check for a `DATABASE_READ_URL` for read replicas.
    Read,
    /// Open the database for writing
    ///
    /// This will check for a `DATABASE_WRITE_URL` for a primary replica.
    Write,
}

/// Create a pooled connection to the Postgres database.  This will check for
/// the presence of `DATABASE_(READ|WRITE)_URL` (depending on the mode
/// specified) or else `DATABASE_URL`.
///
/// # Errors
/// This function fails if neither of the above environment variables are found,
/// if Diesel fails to construct a connection pool, or if any pending database
/// migrations fail to run.
pub fn connect(mode: ConnectMode) -> Result<Pool> {
    let mode_env = match mode {
        ConnectMode::Read => "DATABASE_READ_URL",
        ConnectMode::Write => "DATABASE_WRITE_URL",
    };

    let url = env::var_os(mode_env)
        .or_else(|| env::var_os("DATABASE_URL"))
        .ok_or_else(|| anyhow!("No value found for {} or DATABASE_URL", mode_env))
        .map(move |v| v.to_string_lossy().into_owned())?;

    debug!("Connecting to db: {:?}", url);

    let man = ConnectionManager::new(url);
    let pool = Pool::builder()
        .build(man)
        .context("Failed to create database connection pool")?;

    let mut out = vec![];

    info!("Running database migrations...");
    embedded_migrations::run_with_output(
        &pool.get().context("Failed to connect to the database")?,
        &mut out,
    )
    .context("Failed to run database migrations")?;

    match std::str::from_utf8(&out) {
        Ok(s) => {
            let s = s.trim();

            if !s.is_empty() {
                info!("Output from migrations:\n{}", s);
            }
        },
        Err(e) => warn!("Failed to read migration output: {}", e),
    }

    Ok(pool)
}
