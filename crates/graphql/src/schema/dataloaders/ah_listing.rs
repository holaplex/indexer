use indexer_core::uuid::Uuid;
use objects::{ah_listing::Listing, nft::Nft};
use scalars::PublicKey;
use tables::{current_metadata_owners, listings, metadatas};

use super::prelude::*;

#[async_trait]
impl TryBatchFn<Uuid, Option<Listing>> for Batcher {
    async fn load(&mut self, addresses: &[Uuid]) -> TryBatchMap<Uuid, Option<Listing>> {
        let conn = self.db()?;

        let rows: Vec<models::Listing> = listings::table
            .select(listings::all_columns)
            .filter(listings::id.eq(any(addresses)))
            .load(&conn)
            .context("Failed to load listings")?;

        Ok(rows
            .into_iter()
            .map(|lr| (lr.id.unwrap(), lr.try_into()))
            .batch(addresses))
    }
}

#[async_trait]
impl TryBatchFn<PublicKey<Nft>, Vec<Listing>> for Batcher {
    async fn load(
        &mut self,
        addresses: &[PublicKey<Nft>],
    ) -> TryBatchMap<PublicKey<Nft>, Vec<Listing>> {
        let conn = self.db()?;

        let rows: Vec<models::Listing> = listings::table
            .inner_join(metadatas::table.on(metadatas::address.eq(listings::metadata)))
            .inner_join(
                current_metadata_owners::table
                    .on(current_metadata_owners::mint_address.eq(metadatas::mint_address)),
            )
            .select(listings::all_columns)
            .filter(listings::canceled_at.is_null())
            .filter(listings::metadata.eq(any(addresses)))
            .load(&conn)
            .context("Failed to load listings")?;

        Ok(rows
            .into_iter()
            .map(|listing| (listing.metadata.clone(), listing.try_into()))
            .batch(addresses))
    }
}
