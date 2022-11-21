use borsh::BorshDeserialize;
use indexer_core::{
    db::{
        insert_into,
        models::{AcceptOffer, Purchase},
        tables::{accept_offer_ins, rewards_offers},
    },
    pubkeys,
};

use super::super::Client;
use crate::{
    geyser::instructions::hpl_reward_center::buy_listing::upsert_into_purchases_table, prelude::*,
};

#[allow(clippy::pedantic)]
pub(crate) async fn process(
    client: &Client,
    tx_signature: String,
    data: &[u8],
    accounts: &[Pubkey],
    slot: u64,
) -> Result<()> {
    let params = hpl_reward_center::offers::accept::AcceptOfferParams::try_from_slice(data)
        .context("failed to deserialize accept offer params")?;

    let accts: Vec<_> = accounts.iter().map(ToString::to_string).collect();

    let row = AcceptOffer {
        tx_signature: Owned(tx_signature),
        buyer: Owned(accts[0].clone()),
        buyer_reward_token_account: Owned(accts[1].clone()),
        seller: Owned(accts[2].clone()),
        seller_reward_token_account: Owned(accts[3].clone()),
        offer: Owned(accts[4].clone()),
        token_account: Owned(accts[5].clone()),
        token_mint: Owned(accts[6].clone()),
        metadata: Owned(accts[7].clone()),
        treasury_mint: Owned(accts[8].clone()),
        seller_payment_receipt_account: Owned(accts[9].clone()),
        buyer_receipt_token_account: Owned(accts[10].clone()),
        authority: Owned(accts[11].clone()),
        escrow_payment_account: Owned(accts[12].clone()),
        auction_house: Owned(accts[13].clone()),
        auction_house_fee_account: Owned(accts[14].clone()),
        auction_house_treasury: Owned(accts[15].clone()),
        buyer_trade_state: Owned(accts[16].clone()),
        seller_trade_state: Owned(accts[17].clone()),
        free_seller_trade_state: Owned(accts[18].clone()),
        reward_center: Owned(accts[19].clone()),
        reward_center_reward_token_account: Owned(accts[20].clone()),
        ah_auctioneer_pda: Owned(accts[21].clone()),
        auction_house_program: Owned(accts[23].clone()),
        token_program: Owned(accts[24].clone()),
        escrow_payment_bump: params.escrow_payment_bump.try_into()?,
        free_trade_state_bump: params.free_trade_state_bump.try_into()?,
        program_as_signer_bump: params.program_as_signer_bump.try_into()?,
        seller_trade_state_bump: params.seller_trade_state_bump.try_into()?,
        buyer_trade_state_bump: params.buyer_trade_state_bump.try_into()?,
        slot: slot.try_into()?,
    };

    let offer = client
        .db()
        .run({
            let offer = row.clone().offer;
            move |db| {
                rewards_offers::table
                    .select((rewards_offers::token_size, rewards_offers::price))
                    .filter(rewards_offers::address.eq(offer.to_string()))
                    .first(db)
                    .optional()
            }
        })
        .await
        .context("failed to load reward offer!")?;

    if let Some((token_size, price)) = offer {
        upsert_into_purchases_table(
            client,
            Purchase {
                id: None,
                buyer: row.buyer.clone(),
                seller: row.seller.clone(),
                auction_house: row.auction_house.clone(),
                marketplace_program: Owned(pubkeys::AUCTION_HOUSE.to_string()),
                metadata: row.metadata.clone(),
                token_size,
                price,
                created_at: Utc::now().naive_utc(),
                slot: row.slot,
                write_version: None,
            },
            accts[13].clone(),
            accts[14].clone(),
            row.reward_center.to_string(),
        )
        .await
        .context("failed to insert purchase!")?;
    }

    client
        .db()
        .run(move |db| {
            insert_into(accept_offer_ins::table)
                .values(&row)
                .execute(db)
        })
        .await
        .context("failed to insert reward center accept offer instruction ")?;
    Ok(())
}
