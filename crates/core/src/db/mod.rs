//! Interface with the indexer database

pub mod models;
pub mod pagination;
pub mod queries;
#[allow(missing_docs, unused_imports)]
mod schema;

pub mod tables {
    //! Diesel schema DSLs

    pub use super::schema::*;
}

use std::env;

pub use diesel::{
    backend::Backend, insert_into, pg::upsert::excluded, query_dsl, result::Error, select,
    serialize, sql_types, update, Queryable,
};
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

/// Hint indicating how a returned database connection should be interpreted
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ConnectionType {
    /// The `DATABASE_URL` var was used and is likely writable
    Default,
    /// The `DATABASE_READ_URL` var was used and is not writable
    Read,
    /// The `DATABASE_WRITE_URL` var was used and should be writable
    Write,
}

impl From<ConnectMode> for ConnectionType {
    fn from(mode: ConnectMode) -> Self {
        match mode {
            ConnectMode::Read => Self::Read,
            ConnectMode::Write => Self::Write,
        }
    }
}

/// Create a pooled connection to the Postgres database.  This will check for
/// the presence of `DATABASE_(READ|WRITE)_URL` (depending on the mode
/// specified) or else `DATABASE_URL`.
///
/// # Errors
/// This function fails if neither of the above environment variables are found,
/// if Diesel fails to construct a connection pool, or if any pending database
/// migrations fail to run.
pub fn connect(mode: ConnectMode) -> Result<(Pool, ConnectionType)> {
    let mode_env = match mode {
        ConnectMode::Read => "DATABASE_READ_URL",
        ConnectMode::Write => "DATABASE_WRITE_URL",
    };

    let (ty, url) = env::var_os(mode_env)
        .map(|v| (mode.into(), v))
        .or_else(|| env::var_os("DATABASE_URL").map(|v| (ConnectionType::Default, v)))
        .ok_or_else(|| anyhow!("No value found for {} or DATABASE_URL", mode_env))?;
    let url = url.to_string_lossy().into_owned();

    debug!("Connecting to db: {:?}", url);

    let man = ConnectionManager::new(url);
    let pool = Pool::builder()
        .min_idle(Some(1))
        .idle_timeout(Some(std::time::Duration::from_secs(60)))
        .build(man)
        .context("Failed to create database connection pool")?;

    let mut out = vec![];

    if cfg!(not(debug_assertions)) && matches!(ty, ConnectionType::Default) {
        warn!("Cannot determine if database is writable; assuming yes");
    }

    if matches!(ty, ConnectionType::Read) {
        info!("Not running migrations over a read-only connection");
    } else {
        info!("Running database migrations...");
        embedded_migrations::run_with_output(
            &pool.get().context("Failed to connect to the database")?,
            &mut out,
        )
        .context("Failed to run database migrations")?;
    }

    match std::str::from_utf8(&out) {
        Ok(s) => {
            let s = s.trim();

            if !s.is_empty() {
                info!("Output from migrations:\n{}", s);
            }
        },
        Err(e) => warn!("Failed to read migration output: {}", e),
    }

    Ok((pool, ty))
}
