use indexer_core::db::{queries::nft_count, tables::twitter_handle_name_services};
use objects::store_creator::StoreCreator;
use scalars::{markers::StoreConfig, PublicKey};
use tables::store_creators;

use super::prelude::*;

#[async_trait]
impl TryBatchFn<PublicKey<StoreConfig>, Vec<StoreCreator>> for Batcher {
    async fn load(
        &mut self,
        addresses: &[PublicKey<StoreConfig>],
    ) -> TryBatchMap<PublicKey<StoreConfig>, Vec<StoreCreator>> {
        let conn = self.db()?;

        let rows: Vec<(Option<String>, models::StoreCreator)> = store_creators::table
            .left_join(twitter_handle_name_services::table.on(
                twitter_handle_name_services::wallet_address.eq(store_creators::creator_address),
            ))
            .filter(store_creators::store_config_address.eq(any(addresses)))
            .select((
                twitter_handle_name_services::twitter_handle.nullable(),
                (store_creators::all_columns),
            ))
            .load(&conn)
            .context("Failed to load store creator")?;

        Ok(rows
            .into_iter()
            .map(|c| (c.1.store_config_address.clone(), c.try_into()))
            .batch(addresses))
    }
}

#[async_trait]
impl TryBatchFn<PublicKey<StoreCreator>, Option<i32>> for Batcher {
    async fn load(
        &mut self,
        addresses: &[PublicKey<StoreCreator>],
    ) -> TryBatchMap<PublicKey<StoreCreator>, Option<i32>> {
        let db = self.db()?;
        let rows = nft_count::store_creators(&db, addresses)?;

        Ok(rows
            .into_iter()
            .map(|s| (s.store_creator.clone(), s.nfts.try_into()))
            .batch(addresses))
    }
}
