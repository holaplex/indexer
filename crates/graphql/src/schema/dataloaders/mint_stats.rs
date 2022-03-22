use indexer_core::db::queries::mint_stats;
use objects::{auction_house::AuctionHouse, mint_stats::MintStats};
use scalars::PublicKey;

use super::prelude::*;

#[async_trait]
impl TryBatchFn<PublicKey<AuctionHouse>, Option<MintStats>> for Batcher {
    async fn load(
        &mut self,
        addresses: &[PublicKey<AuctionHouse>],
    ) -> TryBatchMap<PublicKey<AuctionHouse>, Option<MintStats>> {
        let db = self.db()?;
        let rows = mint_stats::load(&db, addresses)?;

        Ok(rows
            .into_iter()
            .map(|s| (s.auction_house.clone(), s.try_into()))
            .batch(addresses))
    }
}
