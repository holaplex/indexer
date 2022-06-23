use objects::{bid_receipt::BidReceipt, nft::Nft};
use scalars::PublicKey;
use tables::{bid_receipts, metadatas};

use super::prelude::*;

#[async_trait]
impl TryBatchFn<PublicKey<Nft>, Vec<BidReceipt>> for Batcher {
    async fn load(
        &mut self,
        addresses: &[PublicKey<Nft>],
    ) -> TryBatchMap<PublicKey<Nft>, Vec<BidReceipt>> {
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
            .map(|br| (br.metadata.clone(), br.try_into()))
            .batch(addresses))
    }
}

#[async_trait]
impl TryBatchFn<PublicKey<BidReceipt>, Option<BidReceipt>> for Batcher {
    async fn load(
        &mut self,
        addresses: &[PublicKey<BidReceipt>],
    ) -> TryBatchMap<PublicKey<BidReceipt>, Option<BidReceipt>> {
        let conn = self.db()?;

        let rows: Vec<models::BidReceipt> = bid_receipts::table
            .select(bid_receipts::all_columns)
            .filter(bid_receipts::address.eq(any(addresses)))
            .load(&conn)
            .context("Failed to load bid receipts")?;

        Ok(rows
            .into_iter()
            .map(|br| (br.address.clone(), br.try_into()))
            .batch(addresses))
    }
}
