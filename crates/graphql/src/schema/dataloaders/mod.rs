pub mod auction_house;
pub mod bid_receipt;
pub mod collection;
pub mod listing;
pub mod listing_receipt;
pub mod nft;
pub mod stats;
pub mod store_creator;
pub mod storefront;

pub(self) mod batcher;
pub mod purchase_receipt;

pub(self) mod prelude {
    pub use async_trait::async_trait;
    pub use dataloader::BatchFn;
    pub use indexer_core::db::Connection;

    #[allow(unused_imports)]
    pub(super) use super::{
        super::prelude::*,
        batcher::{BatchIter, BatchMap, BatchResult, Batcher, TryBatchFn, TryBatchMap},
    };
}

pub use batcher::{BatchResult, Batcher, Error, Loader};
