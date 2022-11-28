use borsh::BorshDeserialize;
use indexer_core::{
    db::{
        custom_types::{ActivityTypeEnum, OfferEventLifecycleEnum},
        insert_into,
        models::{BuyInstruction, FeedEventWallet, Offer, OfferEvent},
        mutations, select,
        tables::{
            buy_instructions, current_metadata_owners, feed_event_wallets, feed_events, metadatas,
            offer_events, offers, purchases,
        },
    },
    pubkeys,
    uuid::Uuid,
};
use mpl_auction_house::instruction::Buy;

use super::Client;
use crate::prelude::*;

pub(crate) async fn process(
    client: &Client,
    data: &[u8],
    accounts: &[Pubkey],
    slot: u64,
) -> Result<()> {
    let params = Buy::try_from_slice(data).context("failed to deserialize")?;

    if accounts.len() != 14 {
        debug!("invalid accounts for BuyInstruction");
        return Ok(());
    }

    let accts: Vec<_> = accounts.iter().map(ToString::to_string).collect();

    let row = BuyInstruction {
        wallet: Owned(accts[0].clone()),
        payment_account: Owned(accts[1].clone()),
        transfer_authority: Owned(accts[2].clone()),
        treasury_mint: Owned(accts[3].clone()),
        token_account: Owned(accts[4].clone()),
        metadata: Owned(accts[5].clone()),
        escrow_payment_account: Owned(accts[6].clone()),
        authority: Owned(accts[7].clone()),
        auction_house: Owned(accts[8].clone()),
        auction_house_fee_account: Owned(accts[9].clone()),
        buyer_trade_state: Owned(accts[10].clone()),
        trade_state_bump: params.trade_state_bump.try_into()?,
        escrow_payment_bump: params.escrow_payment_bump.try_into()?,
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
                        purchases::buyer
                            .eq(row.wallet.clone())
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

    upsert_into_offers_table(client, Offer {
        id: None,
        trade_state: row.buyer_trade_state,
        auction_house: row.auction_house,
        marketplace_program: Owned(pubkeys::AUCTION_HOUSE.to_string()),
        buyer: row.wallet,
        metadata: row.metadata,
        token_account: Some(row.token_account),
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
    .context("failed to insert offer")?;

    client
        .db()
        .run(move |db| {
            insert_into(buy_instructions::table)
                .values(&values)
                .execute(db)
        })
        .await
        .context("failed to insert buy instruction ")?;
    Ok(())
}

pub async fn upsert_into_offers_table<'a>(client: &Client, data: Offer<'static>) -> Result<()> {
    client
        .db()
        .run(move |db| {
            let offer_exists = select(exists(
                offers::table.filter(
                    offers::trade_state
                        .eq(data.trade_state.clone())
                        .and(offers::metadata.eq(data.metadata.clone())),
                ),
            ))
            .get_result::<bool>(db)?;

            let offer_id = mutations::offer::insert(db, &data)?;

            if offer_exists {
                return Ok(());
            }

            mutations::activity::offer(
                db,
                offer_id,
                &data.clone(),
                ActivityTypeEnum::OfferCreated,
            )?;

            db.build_transaction().read_write().run(|| {
                let metadata_owner: String = current_metadata_owners::table
                    .inner_join(
                        metadatas::table
                            .on(metadatas::mint_address.eq(current_metadata_owners::mint_address)),
                    )
                    .filter(metadatas::address.eq(data.metadata.clone()))
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
                        wallet_address: data.buyer,
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
        })
        .await
        .context("Failed to insert offer")?;

    Ok(())
}
