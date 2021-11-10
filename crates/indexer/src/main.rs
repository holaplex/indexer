#![deny(clippy::suspicious, clippy::style)]
#![warn(clippy::pedantic, clippy::cargo)]

use indexer_core::init;
use indexer_core::prelude::*;

fn main() {
    init();

    info!("Hello, world!");
}
