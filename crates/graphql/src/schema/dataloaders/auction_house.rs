use indexer_core::db::tables::store_auction_houses;
use objects::auction_house::AuctionHouse;
use scalars::{PublicKey, markers::StoreConfig};
use tables::auction_houses;

use super::prelude::*;

#[async_trait]
impl TryBatchFn<PublicKey<AuctionHouse>, Option<AuctionHouse>> for Batcher {
    async fn load(
        &mut self,
        addresses: &[PublicKey<AuctionHouse>],
    ) -> TryBatchMap<PublicKey<AuctionHouse>, Option<AuctionHouse>> {
        let conn = self.db()?;

        let rows: Vec<models::AuctionHouse> = auction_houses::table
            .filter(auction_houses::address.eq(any(addresses)))
            .load(&conn)
            .context("Failed to load auction houses")?;

        Ok(rows
            .into_iter()
            .map(|h| (h.address.clone(), h.try_into()))
            .batch(addresses))
    }
}


#[async_trait]
impl TryBatchFn<PublicKey<StoreConfig>, Vec<AuctionHouse>> for Batcher {
    async fn load(
        &mut self,
        addresses: &[PublicKey<StoreConfig>],
    ) -> TryBatchMap<PublicKey<StoreConfig>, Vec<AuctionHouse>> {
        let conn = self.db()?;

        let rows: Vec<(String, models::AuctionHouse)> = store_auction_houses::table
            .filter(store_auction_houses::store_config_address.eq(any(addresses)))
            .inner_join(auction_houses::table.on(
                auction_houses::address.eq(store_auction_houses::auction_house_address),
            ))
            .select((
                store_auction_houses::store_config_address,
                (auction_houses::all_columns),
            ))
            .load(&conn)
            .context("Failed to load store creator")?;

        Ok(rows
            .into_iter()
            .map(|c| (c.0.clone(), c.1.try_into()))
            .batch(addresses))
    }
}