use borsh::BorshDeserialize;
use hpl_reward_center::state::{PayoutOperation, RewardCenter, RewardRules};
use indexer_core::{
    bigdecimal::BigDecimal,
    db::{
        custom_types::PayoutOperationEnum,
        insert_into,
        models::{
            FeedEventWallet, HplRewardCenterExecuteSale, Purchase, PurchaseEvent,
            RewardCenter as DbRewardCenter, RewardPayout,
        },
        on_constraint, select,
        tables::{
            feed_event_wallets, feed_events, hpl_reward_center_execute_sale_ins, listings, offers,
            purchase_events, purchases, reward_centers, reward_payouts, rewards_listings,
        },
        update,
    },
    pubkeys,
    uuid::Uuid,
};

use super::super::Client;
use crate::prelude::*;

#[allow(clippy::pedantic)]
pub(crate) async fn process(
    client: &Client,
    data: &[u8],
    accounts: &[Pubkey],
    slot: u64,
) -> Result<()> {
    let params = hpl_reward_center::execute_sale::ExecuteSaleParams::try_from_slice(data)
        .context("failed to deserialize")?;

    let accts: Vec<_> = accounts.iter().map(ToString::to_string).collect();

    let row = HplRewardCenterExecuteSale {
        buyer: Owned(accts[0].clone()),
        buyer_reward_token_account: Owned(accts[1].clone()),
        seller: Owned(accts[2].clone()),
        seller_reward_token_account: Owned(accts[3].clone()),
        listing: Owned(accts[4].clone()),
        offer: Owned(accts[5].clone()),
        payer: Owned(accts[6].clone()),
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
        free_trade_state: Owned(accts[20].clone()),
        reward_center: Owned(accts[21].clone()),
        reward_center_reward_token_account: Owned(accts[22].clone()),
        ah_auctioneer_pda: Owned(accts[23].clone()),
        escrow_payment_bump: params.escrow_payment_bump.try_into()?,
        free_trade_state_bump: params.free_trade_state_bump.try_into()?,
        program_as_signer_bump: params.program_as_signer_bump.try_into()?,
        created_at: Utc::now().naive_utc(),
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
        .context("failed to load rewards listing!")?;

    if let Some((token_size, price)) = listing {
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
                created_at: row.created_at,
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
            insert_into(hpl_reward_center_execute_sale_ins::table)
                .values(&row)
                .execute(db)
        })
        .await
        .context("failed to insert reward center execute sale instruction ")?;
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
