//! Core components for `metaplex-indexer` and `metaplex-indexer-rpc`.

#![deny(
    clippy::suspicious,
    clippy::style,
    missing_debug_implementations,
    missing_copy_implementations
)]
#![warn(clippy::pedantic, clippy::cargo, missing_docs)]

pub mod error;
pub mod hash;
pub mod pubkeys;

/// Commonly used utilities
pub mod prelude {
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
