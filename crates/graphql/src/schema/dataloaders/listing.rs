use objects::{
    listing::{Bid, Listing, ListingRow},
    nft::Nft,
};
use tables::{
    auction_caches, auction_datas, auction_datas_ext, bids, listing_metadatas, metadata_jsons,
    metadatas,
};

use super::prelude::*;

pub struct ListingBatcher {
    pub db_pool: Arc<Pool>,
}

#[async_trait]
impl BatchFn<String, Option<Listing>> for ListingBatcher {
    async fn load(&mut self, keys: &[String]) -> HashMap<String, Option<Listing>> {
        let now = Local::now().naive_utc();

        let conn = self.db_pool.get().unwrap();
        let mut hash_map = HashMap::new();

        for key in keys {
            hash_map.insert(key.clone(), None);
        }

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
            .unwrap();

        for listing in rows {
            let listing = Listing::new(listing, now)
                .map_err(|e| error!("Failed to load listing: {:?}", e))
                .ok();

            listing.map(|l| hash_map.insert(l.address.clone(), Some(l)));
        }

        hash_map
    }
}

pub struct ListingBidsBatcher {
    pub db_pool: Arc<Pool>,
}

#[async_trait]
impl BatchFn<String, Vec<Bid>> for ListingBidsBatcher {
    async fn load(&mut self, keys: &[String]) -> HashMap<String, Vec<Bid>> {
        let conn = self.db_pool.get().unwrap();
        let mut hash_map = HashMap::new();

        for key in keys {
            hash_map.insert(key.clone(), Vec::new());
        }

        let rows: Vec<models::Bid> = bids::table
            .filter(bids::listing_address.eq(any(keys)))
            .order_by(bids::last_bid_time.desc())
            .load(&conn)
            .unwrap();

        rows.into_iter()
            .fold(hash_map, |mut acc, bid: models::Bid| {
                Bid::try_from(bid)
                    .map(|bid| {
                        acc.entry(bid.listing_address.clone()).and_modify(|bids| {
                            bids.push(bid);
                        });
                    })
                    .ok();

                acc
            })
    }
}

pub struct ListingNftsBatcher {
    pub db_pool: Arc<Pool>,
}

#[async_trait]
impl BatchFn<String, Vec<Nft>> for ListingNftsBatcher {
    async fn load(&mut self, keys: &[String]) -> HashMap<String, Vec<Nft>> {
        let conn = self.db_pool.get().unwrap();
        let mut hash_map = HashMap::new();

        for key in keys {
            hash_map.insert(key.clone(), Vec::new());
        }

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
            .unwrap();

        rows.into_iter().fold(
            hash_map,
            |mut acc, (listing_address, nft): (String, models::Nft)| {
                acc.entry(listing_address).and_modify(|nfts| {
                    nfts.push(Nft::from(nft));
                });

                acc
            },
        )
    }
}
