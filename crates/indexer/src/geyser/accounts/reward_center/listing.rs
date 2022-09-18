use indexer_core::{
    db::{
        insert_into,
        queries,
        models::{
            AuctionHouse, CurrentMetadataOwner, Listing as DbListing,
            RewardsListing as DbRewardsListing,
            RewardCenter,
        },
        mutations,
        tables::{current_metadata_owners, metadatas, reward_centers, auction_houses, rewards_listings},
    },
    prelude::*,
    pubkeys, util,
};
use mpl_auction_house::pda::find_auctioneer_trade_state_address;
use mpl_reward_center::state::Listing;
use solana_program::pubkey;

use super::super::Client;
use crate::prelude::*;

pub(crate) async fn process(
    client: &Client,
    key: Pubkey,
    account_data: Listing,
    slot: u64,
    write_version: u64,
) -> Result<()> {
    let row = DbRewardsListing {
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
        purchase_ticket: accoutn_data.purchase_ticket.map(|p| Owned(p.to_string())),
        bump: account_data.bump,
        slot: slot.try_into()?,
        write_version: write_version.try_into()?,
    };

    client
        .db()
        .run(move |db| {
            let auction_houses = auction_houses::table
                .select(auction_houses::all_columns)
                .inner_join(reward_centers::table.on(auction_houses::table.eq(reward_centers::auction_house)))
                .filter(reward_centers::address.eq(row.auction_house_address))
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
                marketplace_program: pubkeys::REWARD_CENTER,
                purchase_id: None,
                seller: row.seller,
                price: row.price,
                created_at: row.created_at,
                expiry: None,
                canceled_at: row.canceled_at,
                write_version: Some(row.write_version),
                slot: row.slot,
            };

            db.build_transaction().read_write().run(|| {
                insert_into(rewards_listings::table)
                    .values(&row)
                    .on_conflict(rewards_listings::address)
                    .do_update()
                    .set(&row)
                    .execute(db);

                mutations::listing::insert(db, &listing)?;

                Result::Ok(())
            })
        })
        .await
        .context("Failed to insert rewards listing")?;

        Ok(())
}
