use objects::{marketplace::Marketplace, store_creator::StoreCreator};
use scalars::PublicKey;
use tables::store_creators;

use super::prelude::*;

#[async_trait]
impl TryBatchFn<PublicKey<Marketplace>, Vec<StoreCreator>> for Batcher {
    async fn load(&mut self, addresses: &[PublicKey<Marketplace>]) -> TryBatchMap<PublicKey<Marketplace>, Vec<StoreCreator>> {
        let conn = self.db()?;

        let rows: Vec<models::StoreCreator> = store_creators::table
            .filter(store_creators::store_config_address.eq(any(addresses)))
            .load(&conn)
            .context("Failed to load store creator")?;

        Ok(rows
            .into_iter()
            .map(|c| (c.store_config_address.clone(), c.try_into()))
            .batch(addresses))
    }
}
