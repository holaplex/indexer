use indexer_core::{
    db::{
        insert_into,
        models::{
            AuctionHouse, FeedEventWallet, Purchase as DbPurchase, PurchaseEvent,
            RewardsPurchaseTicket as DbRewardsPurchaseTicket,
        },
        on_constraint, select,
        tables::{
            auction_houses, feed_event_wallets, feed_events, listings, offers, purchase_events,
            purchases, reward_centers, rewards_purchase_tickets,
        },
        update,
    },
    prelude::*,
    pubkeys, util,
    uuid::Uuid,
};
use mpl_reward_center::state::PurchaseTicket;

use super::super::Client;
use crate::prelude::*;

#[allow(clippy::too_many_lines)]
pub(crate) async fn process(
    client: &Client,
    key: Pubkey,
    account_data: PurchaseTicket,
    slot: u64,
    write_version: u64,
) -> Result<()> {
    let row = DbRewardsPurchaseTicket {
        address: Owned(bs58::encode(key).into_string()),
        reward_center_address: Owned(bs58::encode(account_data.reward_center).into_string()),
        seller: Owned(bs58::encode(account_data.seller).into_string()),
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
        slot: slot.try_into()?,
        write_version: write_version.try_into()?,
    };

    let values = row.clone();

    client
        .db()
        .run(move |db| {
            insert_into(rewards_purchase_tickets::table)
                .values(&values)
                .on_conflict(rewards_purchase_tickets::address)
                .do_update()
                .set(&values)
                .execute(db)
        })
        .await
        .context("Failed to insert rewards purchase ticket")?;

    client
        .db()
        .run(move |db| {
            let auction_house = auction_houses::table
                .select(auction_houses::all_columns)
                .inner_join(
                    reward_centers::table
                        .on(auction_houses::address.eq(reward_centers::auction_house)),
                )
                .filter(reward_centers::address.eq(row.reward_center_address))
                .first::<AuctionHouse>(db)?;

            let row = DbPurchase {
                id: None,
                buyer: row.buyer.clone(),
                seller: row.seller.clone(),
                auction_house: auction_house.address,
                marketplace_program: Owned(pubkeys::REWARD_CENTER.to_string()),
                metadata: row.metadata.clone(),
                token_size: row.token_size,
                price: row.price,
                created_at: row.created_at,
                slot: row.slot,
                write_version: Some(row.write_version),
            };

            let purchase_exists = select(exists(
                purchases::table.filter(
                    purchases::buyer
                        .eq(row.buyer.clone())
                        .and(purchases::seller.eq(row.seller.clone()))
                        .and(purchases::auction_house.eq(row.auction_house.clone()))
                        .and(purchases::metadata.eq(row.metadata.clone()))
                        .and(purchases::price.eq(row.price))
                        .and(purchases::token_size.eq(row.token_size)),
                ),
            ))
            .get_result::<bool>(db)?;

            let purchase_id = insert_into(purchases::table)
                .values(&row)
                .on_conflict(on_constraint("purchases_unique_fields"))
                .do_update()
                .set(&row)
                .returning(purchases::id)
                .get_result::<Uuid>(db)?;

            update(
                offers::table.filter(
                    offers::auction_house
                        .eq(row.auction_house.clone())
                        .and(offers::buyer.eq(row.buyer.clone()))
                        .and(offers::metadata.eq(row.metadata.clone()))
                        .and(offers::purchase_id.is_null())
                        .and(offers::canceled_at.is_null()),
                ),
            )
            .set((
                offers::purchase_id.eq(Some(purchase_id)),
                offers::slot.eq(row.slot),
            ))
            .execute(db)?;

            update(
                listings::table.filter(
                    listings::auction_house
                        .eq(row.auction_house.clone())
                        .and(listings::seller.eq(row.seller.clone()))
                        .and(listings::metadata.eq(row.metadata.clone()))
                        .and(listings::price.eq(row.price))
                        .and(listings::token_size.eq(row.token_size))
                        .and(listings::purchase_id.is_null())
                        .and(listings::canceled_at.is_null()),
                ),
            )
            .set((
                listings::purchase_id.eq(Some(purchase_id)),
                listings::slot.eq(row.slot),
            ))
            .execute(db)?;

            if purchase_exists {
                return Ok(());
            }

            db.build_transaction().read_write().run(|| {
                let feed_event_id = insert_into(feed_events::table)
                    .default_values()
                    .returning(feed_events::id)
                    .get_result::<Uuid>(db)
                    .context("Failed to insert feed event")?;

                insert_into(purchase_events::table)
                    .values(PurchaseEvent {
                        purchase_id,
                        feed_event_id,
                    })
                    .execute(db)
                    .context("failed to insert purchase created event")?;

                insert_into(feed_event_wallets::table)
                    .values(&FeedEventWallet {
                        wallet_address: row.seller,
                        feed_event_id,
                    })
                    .execute(db)
                    .context("Failed to insert purchase feed event wallet for seller")?;

                insert_into(feed_event_wallets::table)
                    .values(&FeedEventWallet {
                        wallet_address: row.buyer,
                        feed_event_id,
                    })
                    .execute(db)
                    .context("Failed to insert purchase feed event wallet for buyer")?;

                Result::<_>::Ok(())
            })
        })
        .await
        .context("Failed to insert rewards listing")?;

    Ok(())
}
