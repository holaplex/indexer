use objects::storefront::{Storefront, StorefrontAddress};
use tables::storefronts;

use super::prelude::*;

pub struct StorefrontBatcher {
    pub db_pool: Arc<Pool>,
}

#[async_trait]
impl BatchFn<StorefrontAddress, Option<Storefront>> for StorefrontBatcher {
    async fn load(
        &mut self,
        keys: &[StorefrontAddress],
    ) -> HashMap<StorefrontAddress, Option<Storefront>> {
        let conn = self.db_pool.get().unwrap();
        let mut hash_map = HashMap::new();

        for key in keys {
            hash_map.insert(key.clone(), None);
        }

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

        let key_strs: Vec<_> = keys.iter().map(|k| &k.0).collect();

        let rows: Vec<models::Storefront> = storefronts::table
            .select(columns)
            .filter(storefronts::address.eq(any(key_strs)))
            .load(&conn)
            .unwrap();

        for storefront in rows {
            let storefront = Storefront::from(storefront);

            hash_map.insert(
                StorefrontAddress(storefront.address.clone()),
                Some(storefront),
            );
        }

        hash_map
    }
}
