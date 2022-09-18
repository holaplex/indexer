use indexer_core::{
    db::{
        insert_into,
        models::{Offer as Dboffer, RewardsOffer as DbRewardsOffer},
        tables::rewards_offers,
        mutations,
    },
    prelude::*,
    pubkeys, util,
};
use mpl_reward_center::state::Offer;
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
    let row = DbRewardsOffer {
        address: Owned(bs58::encode(key).into_string()),
        is_initialized: account_data.is_initialized,
        reward_center_address: Owned(bs58::encode(account_data.reward_center).into_string()),
        buyer: Owned(bs58::encode(account_data.buyer).into_string()),
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

    let auction_houses = auction_houses::table
        .select(auction_houses::all_columns)
        .inner_join(
            reward_centers::table.on(auction_houses::table.eq(reward_centers::auction_house)),
        )
        .filter(reward_centers::address.eq(row.auction_house_address))
        .first::<AuctionHouse>(db)?;
    let current_metadata_owner = current_metadata_owners::table
        .select(current_metadata_owner::token_account)
        .inner_join(metadatas::table.on(metadatas::metadata_address.eq(row.metadata)))
        .filter(metadata::meta.eq(row.address))
        .first::<CurrentMetadataOwner>(db)?;

    let (trade_state, trade_state_bump) = find_auctioneer_trade_state_address(
        account_data.buyer,
        pubkey!(auction_house.address),
        pubkey!(current_metadata_owner.token_account),
        pubkey!(auction_house.treasury_mint),
        pubkey!(current_metadata_owner.mint_address),
        account_data.token_size,
    );

    let offer = Offer {
        id: None,
        trade_state: Owned(bs58::encode(trade_state).into_string()),
        auction_house: Owned(auction_house.address),
        marketplace_program: Owned(pubkeys::REWARD_CENTER.to_string()),
        buyer: row.buyer,
        metadata: row.metadata,
        token_account: current_metadata_owner.token_account,
        purchase_id: None,
        price: row.price,
        token_size: row.token_size,
        trade_state_bump: row.trade_state_bump,
        created_at: row.created_at,
        canceled_at: row.canceled_at,
        slot: row.slot,
        write_version: row.write_version,
        expiry: None,
    };

    client
        .db()
        .run(move |db| {
            db.build_transaction().read_write().run(|| {
                insert_into(rewards_offers::table)
                    .values(&row)
                    .on_conflict(rewards_offers::address)
                    .do_update()
                    .set(&row)
                    .execute(db);

                mutations::offer::insert(db, &offer)?;

                Result::Ok(())
            })
        })
        .await
        .context("Failed to insert rewards offer")?;

    Ok(())
}
