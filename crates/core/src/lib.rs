//! Core components for `metaplex-indexer` and `metaplex-indexer-rpc`.

#![deny(
    clippy::suspicious,
    clippy::style,
    missing_debug_implementations,
    missing_copy_implementations
)]
#![warn(clippy::pedantic, clippy::cargo, missing_docs)]

// TODO: #[macro_use] is somewhat deprecated, but diesel still relies on it
#[macro_use]
extern crate diesel;
#[macro_use]
extern crate diesel_migrations;

pub extern crate chrono;
pub extern crate clap;

pub mod assets;
pub mod db;
pub mod error;
pub mod hash;
pub mod pubkeys;
pub mod util;

/// Commonly used utilities
pub mod prelude {
    pub use std::borrow::{
        Cow,
        Cow::{Borrowed, Owned},
    };

    pub use chrono::{self, prelude::*};
    pub use diesel::{
        dsl::{any, exists, not},
        expression_methods::*,
        query_dsl::{BelongingToDsl, GroupByDsl, JoinOnDsl, QueryDsl, RunQueryDsl, SaveChangesDsl},
    };
    pub use diesel_full_text_search::{TsQueryExtensions, TsVectorExtensions};
    pub use log::{debug, error, info, trace, warn};

    pub use super::error::prelude::*;
}

use std::path::{Path, PathBuf};

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
#[derive(Debug, Clone, Copy, clap::Parser)]
pub struct ServerOpts {
    /// The port to listen on
    #[clap(short, long, default_value_t = 3000, env)]
    pub port: u16,
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
