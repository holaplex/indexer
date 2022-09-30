use std::str::FromStr;

use indexer_core::{
    db::{
        custom_types::OfferEventLifecycleEnum,
        insert_into,
        models::{
            AuctionHouse, CurrentMetadataOwner, FeedEventWallet, Offer as Dboffer, OfferEvent,
            RewardsOffer as DbRewardsOffer,
        },
        mutations, select,
        tables::{
            auction_houses, current_metadata_owners, feed_event_wallets, feed_events, metadatas,
            offer_events, offers, purchases, reward_centers, rewards_offers,
        },
    },
    prelude::*,
    pubkeys, util,
    uuid::Uuid,
};
use mpl_auction_house::pda::find_auctioneer_trade_state_address;
use mpl_reward_center::state::Offer;
use solana_program::pubkey::Pubkey;

use super::super::Client;
use crate::prelude::*;

#[allow(clippy::too_many_lines)]
pub(crate) async fn process(
    client: &Client,
    key: Pubkey,
    account_data: Offer,
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
            .context("Price is too big to store")?,
        token_size: account_data
            .token_size
            .try_into()
            .context("Token size is too big to store")?,
        created_at: util::unix_timestamp(account_data.created_at)?,
        canceled_at: account_data
            .canceled_at
            .map(util::unix_timestamp)
            .transpose()?,
        purchase_ticket: account_data.purchase_ticket.map(|p| Owned(p.to_string())),
        bump: account_data.bump.try_into()?,
        slot: slot.try_into()?,
        write_version: write_version.try_into()?,
    };

    client
        .db()
        .run({
            let values = row.clone();
            move |db| {
                insert_into(rewards_offers::table)
                    .values(&values)
                    .on_conflict(rewards_offers::address)
                    .do_update()
                    .set(&values)
                    .execute(db)
            }
        })
        .await
        .context("Failed to insert rewards offer")?;

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
                    &account_data.buyer,
                    &Pubkey::from_str(&auction_houses.address)?,
                    &Pubkey::from_str(&current_metadata_owner.token_account_address)?,
                    &Pubkey::from_str(&auction_houses.treasury_mint)?,
                    &Pubkey::from_str(&current_metadata_owner.mint_address)?,
                    account_data.token_size,
                );

                let purchase_id = purchases::table
                    .filter(
                        purchases::buyer
                            .eq(row.buyer.clone())
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

                let offer = Dboffer {
                    id: None,
                    trade_state: Owned(bs58::encode(trade_state).into_string()),
                    auction_house: auction_houses.address,
                    marketplace_program: Owned(pubkeys::REWARD_CENTER.to_string()),
                    buyer: row.buyer.clone(),
                    metadata: row.metadata.clone(),
                    token_account: Some(current_metadata_owner.token_account_address),
                    purchase_id,
                    price: row.price,
                    token_size: row.token_size,
                    trade_state_bump: trade_state_bump.try_into()?,
                    created_at: row.created_at,
                    canceled_at: row.canceled_at,
                    slot: row.slot,
                    write_version: Some(row.write_version),
                    expiry: None,
                };

                let offer_exists = select(exists(
                    offers::table.filter(
                        offers::trade_state
                            .eq(trade_state.to_string())
                            .and(offers::metadata.eq(row.metadata.clone())),
                    ),
                ))
                .get_result::<bool>(db)?;

                let offer_id = mutations::offer::insert(db, &offer)?;

                if offer_exists || row.purchase_ticket.is_some() {
                    return Ok(());
                }

                db.build_transaction().read_write().run(|| {
                    let metadata_owner: String =
                        current_metadata_owners::table
                            .inner_join(metadatas::table.on(
                                metadatas::mint_address.eq(current_metadata_owners::mint_address),
                            ))
                            .filter(metadatas::address.eq(row.metadata.clone()))
                            .select(current_metadata_owners::owner_address)
                            .first(db)?;

                    let feed_event_id = insert_into(feed_events::table)
                        .default_values()
                        .returning(feed_events::id)
                        .get_result::<Uuid>(db)
                        .context("Failed to insert feed event")?;

                    insert_into(offer_events::table)
                        .values(&OfferEvent {
                            feed_event_id,
                            lifecycle: OfferEventLifecycleEnum::Created,
                            offer_id,
                        })
                        .execute(db)
                        .context("failed to insert offer created event")?;

                    insert_into(feed_event_wallets::table)
                        .values(&FeedEventWallet {
                            wallet_address: row.buyer,
                            feed_event_id,
                        })
                        .execute(db)
                        .context("Failed to insert offer feed event wallet for buyer")?;

                    insert_into(feed_event_wallets::table)
                        .values(&FeedEventWallet {
                            wallet_address: Owned(metadata_owner),
                            feed_event_id,
                        })
                        .execute(db)
                        .context("Failed to insert offer feed event wallet for metadata owner")?;

                    Result::<_>::Ok(())
                })
            }
        })
        .await
        .context("Failed to insert rewards offer")?;

    Ok(())
}
