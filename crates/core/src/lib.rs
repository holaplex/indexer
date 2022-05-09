//! Core components for the `holaplex-indexer` family of crates.

#![deny(
    clippy::disallowed_method,
    clippy::suspicious,
    clippy::style,
    missing_debug_implementations,
    missing_copy_implementations
)]
#![warn(clippy::pedantic, clippy::cargo, missing_docs)]
#![feature(iter_intersperse)]

// TODO: #[macro_use] is somewhat deprecated, but diesel still relies on it
#[cfg(feature = "db")]
#[macro_use]
extern crate diesel;
#[cfg(feature = "db")]
#[macro_use]
extern crate diesel_migrations;

pub extern crate chrono;
pub extern crate clap;
pub extern crate num_cpus;
pub extern crate url;
pub extern crate uuid;

#[cfg(feature = "assets")]
pub mod assets;
#[cfg(feature = "db")]
pub mod db;
pub mod error;
pub mod hash;
#[cfg(feature = "meilisearch")]
pub mod meilisearch;
#[cfg(feature = "solana")]
pub mod pubkeys;
pub mod util;

/// Commonly used utilities
pub mod prelude {
    pub use std::{
        borrow::{
            Cow,
            Cow::{Borrowed, Owned},
        },
        time::Duration as StdDuration,
    };

    pub use chrono::{self, prelude::*};
    #[cfg(feature = "db")]
    pub use diesel::{
        debug_query,
        dsl::{any, exists, not},
        expression_methods::*,
        pg::Pg,
        query_dsl::{BelongingToDsl, GroupByDsl, JoinOnDsl, QueryDsl, RunQueryDsl, SaveChangesDsl},
        result::OptionalExtension,
    };
    #[cfg(feature = "db")]
    pub use diesel_full_text_search::{TsQueryExtensions, TsVectorExtensions};
    pub use log::{debug, error, info, trace, warn};

    pub use super::error::prelude::*;
}

use std::{
    net::SocketAddr,
    path::{Path, PathBuf},
};

use anyhow::Context;
use error::Result;

fn dotenv(name: impl AsRef<Path>) -> Result<Option<PathBuf>, dotenv::Error> {
    match dotenv::from_filename(name) {
        Ok(p) => Ok(Some(p)),
        Err(dotenv::Error::Io(e)) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(e) => Err(e),
    }
}

/// Common options for all server crates.
#[derive(Debug, Clone, Copy, clap::Args)]
pub struct ServerOpts {
    /// The address to bind to
    #[clap(long = "addr", default_value = "0.0.0.0:3000", env)]
    address: SocketAddr,

    /// Overrides the port of the provided binding address
    #[clap(short, long, env)]
    port: Option<u16>,
}

impl ServerOpts {
    /// Process and expose the server options
    #[must_use]
    pub fn into_parts(self) -> (SocketAddr,) {
        let Self { mut address, port } = self;

        if let Some(port) = port {
            address.set_port(port);
        }

        (address,)
    }
}

/// Process environment variables, initialize logging, and then execute the
/// provided closure and handle its result before exiting.
///
/// # Panics
/// This function panics if dotenv fails to load a .env file
pub fn run(main: impl FnOnce() -> Result<()>) -> ! {
    [
        ".env.local",
        if cfg!(debug_assertions) {
            ".env.dev"
        } else {
            ".env.prod"
        },
        ".env",
    ]
    .into_iter()
    .try_for_each(|p| {
        dotenv(p)
            .map(|_| ())
            .with_context(|| format!("Failed to load .env file {:?}", p))
    })
    .expect("Failed to load .env files");

    env_logger::builder()
        .filter_level(if cfg!(debug_assertions) {
            log::LevelFilter::Debug
        } else {
            log::LevelFilter::Warn
        })
        .parse_default_env()
        .init();

    std::process::exit(match main() {
        Ok(()) => 0,
        Err(e) => {
            log::error!("{:?}", e);
            1
        },
    });
}
