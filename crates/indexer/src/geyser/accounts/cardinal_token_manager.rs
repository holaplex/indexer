use cardinal_token_manager::state::{TokenManager as TokenManagerAccount, TokenManagerState};
use indexer_core::{
    db::{
        delete, insert_into,
        models::{
            CardinalClaimEvent, CardinalTokenManager, CardinalTokenManagerInvalidator,
            CardinalTokenManagerQuery,
        },
        tables::{
            cardinal_claim_events, cardinal_paid_claim_approvers, cardinal_time_invalidators,
            cardinal_token_manager_invalidators, cardinal_token_managers,
            cardinal_use_invalidators,
        },
    },
    prelude::*,
};

use super::Client;
use crate::prelude::*;

#[inline]
async fn get_current_token_managers(
    client: &Client,
    key: Pubkey,
) -> Result<Vec<CardinalTokenManagerQuery>> {
    client
        .db()
        .run(move |db| {
            cardinal_token_managers::table
                .left_outer_join(
                    cardinal_paid_claim_approvers::table.on(
                        cardinal_token_managers::claim_approver
                            .nullable()
                            .eq(cardinal_paid_claim_approvers::paid_claim_approver_address
                                .nullable()),
                    ),
                )
                .left_outer_join(
                    cardinal_token_manager_invalidators::table.on(cardinal_token_managers::address
                        .eq(cardinal_token_manager_invalidators::token_manager_address)),
                )
                .left_outer_join(
                    cardinal_time_invalidators::table.on(
                        cardinal_time_invalidators::time_invalidator_token_manager_address
                            .eq(cardinal_token_manager_invalidators::token_manager_address)
                            .and(
                                cardinal_time_invalidators::time_invalidator_address
                                    .eq(cardinal_token_manager_invalidators::invalidator),
                            ),
                    ),
                )
                .left_outer_join(
                    cardinal_use_invalidators::table.on(
                        cardinal_use_invalidators::use_invalidator_token_manager_address
                            .eq(cardinal_token_manager_invalidators::token_manager_address)
                            .and(
                                cardinal_use_invalidators::use_invalidator_address
                                    .eq(cardinal_token_manager_invalidators::invalidator),
                            ),
                    ),
                )
                .filter(cardinal_token_managers::address.eq(bs58::encode(key).into_string()))
                .select((
                    cardinal_token_managers::address,
                    cardinal_token_managers::version,
                    cardinal_token_managers::bump,
                    cardinal_token_managers::count,
                    cardinal_token_managers::num_invalidators,
                    cardinal_token_managers::issuer,
                    cardinal_token_managers::mint,
                    cardinal_token_managers::amount,
                    cardinal_token_managers::kind,
                    cardinal_token_managers::state,
                    cardinal_token_managers::state_changed_at,
                    cardinal_token_managers::invalidation_type,
                    cardinal_token_managers::recipient_token_account,
                    cardinal_token_managers::receipt_mint,
                    cardinal_token_managers::claim_approver,
                    cardinal_token_managers::transfer_authority,
                    cardinal_paid_claim_approvers::paid_claim_approver_payment_amount.nullable(),
                    cardinal_paid_claim_approvers::paid_claim_approver_payment_mint.nullable(),
                    cardinal_paid_claim_approvers::paid_claim_approver_payment_manager.nullable(),
                    cardinal_paid_claim_approvers::paid_claim_approver_collector.nullable(),
                    cardinal_time_invalidators::time_invalidator_address.nullable(),
                    cardinal_time_invalidators::time_invalidator_payment_manager.nullable(),
                    cardinal_time_invalidators::time_invalidator_collector.nullable(),
                    cardinal_time_invalidators::time_invalidator_expiration.nullable(),
                    cardinal_time_invalidators::time_invalidator_duration_seconds.nullable(),
                    cardinal_time_invalidators::time_invalidator_extension_payment_amount
                        .nullable(),
                    cardinal_time_invalidators::time_invalidator_extension_duration_seconds
                        .nullable(),
                    cardinal_time_invalidators::time_invalidator_extension_payment_mint.nullable(),
                    cardinal_time_invalidators::time_invalidator_max_expiration.nullable(),
                    cardinal_time_invalidators::time_invalidator_disable_partial_extension
                        .nullable(),
                    cardinal_use_invalidators::use_invalidator_address.nullable(),
                    cardinal_use_invalidators::use_invalidator_payment_manager.nullable(),
                    cardinal_use_invalidators::use_invalidator_collector.nullable(),
                    cardinal_use_invalidators::use_invalidator_usages.nullable(),
                    cardinal_use_invalidators::use_invalidator_use_authority.nullable(),
                    cardinal_use_invalidators::use_invalidator_total_usages.nullable(),
                    cardinal_use_invalidators::use_invalidator_extension_payment_amount.nullable(),
                    cardinal_use_invalidators::use_invalidator_extension_payment_mint.nullable(),
                    cardinal_use_invalidators::use_invalidator_extension_usages.nullable(),
                    cardinal_use_invalidators::use_invalidator_max_usages.nullable(),
                ))
                .load(db)
        })
        .await
        .context("Failed to find TokenManager")
}

#[allow(clippy::too_many_lines)]
async fn store_claim_event(
    client: &Client,
    key: Pubkey,
    token_manager: &TokenManagerAccount,
    current_token_manager: &CardinalTokenManagerQuery,
) -> Result<()> {
    let claim_event = CardinalClaimEvent {
        token_manager_address: Owned(bs58::encode(key).into_string()),
        version: token_manager.version.try_into()?,
        bump: token_manager.bump.try_into()?,
        count: token_manager.count.try_into()?,
        num_invalidators: token_manager.num_invalidators.try_into()?,
        issuer: Owned(bs58::encode(token_manager.issuer).into_string()),
        mint: Owned(bs58::encode(token_manager.mint).into_string()),
        amount: token_manager.amount.try_into()?,
        kind: token_manager.kind.try_into()?,
        state: token_manager.state.try_into()?,
        state_changed_at: NaiveDateTime::from_timestamp(token_manager.state_changed_at, 0),
        invalidation_type: token_manager.invalidation_type.try_into()?,
        recipient_token_account: Owned(
            bs58::encode(token_manager.recipient_token_account).into_string(),
        ),
        receipt_mint: token_manager
            .receipt_mint
            .map(|k| Owned(bs58::encode(k).into_string())),
        claim_approver: token_manager
            .claim_approver
            .map(|k| Owned(bs58::encode(k).into_string())),
        transfer_authority: token_manager
            .transfer_authority
            .map(|k| Owned(bs58::encode(k).into_string())),

        ////////////// claim approver //////////////
        paid_claim_approver_payment_amount: current_token_manager
            .paid_claim_approver_payment_amount,
        paid_claim_approver_payment_mint: current_token_manager
            .paid_claim_approver_payment_mint
            .clone()
            .map(Owned),
        paid_claim_approver_payment_manager: current_token_manager
            .paid_claim_approver_payment_manager
            .clone()
            .map(Owned),
        paid_claim_approver_collector: current_token_manager
            .paid_claim_approver_collector
            .clone()
            .map(Owned),

        ////////////// time invalidator //////////////
        time_invalidator_address: current_token_manager
            .time_invalidator_address
            .clone()
            .map(Owned),
        time_invalidator_payment_manager: current_token_manager
            .time_invalidator_payment_manager
            .clone()
            .map(Owned),
        time_invalidator_collector: current_token_manager
            .time_invalidator_collector
            .clone()
            .map(Owned),
        time_invalidator_expiration: current_token_manager.time_invalidator_expiration,
        time_invalidator_duration_seconds: current_token_manager.time_invalidator_duration_seconds,
        time_invalidator_extension_payment_amount: current_token_manager
            .time_invalidator_extension_payment_amount,
        time_invalidator_extension_duration_seconds: current_token_manager
            .time_invalidator_extension_duration_seconds,
        time_invalidator_extension_payment_mint: current_token_manager
            .time_invalidator_extension_payment_mint
            .clone()
            .map(Owned),
        time_invalidator_max_expiration: current_token_manager.time_invalidator_max_expiration,
        time_invalidator_disable_partial_extension: current_token_manager
            .time_invalidator_disable_partial_extension,

        ////////////// use invalidator //////////////
        use_invalidator_address: current_token_manager
            .use_invalidator_address
            .clone()
            .map(Owned),
        use_invalidator_payment_manager: current_token_manager
            .use_invalidator_payment_manager
            .clone()
            .map(Owned),
        use_invalidator_collector: current_token_manager
            .use_invalidator_collector
            .clone()
            .map(Owned),
        use_invalidator_usages: current_token_manager.use_invalidator_usages,
        use_invalidator_use_authority: current_token_manager
            .use_invalidator_use_authority
            .clone()
            .map(Owned),
        use_invalidator_total_usages: current_token_manager.use_invalidator_total_usages,
        use_invalidator_extension_payment_amount: current_token_manager
            .use_invalidator_extension_payment_amount,
        use_invalidator_extension_payment_mint: current_token_manager
            .use_invalidator_extension_payment_mint
            .clone()
            .map(Owned),
        use_invalidator_extension_usages: current_token_manager.use_invalidator_extension_usages,
        use_invalidator_max_usages: current_token_manager.use_invalidator_max_usages,
    };

    client
        .db()
        .run(move |db| {
            insert_into(cardinal_claim_events::table)
                .values(&claim_event)
                .execute(db)
        })
        .await
        .context("Failed to insert Claim Event")?;

    Ok(())
}

pub(crate) async fn process(
    client: &Client,
    key: Pubkey,
    token_manager: TokenManagerAccount,
) -> Result<()> {
    let current_token_managers = get_current_token_managers(client, key).await?;

    if !current_token_managers.is_empty() && token_manager.state == TokenManagerState::Claimed as u8
    {
        store_claim_event(client, key, &token_manager, &current_token_managers[0]).await?;
    }

    let row = CardinalTokenManager {
        address: Owned(bs58::encode(key).into_string()),
        version: token_manager.version.into(),
        bump: token_manager.bump.into(),
        count: token_manager.count.try_into()?,
        num_invalidators: token_manager.num_invalidators.into(),
        issuer: Owned(bs58::encode(token_manager.issuer).into_string()),
        mint: Owned(bs58::encode(token_manager.mint).into_string()),
        amount: token_manager.amount.try_into()?,
        kind: token_manager.kind.into(),
        state: token_manager.state.into(),
        state_changed_at: NaiveDateTime::from_timestamp(token_manager.state_changed_at, 0),
        invalidation_type: token_manager.invalidation_type.into(),
        recipient_token_account: Owned(
            bs58::encode(token_manager.recipient_token_account).into_string(),
        ),
        receipt_mint: token_manager
            .receipt_mint
            .map(|k| Owned(bs58::encode(k).into_string())),
        claim_approver: token_manager
            .claim_approver
            .map(|k| Owned(bs58::encode(k).into_string())),
        transfer_authority: token_manager
            .transfer_authority
            .map(|k| Owned(bs58::encode(k).into_string())),
    };
    trace!("Processing token manager {:?}", row);

    client
        .db()
        .run(move |db| {
            insert_into(cardinal_token_managers::table)
                .values(&row)
                .on_conflict(cardinal_token_managers::address)
                .do_update()
                .set(&row)
                .execute(db)
        })
        .await
        .context("Failed to insert TokenManager")?;

    let invalidator_strings: Vec<_> = token_manager
        .invalidators
        .into_iter()
        .map(|i| bs58::encode(i).into_string())
        .collect();
    // process invalidators into separate table
    process_invalidators(client, key, invalidator_strings).await?;
    Ok(())
}

async fn process_invalidators(
    client: &Client,
    token_manager_address: Pubkey,
    invalidators: Vec<String>,
) -> Result<()> {
    for invalidator in invalidators {
        let row = CardinalTokenManagerInvalidator {
            token_manager_address: Owned(bs58::encode(token_manager_address).into_string()),
            invalidator: Owned(invalidator),
        };
        client
            .db()
            .run(move |db| {
                delete(cardinal_token_manager_invalidators::table)
                    .filter(
                        cardinal_token_manager_invalidators::token_manager_address
                            .eq(bs58::encode(token_manager_address).into_string()),
                    )
                    .execute(db)
            })
            .await
            .context("failed to delete existing invalidators")?;

        client
            .db()
            .run(move |db| {
                insert_into(cardinal_token_manager_invalidators::table)
                    .values(&row)
                    .on_conflict((
                        cardinal_token_manager_invalidators::token_manager_address,
                        cardinal_token_manager_invalidators::invalidator,
                    ))
                    .do_update()
                    .set(&row)
                    .execute(db)
            })
            .await
            .context("failed to insert invalidator")?;
    }
    Ok(())
}
