use cardinal_token_manager::state::TokenManager as TokenManagerAccount;
use indexer_core::{
    db::{
        insert_into,
        models::{TokenManager, TokenManagerInvalidator},
        tables::{token_manager_invalidators, token_managers},
    },
    prelude::*,
};

use super::Client;
use crate::prelude::*;

pub(crate) async fn process(
    client: &Client,
    key: Pubkey,
    token_manager: TokenManagerAccount,
) -> Result<()> {
    let row = TokenManager {
        address: Owned(bs58::encode(key).into_string()),
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
        receipt_mint: Some(Owned(
            bs58::encode(token_manager.recipient_token_account).into_string(),
        )),
        claim_approver: Some(Owned(
            bs58::encode(token_manager.recipient_token_account).into_string(),
        )),
        transfer_authority: Some(Owned(
            bs58::encode(token_manager.recipient_token_account).into_string(),
        )),
    };
    client
        .db()
        .run(move |db| {
            insert_into(token_managers::table)
                .values(&row)
                .on_conflict(token_managers::address)
                .do_update()
                .set(&row)
                .execute(db)
        })
        .await
        .context("Failed to insert TokenManager")?;

    let mut invalidator_strings = Vec::new();
    for invalidator in token_manager.invalidators {
        invalidator_strings.push(bs58::encode(invalidator).into_string())
    }
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
        let row = TokenManagerInvalidator {
            token_manager_address: Owned(bs58::encode(token_manager_address).into_string()),
            invalidator: Owned(invalidator),
        };

        // TODO remove all other invalidator
        client
            .db()
            .run(move |db| {
                insert_into(token_manager_invalidators::table)
                    .values(&row)
                    .on_conflict((
                        token_manager_invalidators::token_manager_address,
                        token_manager_invalidators::invalidator,
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
