use indexer_core::{
    db::{
        insert_into,
        models::{AuctionCache, ListingMetadata},
        tables::{auction_caches, listing_metadatas},
    },
    pubkeys::find_auction_data_extended,
};
use metaplex::state::AuctionCache as AuctionCacheAccount;

use crate::{prelude::*, Client};

pub(crate) async fn process(
    client: &Client,
    cache_key: Pubkey,
    cache: AuctionCacheAccount,
) -> Result<()> {
    // let mut acct = client
    //     .get_account(&keys.cache)
    //     .context("Failed to get auction cache")?;

    // let cache = AuctionCacheAccount::from_account_info(&util::account_as_info(
    //     &keys.cache,
    //     false,
    //     false,
    //     &mut acct,
    // ))
    // .context("Failed to parse AuctionCache")?;

    let AuctionCacheAccount {
        metadata,
        auction,
        vault,
        timestamp,
        store,
        auction_manager,
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

    let (auction_ext, _bump) = find_auction_data_extended(auction);
    let address: Cow<str> = Owned(bs58::encode(cache_key).into_string());

    let values = AuctionCache {
        address: address.clone(),
        store_address: Owned(bs58::encode(store).into_string()),
        timestamp: NaiveDateTime::from_timestamp(timestamp, 0),
        auction_data: Owned(bs58::encode(auction).into_string()),
        auction_ext: Owned(bs58::encode(auction_ext).into_string()),
        vault: Owned(bs58::encode(vault).into_string()),
        auction_manager: Owned(bs58::encode(auction_manager).into_string()),
    };

    let metadata_values = metadata
        .into_iter()
        .enumerate()
        .map(|(i, meta)| {
            Ok(ListingMetadata {
                listing_address: address.clone(),
                metadata_address: Owned(bs58::encode(meta).into_string()),
                metadata_index: i
                    .try_into()
                    .context("Metadata index was too big to store")?,
            })
        })
        .collect::<Result<Vec<_>>>()?;

    client
        .db(move |db| {
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
