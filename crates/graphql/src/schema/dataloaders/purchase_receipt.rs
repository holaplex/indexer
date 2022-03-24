use objects::{purchase_receipt::PurchaseReceipt, nft::Nft};
use scalars::PublicKey;
use tables::{purchase_receipts, metadatas, token_accounts};

use super::prelude::*;

#[async_trait]
impl TryBatchFn<PublicKey<Nft>, Vec<PurchaseReceipt>> for Batcher {
    async fn load(
        &mut self,
        addresses: &[PublicKey<Nft>],
    ) -> TryBatchMap<PublicKey<Nft>, Vec<PurchaseReceipt>> {
        let conn = self.db()?;

        let rows: Vec<models::PurchaseReceipt> = purchase_receipts::table
            .inner_join(metadatas::table.on(metadatas::address.eq(purchase_receipts::metadata)))
            .inner_join(
                token_accounts::table.on(token_accounts::mint_address.eq(metadatas::mint_address)),
            )
            .select(purchase_receipts::all_columns)
            .filter(token_accounts::amount.eq(1))
            .filter(purchase_receipts::metadata.eq(any(addresses)))
            .load(&conn)
            .context("Failed to load purchase receipts")?;

        Ok(rows
            .into_iter()
            .map(|purchase| (purchase.metadata.clone(), purchase.try_into()))
            .batch(addresses))
    }
}
