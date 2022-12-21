//! Tribeca Locked-Voter program accounts indexing
use indexer_core::{
    db::{
        insert_into,
        models::{
            Escrow as EscrowModel, Locker as LockerModel, LockerParam as LockerParamModel,
            LockerWhitelistEntry as LockerWhitelistEntryModel,
        },
        tables::{escrows, locker_params, locker_whitelist_entries, lockers},
    },
    prelude::*,
};
use tribeca_locked_voter::{Escrow, Locker, LockerParams, LockerWhitelistEntry};

use super::Client;
use crate::prelude::*;

pub(crate) async fn process_locker(client: &Client, key: Pubkey, l: Locker) -> Result<()> {
    let row = LockerModel {
        address: Owned(key.to_string()),
        base: Owned(l.base.to_string()),
        bump: l.bump.try_into()?,
        token_mint: Owned(l.token_mint.to_string()),
        locked_supply: l.locked_supply.try_into()?,
        governor: Owned(l.governor.to_string()),
    };

    client
        .db()
        .run(move |db| {
            insert_into(lockers::table)
                .values(&row)
                .on_conflict(lockers::address)
                .do_update()
                .set(&row)
                .execute(db)
        })
        .await
        .context("failed to insert locker ")?;

    process_locker_params(client, key, l.params).await
}

async fn process_locker_params(client: &Client, key: Pubkey, params: LockerParams) -> Result<()> {
    let row = LockerParamModel {
        locker_address: Owned(key.to_string()),
        whitelist_enabled: params.whitelist_enabled,
        max_stake_vote_multiplier: params.max_stake_vote_multiplier.try_into()?,
        min_stake_duration: params.min_stake_duration.try_into()?,
        max_stake_duration: params.max_stake_duration.try_into()?,
        proposal_activation_min_votes: params.proposal_activation_min_votes.try_into()?,
    };

    client
        .db()
        .run(move |db| {
            insert_into(locker_params::table)
                .values(&row)
                .on_conflict(locker_params::locker_address)
                .do_update()
                .set(&row)
                .execute(db)
        })
        .await
        .context("failed to insert locker parameters")?;

    Ok(())
}

pub(crate) async fn process_escrow(client: &Client, key: Pubkey, es: Escrow) -> Result<()> {
    let row = EscrowModel {
        address: Owned(key.to_string()),
        locker: Owned(es.locker.to_string()),
        owner: Owned(es.owner.to_string()),
        bump: es.bump.try_into()?,
        tokens: Owned(es.tokens.to_string()),
        amount: es.amount.try_into()?,
        escrow_started_at: es.escrow_started_at,
        escrow_ends_at: es.escrow_ends_at,
        vote_delegate: Owned(es.vote_delegate.to_string()),
    };

    client
        .db()
        .run(move |db| {
            insert_into(escrows::table)
                .values(&row)
                .on_conflict(escrows::address)
                .do_update()
                .set(&row)
                .execute(db)
        })
        .await
        .context("failed to insert escrow account ")?;

    Ok(())
}

pub(crate) async fn process_locker_whitelist_entry(
    client: &Client,
    key: Pubkey,
    wl: LockerWhitelistEntry,
) -> Result<()> {
    let row = LockerWhitelistEntryModel {
        address: Owned(key.to_string()),
        bump: wl.bump.try_into()?,
        locker: Owned(wl.locker.to_string()),
        program_id: Owned(wl.program_id.to_string()),
        owner: Owned(wl.owner.to_string()),
    };

    client
        .db()
        .run(move |db| {
            insert_into(locker_whitelist_entries::table)
                .values(&row)
                .on_conflict(locker_whitelist_entries::address)
                .do_update()
                .set(&row)
                .execute(db)
        })
        .await
        .context("failed to insert locker whitelist entry account ")?;

    Ok(())
}
