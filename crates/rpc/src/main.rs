//! JSONRPC server to read data from `metaplex-indexer`

#![deny(
    clippy::suspicious,
    clippy::style,
    missing_debug_implementations,
    missing_copy_implementations
)]
#![warn(clippy::pedantic, clippy::cargo, missing_docs)]

use indexer_core::prelude::*;

fn main() {
    indexer_core::run(|| {
        info!("Hello, world!");

        Ok(())
    });
}
