#![deny(clippy::suspicious, clippy::style)]
#![warn(clippy::pedantic, clippy::cargo)]

use indexer_core::{init, prelude::*};

fn main() {
    init();

    info!("Hello, world!");
}
