use objects::{
    listing::{Bid, Listing, ListingRow},
    nft::Nft,
};
use strings::ListingAddress;
use tables::{
    auction_caches, auction_datas, auction_datas_ext, bids, listing_metadatas, metadata_jsons,
    metadatas,
};

use super::prelude::*;

#[async_trait]
impl TryBatchFn<ListingAddress, Option<Listing>> for Batcher {
    async fn load(
        &mut self,
        keys: &[ListingAddress],
    ) -> TryBatchMap<ListingAddress, Option<Listing>> {
        let now = Local::now().naive_utc();
        let conn = self.db()?;

        let rows: Vec<ListingRow> = auction_caches::table
            .filter(auction_caches::auction_data.eq(any(keys)))
            .inner_join(
                auction_datas::table.on(auction_caches::auction_data.eq(auction_datas::address)),
            )
            .inner_join(
                auction_datas_ext::table
                    .on(auction_caches::auction_ext.eq(auction_datas_ext::address)),
            )
            .select((
                auction_datas::address,
                auction_caches::store_address,
                auction_datas::ends_at,
                auction_datas_ext::gap_tick_size,
                auction_datas::last_bid_time,
            ))
            .load(&conn)
            .context("Failed to load listings")?;

        Ok(rows
            .into_iter()
            .map(|r| (Listing::address(&r), Listing::new(r, now)))
            .batch(keys))
    }
}

#[async_trait]
impl TryBatchFn<ListingAddress, Vec<Bid>> for Batcher {
    async fn load(&mut self, keys: &[ListingAddress]) -> TryBatchMap<ListingAddress, Vec<Bid>> {
        let conn = self.db()?;

        let rows: Vec<models::Bid> = bids::table
            .filter(bids::listing_address.eq(any(keys)))
            .order_by(bids::last_bid_time.desc())
            .load(&conn)
            .context("Failed to load listing bids")?;

        Ok(rows
            .into_iter()
            .map(|b| (b.listing_address.clone(), b.try_into()))
            .batch(keys))
    }
}

#[async_trait]
impl TryBatchFn<ListingAddress, Vec<Nft>> for Batcher {
    async fn load(&mut self, keys: &[ListingAddress]) -> TryBatchMap<ListingAddress, Vec<Nft>> {
        let conn = self.db()?;

        let rows: Vec<(String, models::Nft)> = listing_metadatas::table
            .filter(listing_metadatas::listing_address.eq(any(keys)))
            .inner_join(
                metadatas::table.on(listing_metadatas::metadata_address.eq(metadatas::address)),
            )
            .inner_join(
                metadata_jsons::table
                    .on(listing_metadatas::metadata_address.eq(metadata_jsons::metadata_address)),
            )
            .select((
                listing_metadatas::listing_address,
                (
                    metadatas::address,
                    metadatas::name,
                    metadatas::seller_fee_basis_points,
                    metadatas::mint_address,
                    metadatas::primary_sale_happened,
                    metadata_jsons::description,
                    metadata_jsons::image,
                ),
            ))
            .load(&conn)
            .context("Failed to load listing NFTs")?;

        Ok(rows.into_iter().map(|(k, v)| (k, v.try_into())).batch(keys))
    }
}