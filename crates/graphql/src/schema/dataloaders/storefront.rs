use objects::storefront::{Storefront, StorefrontColumns};
use scalars::PublicKey;
use tables::storefronts;

use super::prelude::*;

#[async_trait]
impl TryBatchFn<PublicKey<Storefront>, Option<Storefront>> for Batcher {
    async fn load(
        &mut self,
        keys: &[PublicKey<Storefront>],
    ) -> TryBatchMap<PublicKey<Storefront>, Option<Storefront>> {
        let conn = self.db()?;

        let rows: Vec<models::Storefront> = storefronts::table
            .select(StorefrontColumns::default())
            .filter(storefronts::address.eq(any(keys)))
            .load(&conn)
            .context("Failed to load storefronts")?;

        Ok(rows
            .into_iter()
            .map(|s| (s.address.clone(), s.try_into()))
            .batch(keys))
    }
}
