use std::str::FromStr;

use hpl_reward_center::state::Listing;
use indexer_core::{
    db::{
        custom_types::ListingEventLifecycleEnum,
        insert_into,
        models::{
            AuctionHouse, CurrentMetadataOwner, FeedEventWallet, Listing as DbListing,
            ListingEvent, RewardsListing as DbRewardsListing,
        },
        mutations, select,
        tables::{
            auction_houses, current_metadata_owners, feed_event_wallets, feed_events,
            listing_events, listings, metadatas, purchases, reward_centers, rewards_listings,
        },
        Error as DbError,
    },
    prelude::*,
    pubkeys, util,
    uuid::Uuid,
};
use mpl_auction_house::pda::find_auctioneer_trade_state_address;
use solana_program::pubkey::Pubkey;

use super::super::Client;
use crate::prelude::*;

#[allow(clippy::too_many_lines)]
pub(crate) async fn process(
    client: &Client,
    key: Pubkey,
    account_data: Listing,
    slot: u64,
    write_version: u64,
) -> Result<()> {
    let row = DbRewardsListing {
        address: Owned(bs58::encode(key).into_string()),
        reward_center_address: Owned(bs58::encode(account_data.reward_center).into_string()),
        seller: Owned(bs58::encode(account_data.seller).into_string()),
        metadata: Owned(bs58::encode(account_data.metadata).into_string()),
        price: account_data
            .price
            .try_into()
            .context("Price is too big to store")?,
        token_size: account_data
            .token_size
            .try_into()
            .context("Token size is too big to store")?,
        created_at: util::unix_timestamp(account_data.created_at)?,
        closed_at: None,
        purchase_id: None,
        bump: account_data.bump.try_into()?,
        slot: slot.try_into()?,
        write_version: write_version.try_into()?,
    };

    client
        .db()
        .run({
            let values = row.clone();
            move |db| {
                insert_into(rewards_listings::table)
                    .values(&values)
                    .on_conflict(rewards_listings::address)
                    .do_update()
                    .set(&values)
                    .execute(db)
            }
        })
        .await
        .context("Failed to insert rewards listing")?;

    client
        .db()
        .run({
            move |db| {
                let auction_houses = auction_houses::table
                    .select(auction_houses::all_columns)
                    .inner_join(
                        reward_centers::table
                            .on(auction_houses::address.eq(reward_centers::auction_house)),
                    )
                    .filter(reward_centers::address.eq(row.reward_center_address.clone()))
                    .first::<AuctionHouse>(db)?;

                let current_metadata_owner = current_metadata_owners::table
                    .select((
                        current_metadata_owners::mint_address,
                        current_metadata_owners::owner_address,
                        current_metadata_owners::token_account_address,
                        current_metadata_owners::slot,
                    ))
                    .inner_join(
                        metadatas::table
                            .on(metadatas::mint_address.eq(current_metadata_owners::mint_address)),
                    )
                    .filter(metadatas::address.eq(row.metadata.clone()))
                    .first::<CurrentMetadataOwner>(db)?;

                let (trade_state, trade_state_bump) = find_auctioneer_trade_state_address(
                    &account_data.seller,
                    &Pubkey::from_str(&auction_houses.address)?,
                    &Pubkey::from_str(&current_metadata_owner.token_account_address)?,
                    &Pubkey::from_str(&auction_houses.treasury_mint)?,
                    &Pubkey::from_str(&current_metadata_owner.mint_address)?,
                    account_data.token_size,
                );

                let purchase_id = purchases::table
                    .filter(
                        purchases::seller
                            .eq(row.seller.clone())
                            .and(purchases::auction_house.eq(auction_houses.address.clone()))
                            .and(purchases::metadata.eq(row.metadata.clone()))
                            .and(purchases::price.eq(row.price))
                            .and(
                                purchases::token_size
                                    .eq(row.token_size)
                                    .and(purchases::slot.eq(row.slot)),
                            ),
                    )
                    .select(purchases::id)
                    .first::<Uuid>(db)
                    .optional()?;

                let listing = DbListing {
                    id: None,
                    trade_state: Owned(bs58::encode(trade_state).into_string()),
                    trade_state_bump: trade_state_bump.into(),
                    auction_house: auction_houses.address,
                    metadata: row.metadata.clone(),
                    token_size: row.token_size,
                    marketplace_program: Owned(pubkeys::REWARD_CENTER.to_string()),
                    purchase_id,
                    seller: row.seller.clone(),
                    price: row.price,
                    created_at: row.created_at,
                    expiry: None,
                    canceled_at: row.canceled_at,
                    write_version: Some(row.write_version),
                    slot: row.slot,
                };

                let listing_exists = select(exists(
                    listings::table.filter(
                        listings::trade_state
                            .eq(trade_state.to_string())
                            .and(listings::metadata.eq(row.metadata)),
                    ),
                ))
                .get_result::<bool>(db)?;

                let listing_id = mutations::listing::insert(db, &listing)?;

                if listing_exists || row.purchase_ticket.is_some() {
                    return Ok(());
                }

                db.build_transaction().read_write().run(|| {
                    let feed_event_id = insert_into(feed_events::table)
                        .default_values()
                        .returning(feed_events::id)
                        .get_result::<Uuid>(db)
                        .context("Failed to insert feed event")?;

                    let listing_event = insert_into(listing_events::table)
                        .values(&ListingEvent {
                            feed_event_id,
                            lifecycle: ListingEventLifecycleEnum::Created,
                            listing_id,
                        })
                        .execute(db);

                    if Err(DbError::RollbackTransaction) == listing_event {
                        return Ok(());
                    }

                    insert_into(feed_event_wallets::table)
                        .values(&FeedEventWallet {
                            wallet_address: row.seller,
                            feed_event_id,
                        })
                        .execute(db)
                        .context("Failed to insert listing feed event wallet")?;

                    Result::<_>::Ok(())
                })
            }
        })
        .await
        .context("Failed to insert rewards listing")?;

    Ok(())
}
