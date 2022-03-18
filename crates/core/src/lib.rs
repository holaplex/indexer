//! Core components for the `holaplex-indexer` family of crates.

#![deny(
    clippy::suspicious,
    clippy::style,
    missing_debug_implementations,
    missing_copy_implementations
)]
#![warn(clippy::pedantic, clippy::cargo, missing_docs)]
#![feature(iter_intersperse)]

// TODO: #[macro_use] is somewhat deprecated, but diesel still relies on it
#[macro_use]
extern crate diesel;
#[macro_use]
extern crate diesel_migrations;

pub extern crate chrono;
pub extern crate clap;
pub extern crate tracing;
pub extern crate tracing_futures;
pub extern crate tracing_timing;
pub extern crate url;

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
        debug_query,
        dsl::{any, exists, not},
        expression_methods::*,
        pg::Pg,
        query_dsl::{BelongingToDsl, GroupByDsl, JoinOnDsl, QueryDsl, RunQueryDsl, SaveChangesDsl},
    };
    pub use diesel_full_text_search::{TsQueryExtensions, TsVectorExtensions};
    pub use tracing::{
        self, debug, debug_span, error, error_span, info, info_span, span, trace, trace_span, warn,
        warn_span,
    };
    pub use tracing_futures;
    pub use tracing_timing;

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
#[derive(Debug, Clone, Copy, clap::Parser)]
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
pub fn run<S: tracing::Subscriber + Send + Sync + 'static>(
    subscriber: impl FnOnce() -> S,
    main: impl FnOnce() -> Result<()>,
) -> ! {
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

    tracing_log::env_logger::init();
    tracing::subscriber::set_global_default(subscriber()).unwrap();

    tracing::error_span!("what");

    std::process::exit(match main() {
        Ok(()) => 0,
        Err(e) => {
            log::error!("{:?}", e);
            1
        },
    });
}

/// TODO: ???????
#[cfg(feature = "prometheus")]
#[must_use]
pub fn prometheus_subscriber() -> impl tracing::Subscriber {
    use tracing_subscriber::prelude::*;

    // ???? where is the API intersection between this and tracing_opentelemetry?
    let exporter = opentelemetry_prometheus::exporter().init();

    let telemetry = tracing_opentelemetry::layer();

    tracing_subscriber::registry().with(telemetry)
}

/// Create a tracing subscriber configured fron the `RUST_LOG` env var that
/// prints to stdout
#[must_use]
pub fn env_subscriber() -> impl tracing::Subscriber {
    tracing_subscriber::fmt()
        .pretty()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .finish()
}
