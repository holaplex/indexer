use objects::storefront::Storefront;
use strings::StorefrontAddress;
use tables::storefronts;

use super::prelude::*;

#[async_trait]
impl TryBatchFn<StorefrontAddress, Option<Storefront>> for Batcher {
    async fn load(
        &mut self,
        keys: &[StorefrontAddress],
    ) -> TryBatchMap<StorefrontAddress, Option<Storefront>> {
        let conn = self.db()?;

        let columns = (
            storefronts::owner_address,
            storefronts::subdomain,
            storefronts::title,
            storefronts::description,
            storefronts::favicon_url,
            storefronts::logo_url,
            storefronts::updated_at,
            storefronts::banner_url,
            storefronts::address,
        );

        let rows: Vec<models::Storefront> = storefronts::table
            .select(columns)
            .filter(storefronts::address.eq(any(keys)))
            .load(&conn)
            .context("Failed to load storefronts")?;

        Ok(rows
            .into_iter()
            .map(|s| (s.address.clone(), s.try_into()))
            .batch(keys))
    }
}
