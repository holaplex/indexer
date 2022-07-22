use indexer_core::uuid::Uuid;
use objects::{ah_purchase::Purchase, nft::BaseNft};
use scalars::PublicKey;
use tables::{metadatas, purchases};

use super::prelude::*;

#[async_trait]
impl TryBatchFn<Uuid, Option<Purchase>> for Batcher {
    async fn load(&mut self, addresses: &[Uuid]) -> TryBatchMap<Uuid, Option<Purchase>> {
        let conn = self.db()?;

        let rows: Vec<models::Purchase> = purchases::table
            .select(purchases::all_columns)
            .filter(purchases::id.eq(any(addresses)))
            .load(&conn)
            .context("Failed to load purchase ")?;

        Ok(rows
            .into_iter()
            .map(|pr| (pr.id.unwrap(), pr.try_into()))
            .batch(addresses))
    }
}

#[async_trait]
impl TryBatchFn<PublicKey<BaseNft>, Vec<Purchase>> for Batcher {
    async fn load(
        &mut self,
        addresses: &[PublicKey<BaseNft>],
    ) -> TryBatchMap<PublicKey<BaseNft>, Vec<Purchase>> {
        let conn = self.db()?;

        let rows: Vec<models::Purchase> = purchases::table
            .inner_join(metadatas::table.on(metadatas::address.eq(purchases::metadata)))
            .select(purchases::all_columns)
            .filter(purchases::metadata.eq(any(addresses)))
            .order(purchases::created_at.desc())
            .load(&conn)
            .context("Failed to load purchases")?;

        Ok(rows
            .into_iter()
            .map(|purchase| (purchase.metadata.clone(), purchase.try_into()))
            .batch(addresses))
    }
}
