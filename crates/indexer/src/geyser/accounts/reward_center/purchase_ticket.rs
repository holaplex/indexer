use indexer_core::{
    db::{
        insert_into,
        models::{AuctionHouse, CurrentMetadataOwner, RewardsPurchaseTicket as DbRewardsPurchaseTicket, Listing as DbListing},
        mutations,
        tables::{current_metadata_owners, metadatas, reward_centers, rewards_listings},
    },
    prelude::*,
    pubkeys, util,
};
use mpl_auction_house::pda::find_auctioneer_trade_state_address;
use mpl_reward_center::state::PurchaseTicket;
use solana_program::pubkey;

use super::super::Client;
use crate::prelude::*;

pub(crate) async fn process(
    client: &Client,
    key: Pubkey,
    account_data: PurchaseTicket,
    slot: u64,
    write_version: u64,
) -> Result<()> {
    let row = DbRewardsPurchaseTicket {
        address: Owned(bs58::encode(key).into_string()),
        is_initialized: account_data.is_initialized,
        reward_center_address: Owned(bs58::encode(account_data.reward_center).into_string()),
        seller: Owned(bs58::encode(account_data.seller).into_string()),
        metadata: Owned(bs58::encode(account_data.metadata).into_string()),
        price: account_data
            .price
            .try_into()
            .context("Price is too big to store"),
        token_size: account_data
            .token_size
            .try_into()
            .context("Token size is too big to store"),
        created_at: util::unix_timestamp(account_data.created_at)?,
        canceled_at: account_data
            .canceled_at
            .map(util::unix_timestamp)
            .transpose()?,
        purchase_ticket: None,
        bump: account_data.bump,
        slot,
        write_version,
    };

    client
        .db()
        .run(move |db| {
            let auction_house = reward_centers::table
                .select(reward_center::all_columns)
                .filter(reward_centers::address.eq(row.address))
                .first::<AuctionHouse>(db)?;
            let current_metadata_owner = current_metadata_owners::table
                .select(current_metadata_owner::token_account)
                .inner_join(metadatas::table.on(metadatas::metadata_address.eq(row.metadata)))
                .filter(metadata::meta.eq(row.address))
                .first::<CurrentMetadataOwner>(db)?;

            let (trade_state, trade_state_bump) = find_auctioneer_trade_state_address(
                account_data.seller,
                pubkey!(auction_house.address),
                pubkey!(current_metadata_owner.token_account),
                pubkey!(auction_house.treasury_mint),
                pubkey!(current_metadata_owner.mint_address),
                account_data.token_size,
            );

            let listing = DbListing {
                id: None,
                trade_state: Owned(bs58::encode(trade_state).into_string()),
                trade_state_bump,
                auction_house: Owned(auction_house.address),
                metadata: row.metadata,
                token_size: row.token_size,
                marketplace_program: pubkeys::AUCTION_HOUSE,
                purchase_id: None,
                seller: row.seller,
                price: row.price,
                created_at: row.created_at,
                expiry: None,
                canceled_at: row.canceled_at,
                write_version,
                slot,
            };

            let listing_exists = select(exists(
                listings::table.filter(
                    listings::trade_state
                        .eq(listing.trade_state.clone())
                        .and(listings::metadata.eq(listing.metadata.clone())),
                ),
            ))
            .get_result::<bool>(db)?;

            db.build_transaction().read_write().run(|| {
                insert_into(rewards_listings::table)
                    .values(&row)
                    .on_conflict(rewards_listings::address)
                    .do_update()
                    .set(&row)
                    .execute(db);

                let listing_id = mutations::listing::insert(db, &listing)?;

                if listing_exists || || row.purchase_ticket.is_some() {
                    return Ok(());
                }

                mutations::feed_event::insert_listing_event(db, listing_id, row.seller)?;

                Result::Ok(())
            });
        })
        .await
        .context("Failed to insert rewards listing")?;
}
