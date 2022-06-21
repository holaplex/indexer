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

    let row = ExecuteSaleInstruction {
        buyer: Owned(accts[0].clone()),
        seller: Owned(accts[1].clone()),
        token_account: Owned(accts[2].clone()),
        token_mint: Owned(accts[3].clone()),
        metadata: Owned(accts[4].clone()),
        treasury_mint: Owned(accts[5].clone()),
        escrow_payment_account: Owned(accts[6].clone()),
        seller_payment_receipt_account: Owned(accts[7].clone()),
        buyer_receipt_token_account: Owned(accts[8].clone()),
        authority: Owned(accts[9].clone()),
        auction_house: Owned(accts[10].clone()),
        auction_house_fee_account: Owned(accts[11].clone()),
        auction_house_treasury: Owned(accts[12].clone()),
        buyer_trade_state: Owned(accts[13].clone()),
        seller_trade_state: Owned(accts[14].clone()),
        free_trade_state: Owned(accts[15].clone()),
        program_as_signer: Owned(accts[19].clone()),
        escrow_payment_bump: params.escrow_payment_bump.try_into()?,
        free_trade_state_bump: params._free_trade_state_bump.try_into()?,
        program_as_signer_bump: params.program_as_signer_bump.try_into()?,
        buyer_price: params.buyer_price.try_into()?,
        token_size: params.token_size.try_into()?,
        created_at: Utc::now().naive_utc(),
        slot: slot.try_into()?,
    };

    upsert_into_purchases_table(client, row.clone())
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

async fn upsert_into_purchases_table<'a>(
    client: &Client,
    data: ExecuteSaleInstruction<'static>,
) -> Result<()> {
    let row = Purchase {
        id: None,
        buyer: data.buyer.clone(),
        seller: data.seller.clone(),
        auction_house: data.auction_house.clone(),
        metadata: data.metadata.clone(),
        token_size: data.token_size,
        price: data.buyer_price,
        created_at: data.created_at,
        slot: data.slot,
        write_version: None,
    };

    client
        .db()
        .run(move |db| {
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

            if purchase_exists {
                return Ok(());
            }

            let purchase_id = insert_into(purchases::table)
                .values(&row)
                .on_conflict(on_constraint("purchases_unique_fields"))
                .do_update()
                .set(&row)
                .returning(purchases::id)
                .get_result::<Uuid>(db)?;

            update(
                listings::table.filter(
                    listings::trade_state
                        .eq(data.seller_trade_state.clone())
                        .and(listings::purchase_id.is_null())
                        .and(listings::canceled_at.is_null()),
                ),
            )
            .set(listings::purchase_id.eq(Some(purchase_id)))
            .execute(db)?;

            update(
                offers::table.filter(
                    offers::trade_state
                        .eq(data.buyer_trade_state.clone())
                        .and(offers::purchase_id.is_null())
                        .and(offers::canceled_at.is_null()),
                ),
            )
            .set(offers::purchase_id.eq(Some(purchase_id)))
            .execute(db)?;

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
        .context("Failed to insert purchase!")?;

    Ok(())
}
