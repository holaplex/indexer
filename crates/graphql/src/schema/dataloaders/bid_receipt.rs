use objects::bid_receipt::BidReceipt;
use strings::MetadataAddress;
use tables::{bid_receipts, metadatas};

use super::prelude::*;

#[async_trait]
impl TryBatchFn<MetadataAddress, Vec<BidReceipt>> for Batcher {
    async fn load(
        &mut self,
        addresses: &[MetadataAddress],
    ) -> TryBatchMap<MetadataAddress, Vec<BidReceipt>> {
        let conn = self.db()?;

        let rows: Vec<models::BidReceipt> = bid_receipts::table
            .inner_join(metadatas::table.on(metadatas::address.eq(bid_receipts::metadata)))
            .select(bid_receipts::all_columns)
            .filter(bid_receipts::canceled_at.is_null())
            .filter(bid_receipts::purchase_receipt.is_null())
            .filter(bid_receipts::metadata.eq(any(addresses)))
            .load(&conn)
            .context("Failed to load bid receipts")?;

        Ok(rows
            .into_iter()
            .map(|br: models::BidReceipt| (br.metadata.clone(), BidReceipt::try_from(br)))
            .batch(addresses))
    }
}
