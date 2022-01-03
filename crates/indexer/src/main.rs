//! Binary for running the write half of the indexer.

#![deny(
    clippy::suspicious,
    clippy::style,
    missing_debug_implementations,
    missing_copy_implementations
)]
#![warn(clippy::pedantic, clippy::cargo, missing_docs)]

fn main() {
    indexer_core::run(entry::run);
}

mod bits;
mod client;
mod entry;
pub mod util;

pub use client::Client;
pub use entry::{
    AuctionCacheKeys, AuctionKeys, Job, ListingMetadata, RcAuctionKeys, ThreadPoolHandle,
};

mod prelude {
    pub use indexer_core::prelude::*;
    pub use solana_sdk::{bs58, pubkey::Pubkey};
    pub use topograph::prelude::*;
}
