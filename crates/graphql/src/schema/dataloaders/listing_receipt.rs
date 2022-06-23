use objects::listing_receipt::ListingReceipt;
use scalars::PublicKey;
use tables::listing_receipts;

use super::prelude::*;

#[async_trait]
impl TryBatchFn<PublicKey<ListingReceipt>, Option<ListingReceipt>> for Batcher {
    async fn load(
        &mut self,
        addresses: &[PublicKey<ListingReceipt>],
    ) -> TryBatchMap<PublicKey<ListingReceipt>, Option<ListingReceipt>> {
        let conn = self.db()?;

        let rows: Vec<models::ListingReceipt> = listing_receipts::table
            .select(listing_receipts::all_columns)
            .filter(listing_receipts::address.eq(any(addresses)))
            .load(&conn)
            .context("Failed to load listing receipts")?;

        Ok(rows
            .into_iter()
            .map(|lr| (lr.address.clone(), lr.try_into()))
            .batch(addresses))
    }
}
