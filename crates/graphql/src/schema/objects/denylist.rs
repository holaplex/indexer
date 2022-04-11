use indexer_core::db::queries::{listing_denylist, store_denylist};
use objects::{listing::Listing, storefront::Storefront};
use scalars::PublicKey;

use super::prelude::*;

#[derive(Debug, Clone, Copy)]
/// Deny-list for Holaplex storefronts and listings
pub struct Denylist;

#[graphql_object(Context = AppContext)]
impl Denylist {
    fn storefronts(&self, ctx: &AppContext) -> FieldResult<Vec<PublicKey<Storefront>>> {
        let db = ctx.db_pool.get().context("Failed to connect to DB")?;

        store_denylist::get_hard_banned(&db)
            .context("Failed to load denylist")
            .map_err(Into::into)
    }

    fn listings(&self, ctx: &AppContext) -> FieldResult<Vec<PublicKey<Listing>>> {
        let db = ctx.db_pool.get().context("Failed to connect to DB")?;

        listing_denylist::get_hard_banned(&db)
            .context("Failed to load denylist")
            .map_err(Into::into)
    }
}
