use std::sync::Arc;

use chrono::NaiveDateTime;
use indexer_core::db::{insert_into, models::ListingMetadata, tables::listing_metadatas};
use metaplex::state::AuctionCache;

use crate::{prelude::*, util, AuctionCacheKeys, AuctionKeys, Client, Job, ThreadPoolHandle};

pub fn process(client: &Client, keys: AuctionCacheKeys, handle: ThreadPoolHandle) -> Result<()> {
    let mut acct = client
        .get_account(&keys.cache)
        .context("Failed to get auction cache")?;

    let cache = AuctionCache::from_account_info(&util::account_as_info(
        &keys.cache,
        false,
        false,
        &mut acct,
    ))
    .context("Failed to parse AuctionCache")?;

    let AuctionCache {
        metadata,
        auction,
        auction_manager: manager,
        vault,
        timestamp,
        ..
    } = cache;

    let mut auction_outs = Vec::new();

    for meta in metadata {
        let mut deps = handle.create_node(
            Job::ListingMetadata(crate::ListingMetadata {
                listing: auction,
                metadata: meta,
            }),
            2,
        );

        auction_outs.push(deps.take());
        handle.push_dependency(Job::Metadata(meta), Some(deps.take()));
    }

    handle.push_dependency(
        Job::Auction(Arc::new(AuctionKeys {
            auction,
            manager,
            vault,
            store_owner: keys.store_owner,
            created_at: NaiveDateTime::from_timestamp(timestamp, 0),
        })),
        auction_outs,
    );

    Ok(())
}

pub fn process_listing_metadata(
    client: &Client,
    crate::ListingMetadata { listing, metadata }: crate::ListingMetadata,
    _handle: ThreadPoolHandle,
) -> Result<()> {
    let db = client.db()?;

    insert_into(listing_metadatas::table)
        .values(ListingMetadata {
            listing_address: Owned(bs58::encode(listing).into_string()),
            metadata_address: Owned(bs58::encode(metadata).into_string()),
        })
        .on_conflict((
            listing_metadatas::listing_address,
            listing_metadatas::metadata_address,
        ))
        .do_nothing()
        .execute(&db)
        .context("Failed to insert listing-metadata join")?;

    Ok(())
}
