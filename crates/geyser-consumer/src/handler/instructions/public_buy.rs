use borsh::BorshDeserialize;
use indexer::prelude::*;
use indexer_core::{
    db::{
        custom_types::ActivityTypeEnum,
        insert_into,
        models::{Offer, PublicBuyInstruction},
        mutations,
        tables::{offers, public_buy_instructions},
    },
    pubkeys,
};
use mpl_auction_house::instruction::PublicBuy;

use super::Client;

pub(crate) async fn process(
    client: &Client,
    data: &[u8],
    accounts: &[Pubkey],
    slot: u64,
) -> Result<()> {
    let params = PublicBuy::try_from_slice(data).context("failed to deserialize")?;

    let accts: Vec<_> = accounts.iter().map(ToString::to_string).collect();

    let row = PublicBuyInstruction {
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

    upsert_into_offers_table(client, row.clone())
        .await
        .context("failed to insert offer")?;

    client
        .db()
        .run(move |db| {
            insert_into(public_buy_instructions::table)
                .values(&row)
                .execute(db)
        })
        .await
        .context("failed to insert public buy instruction ")?;
    Ok(())
}

async fn upsert_into_offers_table<'a>(
    client: &Client,
    data: PublicBuyInstruction<'static>,
) -> Result<()> {
    let row = Offer {
        id: None,
        trade_state: data.buyer_trade_state,
        auction_house: data.auction_house,
        marketplace_program: Owned(pubkeys::AUCTION_HOUSE.to_string()),
        buyer: data.wallet,
        metadata: data.metadata,
        token_account: Some(data.token_account),
        purchase_id: None,
        price: data.buyer_price,
        token_size: data.token_size,
        trade_state_bump: data.trade_state_bump,
        created_at: data.created_at,
        canceled_at: Some(None),
        slot: data.slot,
        write_version: None,
        expiry: None,
    };

    client
        .db()
        .run(move |db| {
            let auction_house: Pubkey = row.auction_house.to_string().parse()?;

            let indexed_offer_slot: Option<i64> = offers::table
                .filter(
                    offers::trade_state
                        .eq(row.trade_state.clone())
                        .and(offers::metadata.eq(row.metadata.clone())),
                )
                .select(offers::slot)
                .first(db)
                .optional()?;

            let offer_id = mutations::offer::insert(db, &row)?;

            if Some(row.slot) == indexed_offer_slot
                || auction_house == pubkeys::OPENSEA_AUCTION_HOUSE
            {
                return Ok(());
            }

            mutations::activity::offer(db, offer_id, &row.clone(), ActivityTypeEnum::OfferCreated)?;

            Result::<_>::Ok(())
        })
        .await
        .context("Failed to insert offer!")?;

    Ok(())
}
