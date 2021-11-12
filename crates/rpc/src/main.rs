#![deny(clippy::suspicious, clippy::style)]
#![warn(clippy::pedantic, clippy::cargo)]

use indexer_core::prelude::*;

fn main() {
    indexer_core::run(|| {
        info!("Hello, world!");

        Ok(())
    });
}
