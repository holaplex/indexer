//! Interface with the indexer database

pub mod custom_types;
pub mod models;
pub mod mutations;
pub mod queries;

#[allow(missing_docs, unused_imports)]
mod schema;

pub mod tables {
    //! Diesel schema DSLs

    pub use super::schema::*;
}

use std::fmt;

pub use diesel::{
    backend::Backend,
    debug_query, delete, expression, insert_into,
    pg::{
        upsert::{excluded, on_constraint},
        Pg,
    },
    query_dsl,
    result::{DatabaseErrorKind, Error},
    select, serialize, sql_query, sql_types, update, Queryable,
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
    Write {
        /// Set to true to run migrations upon connecting
        migrate: bool,
    },
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

/// Resturn type of [`connect`]
pub struct ConnectResult {
    /// The database connection pool
    pub pool: Pool,
    /// Type hint indicating if the connected database should be writable
    pub ty: ConnectionType,
    /// True if migrations were run upon connecting
    pub migrated: bool,
}

impl fmt::Debug for ConnectResult {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("ConnectResult")
            .field("ty", &self.ty)
            .field("migrated", &self.migrated)
            .finish_non_exhaustive()
    }
}

/// Arguments for establishing a database connection
#[derive(Debug, clap::Args)]
pub struct ConnectArgs {
    /// Connection string for a read-only database
    #[clap(long, env, conflicts_with("database-write-url"))]
    database_read_url: Option<String>,

    /// Connection string for a writable database
    #[clap(long, env, conflicts_with("database-read-url"))]
    database_write_url: Option<String>,

    /// Fallback database connection string
    #[clap(
        long,
        env,
        required_unless_present_any(["database-read-url", "database-write-url"])
    )]
    database_url: Option<String>,
}

impl From<ConnectMode> for ConnectionType {
    fn from(mode: ConnectMode) -> Self {
        match mode {
            ConnectMode::Read => Self::Read,
            ConnectMode::Write { .. } => Self::Write,
        }
    }
}

/// Create a pooled connection to the Postgres database, using the given CLI
/// arguments and a hint indicating if the database is writable.
///
/// # Errors
/// This function fails if Diesel fails to construct a connection pool or if any
/// pending database migrations fail to run.
pub fn connect(args: ConnectArgs, mode: ConnectMode) -> Result<ConnectResult> {
    let ConnectArgs {
        database_read_url,
        database_write_url,
        database_url,
    } = args;

    let mode_url = match mode {
        ConnectMode::Read => database_read_url,
        ConnectMode::Write { .. } => database_write_url,
    };

    let (ty, url) = mode_url
        .map(|u| (mode.into(), u))
        .or_else(|| database_url.map(|u| (ConnectionType::Default, u)))
        .ok_or_else(|| {
            anyhow!(
                "Invalid database URL, expected a {} connection string",
                match mode {
                    ConnectMode::Read => "read-only",
                    ConnectMode::Write { .. } => "writable",
                }
            )
        })?;

    debug!("Connecting to db: {:?}", url);

    let man = ConnectionManager::new(url);
    let pool = Pool::builder()
        .max_size(num_cpus::get().try_into().unwrap_or(u32::MAX))
        .min_idle(Some(1))
        .idle_timeout(Some(std::time::Duration::from_secs(60)))
        .build(man)
        .context("Failed to create database connection pool")?;

    let mut out = vec![];

    if cfg!(not(debug_assertions)) && matches!(ty, ConnectionType::Default) {
        warn!("Cannot determine if database is writable; assuming yes");
    }

    let migrated = match (mode, ty) {
        (ConnectMode::Write { migrate: true }, ConnectionType::Write | ConnectionType::Default) => {
            true
        },
        (ConnectMode::Read, ConnectionType::Read | ConnectionType::Default) => {
            info!("Not running migrations over a read-only connection");
            false
        },
        (
            ConnectMode::Write { migrate: false },
            ConnectionType::Write | ConnectionType::Default,
        ) => {
            info!("Migrations not requested, skipping");
            false
        },
        (ConnectMode::Read, ConnectionType::Write)
        | (ConnectMode::Write { .. }, ConnectionType::Read) => {
            unreachable!("Logic error in database connection writable checks");
        },
    };

    if migrated {
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

    Ok(ConnectResult { pool, ty, migrated })
}
