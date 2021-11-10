#![deny(clippy::suspicious, clippy::style)]
#![warn(clippy::pedantic, clippy::cargo)]
#![allow()]

pub mod error;

pub mod prelude {
    pub use log::{debug, error, info, trace, warn};

    pub use super::error::prelude::*;
}

use std::path::{Path, PathBuf};

fn dotenv(name: impl AsRef<Path>) -> Result<Option<PathBuf>, dotenv::Error> {
    match dotenv::from_filename(name) {
        Ok(p) => Ok(Some(p)),
        Err(dotenv::Error::Io(e)) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(e) => Err(e),
    }
}

/// # Panics
/// This function panics if dotenv fails to load a .env file
pub fn init() {
    dotenv(".env")
        .and_then(|_| {
            dotenv(if cfg!(debug_assertions) {
                ".env.local"
            } else {
                ".env.prod"
            })
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
}
