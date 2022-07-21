use indexer_core::db::queries::metadatas::NFT_COLUMNS;
use objects::{
    listing::{Bid, Listing, ListingColumns, ListingRow},
    nft::Nft,
};
use scalars::PublicKey;
use tables::{
    auction_caches, auction_datas, auction_datas_ext, bids, current_metadata_owners,
    listing_metadatas, metadata_jsons, metadatas,
};

use super::prelude::*;

#[async_trait]
impl TryBatchFn<PublicKey<Listing>, Option<Listing>> for Batcher {
    async fn load(
        &mut self,
        keys: &[PublicKey<Listing>],
    ) -> TryBatchMap<PublicKey<Listing>, Option<Listing>> {
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
            .select(ListingColumns::default())
            .load(&conn)
            .context("Failed to load listings")?;

        Ok(rows
            .into_iter()
            .map(|r| (Listing::address(&r), Listing::new(r, now)))
            .batch(keys))
    }
}

#[async_trait]
impl TryBatchFn<PublicKey<Listing>, Vec<Bid>> for Batcher {
    async fn load(
        &mut self,
        keys: &[PublicKey<Listing>],
    ) -> TryBatchMap<PublicKey<Listing>, Vec<Bid>> {
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
impl TryBatchFn<PublicKey<Listing>, Vec<(usize, Nft)>> for Batcher {
    async fn load(
        &mut self,
        keys: &[PublicKey<Listing>],
    ) -> TryBatchMap<PublicKey<Listing>, Vec<(usize, Nft)>> {
        let conn = self.db()?;

        let rows: Vec<(String, i32, models::Nft)> = listing_metadatas::table
            .filter(listing_metadatas::listing_address.eq(any(keys)))
            .inner_join(
                metadatas::table.on(listing_metadatas::metadata_address.eq(metadatas::address)),
            )
            .inner_join(
                metadata_jsons::table
                    .on(listing_metadatas::metadata_address.eq(metadata_jsons::metadata_address)),
            )
            .inner_join(
                current_metadata_owners::table
                    .on(current_metadata_owners::mint_address.eq(metadatas::mint_address)),
            )
            .select((
                listing_metadatas::listing_address,
                listing_metadatas::metadata_index,
                NFT_COLUMNS,
            ))
            .load(&conn)
            .context("Failed to load listing NFTs")?;

        Ok(rows
            .into_iter()
            .map(|(k, i, v)| (k, usize::try_from(i).and_then(|i| Ok((i, v.try_into()?)))))
            .batch(keys))
    }
}
