use indexer_core::db::{insert_into, models::ListingMetadata, tables::listing_metadatas};
use metaplex::state::AuctionCache;

use super::AuctionCacheKeys;
use crate::{prelude::*, util, Client};

pub(super) fn process(client: &Client, keys: AuctionCacheKeys) -> Result<()> {
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
        vault,
        timestamp,
        ..
    } = cache;

    // TODO: store listing metadata
    // let mut auction_outs = Vec::new();

    // for (index, meta) in metadata.into_iter().enumerate() {
    //     let mut deps = handle.create_node(
    //         Job::ListingMetadata(crate::ListingMetadata {
    //             listing: auction,
    //             metadata: meta,
    //             index,
    //         }),
    //         2,
    //     );

    //     auction_outs.push(deps.get_in_edge());
    //     handle.push_dependency(Job::Metadata(meta), Some(deps.get_in_edge()));
    // }

    // TODO: store auction
    // let mut auction = handle.create_node(
    //     Job::Auction(Arc::new(AuctionKeys {
    //         auction,
    //         vault,
    //         store_owner: keys.store_owner,
    //         created_at: NaiveDateTime::from_timestamp(timestamp, 0),
    //     })),
    //     1,
    // );

    // auction
    //     .set_dependents(Dependents::new(auction_outs))
    //     .expect("Failed to sync auction outs - this shouldn't happen!");

    // bid_dependents.lock().push(&handle, auction.get_in_edge());

    Ok(())
}

pub(super) fn process_listing_metadata(
    client: &Client,
    super::ListingMetadata {
        listing,
        metadata,
        index,
    }: super::ListingMetadata,
) -> Result<()> {
    todo!();
    // let db = client.db()?;

    // insert_into(listing_metadatas::table)
    //     .values(ListingMetadata {
    //         listing_address: Owned(bs58::encode(listing).into_string()),
    //         metadata_address: Owned(bs58::encode(metadata).into_string()),
    //         metadata_index: index
    //             .try_into()
    //             .context("Metadata index too big to store")?,
    //     })
    //     .on_conflict((
    //         listing_metadatas::listing_address,
    //         listing_metadatas::metadata_address,
    //     ))
    //     .do_nothing()
    //     .execute(&db)
    //     .context("Failed to insert listing-metadata join")?;

    Ok(())
}
