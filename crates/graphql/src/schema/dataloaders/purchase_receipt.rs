use objects::purchase_receipt::PurchaseReceipt;
use scalars::PublicKey;
use tables::purchase_receipts;

use super::prelude::*;

#[async_trait]
impl TryBatchFn<PublicKey<PurchaseReceipt>, Option<PurchaseReceipt>> for Batcher {
    async fn load(
        &mut self,
        addresses: &[PublicKey<PurchaseReceipt>],
    ) -> TryBatchMap<PublicKey<PurchaseReceipt>, Option<PurchaseReceipt>> {
        let conn = self.db()?;

        let rows: Vec<models::PurchaseReceipt> = purchase_receipts::table
            .select(purchase_receipts::all_columns)
            .filter(purchase_receipts::address.eq(any(addresses)))
            .load(&conn)
            .context("Failed to load purchase receipts")?;

        Ok(rows
            .into_iter()
            .map(|pr| (pr.address.clone(), pr.try_into()))
            .batch(addresses))
    }
}
