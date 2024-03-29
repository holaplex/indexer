use borsh::BorshDeserialize;
use hpl_reward_center::state::{PayoutOperation, RewardCenter, RewardRules};
use indexer::prelude::*;
use indexer_core::{
    bigdecimal::BigDecimal,
    db::{
        custom_types::{ActivityTypeEnum, PayoutOperationEnum},
        insert_into,
        models::{BuyListing, Purchase, RewardCenter as DbRewardCenter, RewardPayout},
        mutations,
        mutations::activity,
        select,
        tables::{
            buy_listing_ins, listings, offers, purchases, reward_centers, reward_payouts,
            rewards_listings,
        },
        update,
    },
    pubkeys,
};

use super::super::Client;

#[allow(clippy::pedantic)]
pub(crate) async fn process(
    client: &Client,
    tx_signature: String,
    data: &[u8],
    accounts: &[Pubkey],
    slot: u64,
    timestamp: NaiveDateTime,
) -> Result<()> {
    let params = hpl_reward_center::listings::buy::BuyListingParams::try_from_slice(data)
        .context("failed to deserialize buy listing params")?;

    let accts: Vec<_> = accounts.iter().map(ToString::to_string).collect();

    let row = BuyListing {
        tx_signature: Owned(tx_signature),
        buyer: Owned(accts[0].clone()),
        payment_account: Owned(accts[1].clone()),
        transfer_authority: Owned(accts[2].clone()),
        buyer_reward_token_account: Owned(accts[3].clone()),
        seller: Owned(accts[4].clone()),
        seller_reward_token_account: Owned(accts[5].clone()),
        listing: Owned(accts[6].clone()),
        token_account: Owned(accts[7].clone()),
        token_mint: Owned(accts[8].clone()),
        metadata: Owned(accts[9].clone()),
        treasury_mint: Owned(accts[10].clone()),
        seller_payment_receipt_account: Owned(accts[11].clone()),
        buyer_receipt_token_account: Owned(accts[12].clone()),
        authority: Owned(accts[13].clone()),
        escrow_payment_account: Owned(accts[14].clone()),
        auction_house: Owned(accts[15].clone()),
        auction_house_fee_account: Owned(accts[16].clone()),
        auction_house_treasury: Owned(accts[17].clone()),
        buyer_trade_state: Owned(accts[18].clone()),
        seller_trade_state: Owned(accts[19].clone()),
        free_seller_trade_state: Owned(accts[20].clone()),
        reward_center: Owned(accts[21].clone()),
        reward_center_reward_token_account: Owned(accts[22].clone()),
        ah_auctioneer_pda: Owned(accts[23].clone()),
        auction_house_program: Owned(accts[25].clone()),
        token_program: Owned(accts[26].clone()),
        buyer_trade_state_bump: params.buyer_trade_state_bump.try_into()?,
        escrow_payment_bump: params.escrow_payment_bump.try_into()?,
        free_trade_state_bump: params.free_trade_state_bump.try_into()?,
        seller_trade_state_bump: params.seller_trade_state_bump.try_into()?,
        program_as_signer_bump: params.program_as_signer_bump.try_into()?,
        slot: slot.try_into()?,
    };

    let listing = client
        .db()
        .run({
            let listing = row.clone().listing;
            move |db| {
                rewards_listings::table
                    .select((rewards_listings::token_size, rewards_listings::price))
                    .filter(rewards_listings::address.eq(listing.to_string()))
                    .first(db)
                    .optional()
            }
        })
        .await
        .context("failed to load reward listing!")?;

    if let Some((token_size, price)) = listing {
        upsert_into_purchases_table(
            client,
            Purchase {
                id: None,
                buyer: row.buyer.clone(),
                seller: row.seller.clone(),
                auction_house: row.auction_house.clone(),
                marketplace_program: Owned(pubkeys::REWARD_CENTER.to_string()),
                metadata: row.metadata.clone(),
                token_size,
                price,
                created_at: timestamp,
                slot: row.slot,
                write_version: None,
            },
            row.buyer_trade_state.to_string(),
            row.seller_trade_state.to_string(),
            row.reward_center.to_string(),
        )
        .await
        .context("failed to insert purchase!")?;
    }

    client
        .db()
        .run(move |db| insert_into(buy_listing_ins::table).values(&row).execute(db))
        .await
        .context("failed to insert reward center accept offer instruction ")?;
    Ok(())
}

#[allow(clippy::too_many_lines)]
pub(crate) async fn upsert_into_purchases_table<'a>(
    client: &Client,
    data: Purchase<'static>,
    buyer_trade_state: String,
    seller_trade_state: String,
    reward_center_address: String,
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

            let purchase_id = mutations::purchase::insert(db, &data)?;

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

            activity::purchase(db, purchase_id, &data.clone(), ActivityTypeEnum::Purchase)?;

            let reward_center = reward_centers::table
                .select(reward_centers::all_columns)
                .filter(reward_centers::address.eq(reward_center_address.clone()))
                .first::<DbRewardCenter>(db)
                .optional()?;

            if let Some(r) = reward_center {
                let (buyer_reward, seller_reward) = calculate_payout(data.price.try_into()?, &r)?;

                let reward_payout = RewardPayout {
                    purchase_id,
                    metadata: Owned(data.metadata.to_string()),
                    reward_center: Owned(reward_center_address),
                    buyer: Owned(data.buyer.to_string()),
                    buyer_reward,
                    seller: Owned(data.seller.to_string()),
                    seller_reward,
                    created_at: data.created_at,
                    slot: data.slot,
                    write_version: -1,
                };

                insert_into(reward_payouts::table)
                    .values(&reward_payout)
                    .on_conflict_do_nothing()
                    .execute(db)?;
            }
            Result::<_>::Ok(())
        })
        .await?;

    Ok(())
}

fn calculate_payout(price: u64, r: &DbRewardCenter) -> Result<(BigDecimal, BigDecimal)> {
    let reward_center = RewardCenter {
        token_mint: r.token_mint.parse()?,
        auction_house: r.auction_house.parse()?,
        reward_rules: RewardRules {
            seller_reward_payout_basis_points: r.seller_reward_payout_basis_points.try_into()?,
            mathematical_operand: match r.mathematical_operand {
                PayoutOperationEnum::Multiple => PayoutOperation::Multiple,
                PayoutOperationEnum::Divide => PayoutOperation::Divide,
            },
            payout_numeral: r.payout_numeral.try_into()?,
        },
        bump: r.bump.try_into()?,
    };

    let (seller, buyer) = reward_center.payouts(price)?;

    Ok((seller.try_into()?, buyer.try_into()?))
}
