pub mod auction_house;
pub mod listing;
pub mod nft;
pub mod storefront;

pub(self) mod batcher;

mod prelude {
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
