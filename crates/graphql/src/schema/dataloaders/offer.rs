use indexer_core::uuid::Uuid;
use objects::{nft::Nft, offer::Offer};
use scalars::PublicKey;
use tables::{metadatas, offers};

use super::prelude::*;

#[async_trait]
impl TryBatchFn<PublicKey<Nft>, Vec<Offer>> for Batcher {
    async fn load(
        &mut self,
        addresses: &[PublicKey<Nft>],
    ) -> TryBatchMap<PublicKey<Nft>, Vec<Offer>> {
        let conn = self.db()?;

        let rows: Vec<models::Offer> = offers::table
            .inner_join(metadatas::table.on(metadatas::address.eq(offers::metadata)))
            .select(offers::all_columns)
            .filter(offers::canceled_at.is_null())
            .filter(offers::purchase_id.is_null())
            .filter(offers::metadata.eq(any(addresses)))
            .load(&conn)
            .context("Failed to load offers")?;

        Ok(rows
            .into_iter()
            .map(|br| (br.metadata.clone(), br.try_into()))
            .batch(addresses))
    }
}

#[async_trait]
impl TryBatchFn<Uuid, Option<Offer>> for Batcher {
    async fn load(&mut self, ids: &[Uuid]) -> TryBatchMap<Uuid, Option<Offer>> {
        let conn = self.db()?;

        let rows: Vec<models::Offer> = offers::table
            .select(offers::all_columns)
            .filter(offers::id.eq(any(ids)))
            .load(&conn)
            .context("Failed to load bid receipts")?;

        Ok(rows
            .into_iter()
            .map(|br| (br.id.unwrap(), br.try_into()))
            .batch(ids))
    }
}
