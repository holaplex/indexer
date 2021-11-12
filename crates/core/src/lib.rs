#![deny(clippy::suspicious, clippy::style)]
#![warn(clippy::pedantic, clippy::cargo)]

pub mod error;
pub mod hash;
pub mod pubkeys;
mod thread_pool;

pub use thread_pool::ThreadPool;

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

/// # Panics
/// This function panics if dotenv fails to load a .env file
pub fn run(main: impl FnOnce() -> Result<()>) {
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

    match main() {
        Ok(()) => (),
        Err(e) => log::error!("{:?}", e),
    }
}
