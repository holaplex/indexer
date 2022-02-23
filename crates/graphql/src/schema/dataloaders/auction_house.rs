use objects::auction_house::AuctionHouse;
use strings::AuctionHouseAddress;
use tables::auction_houses;

use super::prelude::*;

#[async_trait]
impl TryBatchFn<AuctionHouseAddress, Option<AuctionHouse>> for Batcher {
    async fn load(
        &mut self,
        addresses: &[AuctionHouseAddress],
    ) -> TryBatchMap<AuctionHouseAddress, Option<AuctionHouse>> {
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
