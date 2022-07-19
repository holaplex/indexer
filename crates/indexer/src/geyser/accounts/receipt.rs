use indexer_core::{
    db::{
        custom_types::{ListingEventLifecycleEnum, OfferEventLifecycleEnum},
        insert_into,
        models::{
            BidReceipt as DbBidReceipt, FeedEventWallet, Listing, ListingEvent,
            ListingReceipt as DbListingReceipt, Offer, OfferEvent, Purchase, PurchaseEvent,
            PurchaseReceipt as DbPurchaseReceipt,
        },
        on_constraint, select,
        tables::{
            bid_receipts, current_metadata_owners, feed_event_wallets, feed_events, listing_events,
            listing_receipts, listings, metadatas, offer_events, offers, purchase_events,
            purchase_receipts, purchases,
        },
        update, Error as DbError,
    },
    prelude::*,
    util,
    uuid::Uuid,
};
use mpl_auction_house::receipt::{BidReceipt, ListingReceipt, PurchaseReceipt};

use super::Client;
use crate::prelude::*;

pub(crate) async fn process_listing_receipt(
    client: &Client,
    key: Pubkey,
    listing: ListingReceipt,
    slot: u64,
    write_version: u64,
) -> Result<()> {
    let row = DbListingReceipt {
        address: Owned(key.to_string()),
        trade_state: Owned(listing.trade_state.to_string()),
        bookkeeper: Owned(listing.bookkeeper.to_string()),
        auction_house: Owned(listing.auction_house.to_string()),
        seller: Owned(listing.seller.to_string()),
        metadata: Owned(listing.metadata.to_string()),
        purchase_receipt: listing.purchase_receipt.map(|p| Owned(p.to_string())),
        price: listing.price.try_into()?,
        token_size: listing.token_size.try_into()?,
        bump: listing.bump.into(),
        trade_state_bump: listing.trade_state_bump.into(),
        created_at: util::unix_timestamp(listing.created_at)?,
        canceled_at: listing.canceled_at.map(util::unix_timestamp).transpose()?,
        slot: slot.try_into()?,
        write_version: write_version.try_into()?,
    };

    client
        .db()
        .run(move |db| {
            let listing_exists = select(exists(
                listings::table.filter(
                    listings::trade_state
                        .eq(row.trade_state.clone())
                        .and(listings::metadata.eq(row.metadata.clone())),
                ),
            ))
            .get_result::<bool>(db)?;

            insert_into(listing_receipts::table)
                .values(&row)
                .on_conflict(listing_receipts::address)
                .do_update()
                .set(&row)
                .execute(db)?;

            let values = Listing {
                id: None,
                trade_state: row.trade_state.clone(),
                auction_house: row.auction_house.clone(),
                seller: row.seller.clone(),
                metadata: row.metadata.clone(),
                purchase_id: None,
                price: row.price,
                token_size: row.token_size,
                trade_state_bump: row.trade_state_bump,
                created_at: row.created_at,
                canceled_at: row.canceled_at,
                slot: row.slot,
                write_version: Some(row.write_version),
            };

            let listing_id = insert_into(listings::table)
                .values(&values)
                .on_conflict(on_constraint("listings_unique_fields"))
                .do_update()
                .set(&values)
                .returning(listings::id)
                .get_result::<Uuid>(db)?;

            if listing_exists || row.purchase_receipt.is_some() {
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
        })
        .await
        .context("Failed to insert listing receipt!")?;

    Ok(())
}

pub(crate) async fn process_purchase_receipt(
    client: &Client,
    key: Pubkey,
    purchase: PurchaseReceipt,
    slot: u64,
    write_version: u64,
) -> Result<()> {
    let row = DbPurchaseReceipt {
        address: Owned(key.to_string()),
        bookkeeper: Owned(purchase.bookkeeper.to_string()),
        buyer: Owned(purchase.buyer.to_string()),
        seller: Owned(purchase.seller.to_string()),
        auction_house: Owned(purchase.auction_house.to_string()),
        metadata: Owned(purchase.metadata.to_string()),
        token_size: purchase.token_size.try_into()?,
        price: purchase.price.try_into()?,
        bump: purchase.bump.into(),
        created_at: util::unix_timestamp(purchase.created_at)?,
        slot: slot.try_into()?,
        write_version: write_version.try_into()?,
    };

    let purchase_exists = client
        .db()
        .run({
            let row = row.clone();
            move |db| {
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

                insert_into(purchase_receipts::table)
                    .values(&row)
                    .on_conflict(purchase_receipts::address)
                    .do_update()
                    .set(&row)
                    .execute(db)?;

                Result::<bool>::Ok(purchase_exists)
            }
        })
        .await
        .context("failed to check if purchase receipt exists!")?;

    let purchase_id = upsert_into_purchases_table(client, row.clone()).await?;

    if purchase_exists {
        return Ok(());
    }

    client
        .db()
        .run(move |db| {
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
        .context("failed to insert purchase event")?;

    Ok(())
}

pub(crate) async fn process_bid_receipt(
    client: &Client,
    key: Pubkey,
    bid_receipt: BidReceipt,
    slot: u64,
    write_version: u64,
) -> Result<()> {
    let row = DbBidReceipt {
        address: Owned(key.to_string()),
        trade_state: Owned(bid_receipt.trade_state.to_string()),
        bookkeeper: Owned(bid_receipt.bookkeeper.to_string()),
        auction_house: Owned(bid_receipt.auction_house.to_string()),
        buyer: Owned(bid_receipt.buyer.to_string()),
        metadata: Owned(bid_receipt.metadata.to_string()),
        token_account: bid_receipt.token_account.map(|t| Owned(t.to_string())),
        purchase_receipt: bid_receipt.purchase_receipt.map(|p| Owned(p.to_string())),
        price: bid_receipt.price.try_into()?,
        token_size: bid_receipt.token_size.try_into()?,
        bump: bid_receipt.bump.into(),
        trade_state_bump: bid_receipt.trade_state_bump.into(),
        created_at: util::unix_timestamp(bid_receipt.created_at)?,
        canceled_at: bid_receipt
            .canceled_at
            .map(util::unix_timestamp)
            .transpose()?,
        slot: slot.try_into()?,
        write_version: write_version.try_into()?,
    };

    let offer_exists = client
        .db()
        .run({
            let row = row.clone();
            move |db| {
                let offer_exists = select(exists(
                    offers::table.filter(
                        offers::trade_state
                            .eq(row.trade_state.clone())
                            .and(offers::metadata.eq(row.metadata.clone())),
                    ),
                ))
                .get_result::<bool>(db)?;

                insert_into(bid_receipts::table)
                    .values(&row)
                    .on_conflict(bid_receipts::address)
                    .do_update()
                    .set(&row)
                    .execute(db)?;

                Result::<bool>::Ok(offer_exists)
            }
        })
        .await
        .context("failed to insert bid reciept")?;

    let offer_id = upsert_into_offers_table(client, row.clone())
        .await
        .context("failed to insert offer")?;

    if offer_exists || row.purchase_receipt.is_some() {
        return Ok(());
    }

    let offer_event = client
        .db()
        .run(move |db| {
            db.build_transaction().read_write().run(|| {
                let metadata_owner: String = current_metadata_owners::table
                    .inner_join(
                        metadatas::table
                            .on(metadatas::mint_address.eq(current_metadata_owners::mint_address)),
                    )
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

                Result::<_>::Ok(Some(feed_event_id))
            })
        })
        .await
        .context("Failed to insert bid receipt!")?;

    if offer_event.is_some() {
        client
            .dispatch_dialect_offer_event(key, bid_receipt.metadata)
            .await?;
    }

    Ok(())
}

async fn upsert_into_offers_table<'a>(client: &Client, row: DbBidReceipt<'static>) -> Result<Uuid> {
    let values = Offer {
        id: None,
        trade_state: row.trade_state,
        auction_house: row.auction_house,
        buyer: row.buyer,
        metadata: row.metadata,
        token_account: row.token_account,
        purchase_id: None,
        price: row.price,
        token_size: row.token_size,
        trade_state_bump: row.trade_state_bump,
        created_at: row.created_at,
        canceled_at: row.canceled_at,
        slot: row.slot,
        write_version: Some(row.write_version),
    };

    let offer_id = client
        .db()
        .run({
            move |db| {
                let offer_id = insert_into(offers::table)
                    .values(&values)
                    .on_conflict(on_constraint("offers_unique_fields"))
                    .do_update()
                    .set(&values)
                    .returning(offers::id)
                    .get_result::<Uuid>(db)?;

                Result::<_>::Ok(offer_id)
            }
        })
        .await
        .context("failed to insert purchase")?;

    Ok(offer_id)
}

async fn upsert_into_purchases_table<'a>(
    client: &Client,
    row: DbPurchaseReceipt<'static>,
) -> Result<Uuid> {
    let row = Purchase {
        id: None,
        buyer: row.buyer.clone(),
        seller: row.seller.clone(),
        auction_house: row.auction_house.clone(),
        metadata: row.metadata.clone(),
        token_size: row.token_size,
        price: row.price,
        created_at: row.created_at,
        slot: row.slot,
        write_version: Some(row.write_version),
    };
    let purchase_id = client
        .db()
        .run({
            move |db| {
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
                            .and(offers::token_size.eq(row.token_size))
                            .and(offers::price.eq(row.price))
                            .and(offers::purchase_id.is_null())
                            .and(offers::canceled_at.is_null()),
                    ),
                )
                .set(offers::purchase_id.eq(Some(purchase_id)))
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
                .set(listings::purchase_id.eq(Some(purchase_id)))
                .execute(db)?;

                Result::<_>::Ok(purchase_id)
            }
        })
        .await
        .context("failed to insert purchase")?;

    Ok(purchase_id)
}
