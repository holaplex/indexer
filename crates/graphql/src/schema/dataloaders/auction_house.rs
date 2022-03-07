use objects::auction_house::AuctionHouse;
use scalars::PublicKey;
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
