use borsh::BorshDeserialize;
use indexer_core::{
    db::{
        custom_types::ListingEventLifecycleEnum,
        insert_into,
        models::{FeedEventWallet, Listing, ListingEvent, SellInstruction},
        on_constraint, select,
        tables::{
            feed_event_wallets, feed_events, listing_events, listings, purchases, sell_instructions,
        },
        Error as DbError,
    },
    pubkeys,
    uuid::Uuid,
};
use mpl_auction_house::instruction::Sell;

use super::Client;
use crate::prelude::*;

#[allow(clippy::pedantic)]
pub(crate) async fn process(
    client: &Client,
    data: &[u8],
    accounts: &[Pubkey],
    slot: u64,
) -> Result<()> {
    let params = Sell::try_from_slice(data).context("failed to deserialize")?;

    if accounts.len() != 12 {
        debug!("invalid accounts for SellInstruction");
        return Ok(());
    }

    let accts: Vec<_> = accounts.iter().map(ToString::to_string).collect();

    let row = SellInstruction {
        wallet: Owned(accts[0].clone()),
        token_account: Owned(accts[1].clone()),
        metadata: Owned(accts[2].clone()),
        authority: Owned(accts[3].clone()),
        auction_house: Owned(accts[4].clone()),
        auction_house_fee_account: Owned(accts[5].clone()),
        seller_trade_state: Owned(accts[6].clone()),
        free_seller_trader_state: Owned(accts[7].clone()),
        program_as_signer: Owned(accts[10].clone()),
        trade_state_bump: params.trade_state_bump.try_into()?,
        free_trade_state_bump: params.free_trade_state_bump.try_into()?,
        program_as_signer_bump: params.program_as_signer_bump.try_into()?,
        buyer_price: params.buyer_price.try_into()?,
        token_size: params.token_size.try_into()?,
        created_at: Utc::now().naive_utc(),
        slot: slot.try_into()?,
    };

    let values = row.clone();

    let purchase_id = client
        .db()
        .run({
            let row = row.clone();
            move |db| {
                purchases::table
                    .filter(
                        purchases::seller
                            .eq(row.wallet)
                            .and(purchases::auction_house.eq(row.auction_house))
                            .and(purchases::metadata.eq(row.metadata))
                            .and(purchases::price.eq(row.buyer_price))
                            .and(
                                purchases::token_size
                                    .eq(row.token_size)
                                    .and(purchases::slot.eq(row.slot)),
                            ),
                    )
                    .select(purchases::id)
                    .first::<Uuid>(db)
                    .optional()
                    .context("failed to get purchase ids")
            }
        })
        .await?;

    upsert_into_listings_table(client, Listing {
        id: None,
        trade_state: row.seller_trade_state.clone(),
        auction_house: row.auction_house.clone(),
        marketplace_program: Owned(pubkeys::AUCTION_HOUSE.to_string()),
        seller: row.wallet.clone(),
        metadata: row.metadata.clone(),
        purchase_id,
        price: row.buyer_price,
        token_size: row.token_size,
        trade_state_bump: row.trade_state_bump,
        created_at: row.created_at,
        canceled_at: None,
        slot: row.slot,
        write_version: None,
        expiry: None,
    })
    .await
    .context("failed to insert listing!")?;

    client
        .db()
        .run(move |db| {
            insert_into(sell_instructions::table)
                .values(&values)
                .execute(db)
        })
        .await
        .context("failed to insert sell instruction ")?;
    Ok(())
}

pub async fn upsert_into_listings_table<'a>(client: &Client, row: Listing<'static>) -> Result<()> {
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

            let listing_id = insert_into(listings::table)
                .values(&row)
                .on_conflict(on_constraint("listings_unique_fields"))
                .do_update()
                .set(&row)
                .returning(listings::id)
                .get_result::<Uuid>(db)?;

            if listing_exists {
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
        .context("Failed to insert listing!")?;

    Ok(())
}
