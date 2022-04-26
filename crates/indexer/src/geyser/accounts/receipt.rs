use indexer_core::{
    db::{
        custom_types::{ListingEventLifecycleEnum, OfferEventLifecycleEnum},
        insert_into,
        models::{
            BidReceipt as DbBidReceipt, FeedEventWallet, ListingEvent,
            ListingReceipt as DbListingReceipt, OfferEvent, PurchaseEvent,
            PurchaseReceipt as DbPurchaseReceipt,
        },
        select,
        tables::{
            bid_receipts, current_metadata_owners, feed_event_wallets, feed_events, listing_events,
            listing_receipts, metadatas, offer_events, purchase_events, purchase_receipts,
        },
        Error as DbError,
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
        address: Owned(bs58::encode(key).into_string()),
        trade_state: Owned(bs58::encode(listing.trade_state).into_string()),
        bookkeeper: Owned(bs58::encode(listing.bookkeeper).into_string()),
        auction_house: Owned(bs58::encode(listing.auction_house).into_string()),
        seller: Owned(bs58::encode(listing.seller).into_string()),
        metadata: Owned(bs58::encode(listing.metadata).into_string()),
        purchase_receipt: listing
            .purchase_receipt
            .map(|p| Owned(bs58::encode(p).into_string())),
        price: listing.price.try_into()?,
        token_size: listing.token_size.try_into()?,
        bump: listing.bump.into(),
        trade_state_bump: listing.trade_state_bump.into(),
        created_at: util::unix_timestamp(listing.created_at)?,
        canceled_at: listing.canceled_at.map(util::unix_timestamp).transpose()?,
    };

    client
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
                        feed_event_id: Owned(feed_event_id),
                        lifecycle: ListingEventLifecycleEnum::Created,
                        listing_receipt_address: row.address,
                    })
                    .execute(db);

                if Err(DbError::RollbackTransaction) == listing_event {
                    return Ok(());
                }

                insert_into(feed_event_wallets::table)
                    .values(&FeedEventWallet {
                        wallet_address: row.seller,
                        feed_event_id: Owned(feed_event_id),
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
) -> Result<()> {
    let row = DbPurchaseReceipt {
        address: Owned(bs58::encode(key).into_string()),
        bookkeeper: Owned(bs58::encode(purchase.bookkeeper).into_string()),
        buyer: Owned(bs58::encode(purchase.buyer).into_string()),
        seller: Owned(bs58::encode(purchase.seller).into_string()),
        auction_house: Owned(bs58::encode(purchase.auction_house).into_string()),
        metadata: Owned(bs58::encode(purchase.metadata).into_string()),
        token_size: purchase.token_size.try_into()?,
        price: purchase.price.try_into()?,
        bump: purchase.bump.into(),
        created_at: util::unix_timestamp(purchase.created_at)?,
    };

    client
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
                return Ok(());
            }

            db.build_transaction().read_write().run(|| {
                let feed_event_id = insert_into(feed_events::table)
                    .default_values()
                    .returning(feed_events::id)
                    .get_result::<Uuid>(db)
                    .context("Failed to insert feed event")?;

                insert_into(purchase_events::table)
                    .values(&PurchaseEvent {
                        feed_event_id: Owned(feed_event_id),
                        purchase_receipt_address: row.address,
                    })
                    .execute(db)
                    .context("failed to insert purchase created event")?;

                insert_into(feed_event_wallets::table)
                    .values(&FeedEventWallet {
                        wallet_address: row.seller,
                        feed_event_id: Owned(feed_event_id),
                    })
                    .execute(db)
                    .context("Failed to insert purchase feed event wallet for seller")?;

                insert_into(feed_event_wallets::table)
                    .values(&FeedEventWallet {
                        wallet_address: row.buyer,
                        feed_event_id: Owned(feed_event_id),
                    })
                    .execute(db)
                    .context("Failed to insert purchase feed event wallet for buyer")?;

                Result::<_>::Ok(())
            })
        })
        .await
        .context("Failed to insert purchase receipt!")?;

    Ok(())
}

pub(crate) async fn process_bid_receipt(
    client: &Client,
    key: Pubkey,
    bid_receipt: BidReceipt,
) -> Result<()> {
    let row = DbBidReceipt {
        address: Owned(bs58::encode(key).into_string()),
        trade_state: Owned(bs58::encode(bid_receipt.trade_state).into_string()),
        bookkeeper: Owned(bs58::encode(bid_receipt.bookkeeper).into_string()),
        auction_house: Owned(bs58::encode(bid_receipt.auction_house).into_string()),
        buyer: Owned(bs58::encode(bid_receipt.buyer).into_string()),
        metadata: Owned(bs58::encode(bid_receipt.metadata).into_string()),
        token_account: bid_receipt
            .token_account
            .map(|t| Owned(bs58::encode(t).into_string())),
        purchase_receipt: bid_receipt
            .purchase_receipt
            .map(|p| Owned(bs58::encode(p).into_string())),
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

    client
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
                return Ok(());
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
                        feed_event_id: Owned(feed_event_id),
                        lifecycle: OfferEventLifecycleEnum::Created,
                        bid_receipt_address: row.address,
                    })
                    .execute(db)
                    .context("failed to insert offer created event")?;

                insert_into(feed_event_wallets::table)
                    .values(&FeedEventWallet {
                        wallet_address: row.buyer,
                        feed_event_id: Owned(feed_event_id),
                    })
                    .execute(db)
                    .context("Failed to insert offer feed event wallet for buyer")?;

                insert_into(feed_event_wallets::table)
                    .values(&FeedEventWallet {
                        wallet_address: Owned(metadata_owner),
                        feed_event_id: Owned(feed_event_id),
                    })
                    .execute(db)
                    .context("Failed to insert offer feed event wallet for metadata owner")?;

                Result::<_>::Ok(())
            })
        })
        .await
        .context("Failed to insert bid receipt!")?;

    Ok(())
}
