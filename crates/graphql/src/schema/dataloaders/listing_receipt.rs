use objects::listing_receipt::ListingReceipt;
use strings::MetadataAddress;
use tables::{listing_receipts, metadatas, token_accounts};

use super::prelude::*;

#[async_trait]
impl TryBatchFn<MetadataAddress, Vec<ListingReceipt>> for Batcher {
    async fn load(
        &mut self,
        addresses: &[MetadataAddress],
    ) -> TryBatchMap<MetadataAddress, Vec<ListingReceipt>> {
        let conn = self.db()?;

        let rows: Vec<models::ListingReceipt> = listing_receipts::table
            .inner_join(metadatas::table.on(metadatas::address.eq(listing_receipts::metadata)))
            .inner_join(
                token_accounts::table.on(token_accounts::mint_address.eq(metadatas::mint_address)),
            )
            .select(listing_receipts::all_columns)
            .filter(token_accounts::amount.eq(1))
            .filter(listing_receipts::canceled_at.is_null())
            .filter(listing_receipts::purchase_receipt.is_null())
            .filter(listing_receipts::metadata.eq(any(addresses)))
            .load(&conn)
            .context("Failed to load listing receipts")?;

        Ok(rows
            .into_iter()
            .map(|listing| (listing.metadata.clone(), listing.try_into()))
            .batch(addresses))
    }
}
