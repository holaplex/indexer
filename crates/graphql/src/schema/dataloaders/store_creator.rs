use objects::store_creator::StoreCreator;
use strings::StoreConfigAddress;
use tables::store_creators;

use super::prelude::*;

#[async_trait]
impl TryBatchFn<StoreConfigAddress, Vec<StoreCreator>> for Batcher {
    async fn load(
        &mut self,
        addresses: &[StoreConfigAddress],
    ) -> TryBatchMap<StoreConfigAddress, Vec<StoreCreator>> {
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
