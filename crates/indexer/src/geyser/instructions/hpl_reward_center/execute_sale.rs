use borsh::BorshDeserialize;
use indexer_core::{
    db::{
        insert_into,
        models::{ExecuteSaleInstruction, FeedEventWallet, Purchase, PurchaseEvent},
        on_constraint, select,
        tables::{
            execute_sale_instructions, feed_event_wallets, feed_events, listings, offers,
            purchase_events, purchases,
        },
        update,
    },
    pubkeys,
    uuid::Uuid,
};
use mpl_auction_house::instruction::ExecuteSale;

use super::Client;
use crate::prelude::*;

#[allow(clippy::pedantic)]
pub(crate) async fn process(
    client: &Client,
    data: &[u8],
    accounts: &[Pubkey],
    slot: u64,
) -> Result<()> {
    let params = ExecuteSale::try_from_slice(data).context("failed to deserialize")?;

    if accounts.len() < 21 {
        debug!("invalid accounts for ExecuteSaleInstruction");
        return Ok(());
    }

    let accts: Vec<String> = accounts.iter().map(ToString::to_string).collect();


    upsert_into_purchases_table(
        client,
        Purchase {
            id: None,
            buyer: row.buyer.clone(),
            seller: row.seller.clone(),
            auction_house: row.auction_house.clone(),
            marketplace_program: Owned(pubkeys::AUCTION_HOUSE.to_string()),
            metadata: row.metadata.clone(),
            token_size: row.token_size,
            price: row.buyer_price,
            created_at: row.created_at,
            slot: row.slot,
            write_version: None,
        },
        accts[13].clone(),
        accts[14].clone(),
    )
    .await
    .context("failed to insert purchase!")?;

    client
        .db()
        .run(move |db| {
            insert_into(execute_sale_instructions::table)
                .values(&row)
                .execute(db)
        })
        .await
        .context("failed to insert execute sale instruction ")?;
    Ok(())
}

pub(crate) async fn upsert_into_purchases_table<'a>(
    client: &Client,
    data: Purchase<'static>,
    buyer_trade_state: String,
    seller_trade_state: String,
) -> Result<()> {
    client
        .db()
        .run(move |db| {
            let purchase_exists = select(exists(
                purchases::table.filter(
                    purchases::buyer
                        .eq(data.buyer.clone())
                        .and(purchases::seller.eq(data.seller.clone()))
                        .and(purchases::auction_house.eq(data.auction_house.clone()))
                        .and(purchases::metadata.eq(data.metadata.clone()))
                        .and(purchases::price.eq(data.price))
                        .and(purchases::token_size.eq(data.token_size)),
                ),
            ))
            .get_result::<bool>(db)?;

            let purchase_id = insert_into(purchases::table)
                .values(&data)
                .on_conflict(on_constraint("purchases_unique_fields"))
                .do_update()
                .set(&data)
                .returning(purchases::id)
                .get_result::<Uuid>(db)?;

            update(
                listings::table.filter(
                    listings::trade_state
                        .eq(seller_trade_state.clone())
                        .and(listings::purchase_id.is_null())
                        .and(listings::canceled_at.is_null()),
                ),
            )
            .set(listings::purchase_id.eq(Some(purchase_id)))
            .execute(db)?;

            update(
                offers::table.filter(
                    offers::trade_state
                        .eq(buyer_trade_state.clone())
                        .and(offers::purchase_id.is_null())
                        .and(offers::canceled_at.is_null()),
                ),
            )
            .set(offers::purchase_id.eq(Some(purchase_id)))
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
                        wallet_address: data.seller,
                        feed_event_id,
                    })
                    .execute(db)
                    .context("Failed to insert purchase feed event wallet for seller")?;

                insert_into(feed_event_wallets::table)
                    .values(&FeedEventWallet {
                        wallet_address: data.buyer,
                        feed_event_id,
                    })
                    .execute(db)
                    .context("Failed to insert purchase feed event wallet for buyer")?;

                Result::<_>::Ok(())
            })
        })
        .await
        .context("Failed to insert purchase!")?;

    Ok(())
}
