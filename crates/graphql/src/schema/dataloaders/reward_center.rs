use indexer_core::db::tables::reward_centers;

use super::prelude::*;
use crate::schema::{
    objects::{auction_house::AuctionHouse, reward_center::RewardCenter},
    scalars::PublicKey,
};

#[async_trait]
impl TryBatchFn<PublicKey<AuctionHouse>, Option<RewardCenter>> for Batcher {
    async fn load(
        &mut self,
        addresses: &[PublicKey<AuctionHouse>],
    ) -> TryBatchMap<PublicKey<AuctionHouse>, Option<RewardCenter>> {
        let conn = self.db()?;

        let rows: Vec<models::RewardCenter> = reward_centers::table
            .filter(reward_centers::auction_house.eq(any(addresses)))
            .load(&conn)
            .context("Failed to load reward center")?;

        Ok(rows
            .into_iter()
            .map(|h| (h.address.clone(), h.try_into()))
            .batch(addresses))
    }
}
