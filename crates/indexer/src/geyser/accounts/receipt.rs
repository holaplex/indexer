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
    };

    let values = row.clone();

    let feed_event = client
        .db()
        .run(move |db| {
            let listing_receipt_exists = select(exists(
                listing_receipts::table.filter(listing_receipts::address.eq(row.address.clone())),
            ))
            .get_result::<bool>(db);

            insert_into(listing_receipts::table)
                .values(&row)
                .on_conflict(listing_receipts::address)
                .do_update()
                .set(&row)
                .execute(db)?;

            if Ok(true) == listing_receipt_exists || row.purchase_receipt.is_some() {
                return Ok(None);
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
                        listing_receipt_address: row.address,
                    })
                    .execute(db);

                if Err(DbError::RollbackTransaction) == listing_event {
                    return Ok(None);
                }

                insert_into(feed_event_wallets::table)
                    .values(&FeedEventWallet {
                        wallet_address: row.seller,
                        feed_event_id,
                    })
                    .execute(db)
                    .context("Failed to insert listing feed event wallet")?;

                Result::<_>::Ok(Some(feed_event_id))
            })
        })
        .await
        .context("Failed to insert listing receipt!")?;

    if feed_event.is_none() {
        upsert_into_listings_table(client, values)
            .await
            .context("Failed to insert listing")?;
    }

    Ok(())
}

pub(crate) async fn process_purchase_receipt(
    client: &Client,
    key: Pubkey,
    purchase: PurchaseReceipt,
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
    };

    let values = row.clone();

    let feed_event = client
        .db()
        .run(move |db| {
            let purchase_receipt_exists = select(exists(
                purchase_receipts::table.filter(purchase_receipts::address.eq(row.address.clone())),
            ))
            .get_result::<bool>(db);

            insert_into(purchase_receipts::table)
                .values(&row)
                .on_conflict(purchase_receipts::address)
                .do_update()
                .set(&row)
                .execute(db)?;

            if Ok(true) == purchase_receipt_exists {
                return Ok(None);
            }

            db.build_transaction().read_write().run(|| {
                let feed_event_id = insert_into(feed_events::table)
                    .default_values()
                    .returning(feed_events::id)
                    .get_result::<Uuid>(db)
                    .context("Failed to insert feed event")?;

                insert_into(purchase_events::table)
                    .values(&PurchaseEvent {
                        feed_event_id,
                        purchase_receipt_address: row.address,
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

                Result::<_>::Ok(Some(feed_event_id))
            })
        })
        .await
        .context("Failed to insert purchase receipt!")?;

    if feed_event.is_none() {
        upsert_into_purchases_table(client, values)
            .await
            .context("Failed to insert purchase")?;
    }

    Ok(())
}

pub(crate) async fn process_bid_receipt(
    client: &Client,
    key: Pubkey,
    bid_receipt: BidReceipt,
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
    };

    let values = row.clone();

    let offer_event = client
        .db()
        .run(move |db| {
            let bid_receipt_exists = select(exists(
                bid_receipts::table.filter(bid_receipts::address.eq(row.address.clone())),
            ))
            .get_result::<bool>(db);

            insert_into(bid_receipts::table)
                .values(&row)
                .on_conflict(bid_receipts::address)
                .do_update()
                .set(&row)
                .execute(db)?;

            if Ok(true) == bid_receipt_exists || row.purchase_receipt.is_some() {
                return Ok(None);
            }

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
                        bid_receipt_address: row.address,
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
    } else {
        upsert_into_offers_table(client, values)
            .await
            .context("failed to insert into offers table")?;

        trace!("Skipping Dialect dispatch for offer");
    }

    Ok(())
}

async fn upsert_into_offers_table<'a>(client: &Client, data: DbBidReceipt<'static>) -> Result<()> {
    let row = Offer {
        id: None,
        trade_state: data.trade_state,
        auction_house: data.auction_house,
        buyer: data.buyer,
        metadata: data.metadata,
        token_account: data.token_account,
        purchase_id: None,
        price: data.price,
        token_size: data.token_size,
        trade_state_bump: data.trade_state_bump,
        created_at: data.created_at,
        canceled_at: data.canceled_at,
    };

    client
        .db()
        .run(move |db| {
            insert_into(offers::table)
                .values(&row)
                .on_conflict(on_constraint("offers_unique_fields"))
                .do_update()
                .set(&row)
                .execute(db)
        })
        .await
        .context("Failed to insert offer")?;

    Ok(())
}

async fn upsert_into_purchases_table<'a>(
    client: &Client,
    data: DbPurchaseReceipt<'static>,
) -> Result<()> {
    let row = Purchase {
        id: None,
        buyer: data.buyer.clone(),
        seller: data.seller.clone(),
        auction_house: data.auction_house.clone(),
        metadata: data.metadata.clone(),
        token_size: data.token_size,
        price: data.price,
        created_at: data.created_at,
    };

    client
        .db()
        .run(move |db| {
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
            .execute(db)
        })
        .await
        .context("Failed to insert purchase!")?;

    Ok(())
}

async fn upsert_into_listings_table<'a>(
    client: &Client,
    data: DbListingReceipt<'static>,
) -> Result<()> {
    let row = Listing {
        id: None,
        trade_state: data.trade_state.clone(),
        auction_house: data.auction_house.clone(),
        seller: data.seller.clone(),
        metadata: data.metadata.clone(),
        purchase_id: None,
        price: data.price,
        token_size: data.token_size,
        trade_state_bump: data.trade_state_bump,
        created_at: data.created_at,
        canceled_at: None,
    };

    client
        .db()
        .run(move |db| {
            insert_into(listings::table)
                .values(&row)
                .on_conflict(on_constraint("listings_unique_fields"))
                .do_update()
                .set(&row)
                .execute(db)
        })
        .await
        .context("Failed to insert listing!")?;

    Ok(())
}
