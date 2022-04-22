use indexer_core::{
    db::{
        insert_into,
        models::{
            BidReceipt as DbBidReceipt, ListingReceipt as DbListingReceipt,
            PurchaseReceipt as DbPurchaseReceipt,
        },
        select,
        tables::{bid_receipts, listing_receipts, purchase_receipts},
    },
    prelude::*,
    util,
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
            insert_into(listing_receipts::table)
                .values(&row)
                .on_conflict(listing_receipts::address)
                .do_update()
                .set(&row)
                .execute(db)
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
            insert_into(purchase_receipts::table)
                .values(&row)
                .on_conflict(purchase_receipts::address)
                .do_update()
                .set(&row)
                .execute(db)
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
                return Result::<_>::Ok(());
            }
            Result::<_>::Ok(())
        })
        .await
        .context("Failed to insert bid receipt!")?;
    client.dispatch_dialect_offer_event(key).await?;
    Ok(())
}
