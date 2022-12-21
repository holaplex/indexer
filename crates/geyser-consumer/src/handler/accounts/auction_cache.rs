use indexer_core::{
    db::{
        insert_into,
        models::{AuctionCache, ListingMetadata},
        tables::{auction_caches, listing_metadatas},
    },
    pubkeys::find_auction_data_extended,
    util,
};
use metaplex::state::AuctionCache as AuctionCacheAccount;

use super::Client;
use crate::prelude::*;

pub(crate) async fn process(
    client: &Client,
    cache_key: Pubkey,
    cache: AuctionCacheAccount,
) -> Result<()> {
    let AuctionCacheAccount {
        metadata,
        auction,
        vault,
        timestamp,
        store,
        auction_manager,
        ..
    } = cache;

    let (auction_ext, _bump) = find_auction_data_extended(vault);

    let values = AuctionCache {
        address: Owned(bs58::encode(cache_key).into_string()),
        store_address: Owned(bs58::encode(store).into_string()),
        timestamp: util::unix_timestamp(timestamp)?,
        auction_data: Owned(bs58::encode(auction).into_string()),
        auction_ext: Owned(bs58::encode(auction_ext).into_string()),
        vault: Owned(bs58::encode(vault).into_string()),
        auction_manager: Owned(bs58::encode(auction_manager).into_string()),
    };

    let listing_address: Cow<str> = Owned(bs58::encode(auction).into_string());
    let metadata_values = metadata
        .into_iter()
        .enumerate()
        .map(|(i, meta)| {
            Ok(ListingMetadata {
                listing_address: listing_address.clone(),
                metadata_address: Owned(bs58::encode(meta).into_string()),
                metadata_index: i
                    .try_into()
                    .context("Metadata index was too big to store")?,
            })
        })
        .collect::<Result<Vec<_>>>()?;

    client
        .db()
        .run(move |db| {
            insert_into(auction_caches::table)
                .values(&values)
                .on_conflict(auction_caches::address)
                .do_update()
                .set(&values)
                .execute(db)?;

            metadata_values.into_iter().try_for_each(|v| {
                insert_into(listing_metadatas::table)
                    .values(&v)
                    .on_conflict((
                        listing_metadatas::listing_address,
                        listing_metadatas::metadata_address,
                    ))
                    .do_update()
                    .set(&v)
                    .execute(db)
                    .map(|_| ())
            })
        })
        .await
        .context("Failed to store auction cache data")?;

    Ok(())
}
