use indexer_core::db::queries::stats;
use objects::{
    auction_house::AuctionHouse,
    stats::{MarketStats, MintStats},
};
use scalars::{markers::StoreConfig, PublicKey};

use super::prelude::*;
use crate::schema::objects::{stats::StoreCreatorStats, store_creator::StoreCreator};

#[async_trait]
impl TryBatchFn<PublicKey<AuctionHouse>, Option<MintStats>> for Batcher {
    async fn load(
        &mut self,
        addresses: &[PublicKey<AuctionHouse>],
    ) -> TryBatchMap<PublicKey<AuctionHouse>, Option<MintStats>> {
        let db = self.db()?;
        let rows = stats::mint(&db, addresses)?;

        Ok(rows
            .into_iter()
            .map(|s| (s.auction_house.clone(), s.try_into()))
            .batch(addresses))
    }
}

#[async_trait]
impl TryBatchFn<PublicKey<StoreConfig>, Option<MarketStats>> for Batcher {
    async fn load(
        &mut self,
        addresses: &[PublicKey<StoreConfig>],
    ) -> TryBatchMap<PublicKey<StoreConfig>, Option<MarketStats>> {
        let db = self.db()?;
        let rows = stats::marketplace(&db, addresses)?;

        Ok(rows
            .into_iter()
            .map(|s| (s.store_config.clone(), s.try_into()))
            .batch(addresses))
    }
}

#[async_trait]
impl TryBatchFn<PublicKey<StoreCreator>, Option<StoreCreatorStats>> for Batcher {
    async fn load(
        &mut self,
        addresses: &[PublicKey<StoreCreator>],
    ) -> TryBatchMap<PublicKey<StoreCreator>, Option<StoreCreatorStats>> {
        let db = self.db()?;
        let rows = stats::store_creator(&db, addresses)?;

        Ok(rows
            .into_iter()
            .map(|s| (s.store_creator.clone(), s.try_into()))
            .batch(addresses))
    }
}
