//! Interface with the indexer database

pub mod models;
pub mod queries;
#[allow(missing_docs)]
mod schema;

pub mod tables {
    //! Diesel schema DSLs

    pub use super::schema::{
        editions, listing_metadatas, listings, master_editions, metadata_creators, metadatas,
        storefronts,
    };
}

pub use diesel::{insert_into, pg::upsert::excluded};
use diesel::{pg, r2d2};

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

/// Create a pooled connection to the Postgres database
///
/// # Errors
/// This function fails if Diesel fails to construct a connection pool or if any
/// pending database migrations fail to run.
pub fn connect(url: impl Into<String>) -> Result<Pool> {
    let url = url.into();
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
