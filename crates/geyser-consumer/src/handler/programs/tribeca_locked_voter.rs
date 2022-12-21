use anchor_lang_v0_22::AccountDeserialize;
use tribeca_locked_voter::{Escrow, Locker, LockerWhitelistEntry};

use super::{accounts::locked_voter, AccountUpdate, Client};
use crate::prelude::*;

const LOCKER_SIZE: usize = 8 + 32 + 1 + 32 + 8 + 32 + LOCKER_PARAMS_SIZE;
const ESCROW_SIZE: usize = 8 + 32 + 32 + 1 + 32 + 8 + 8 + 8 + 32;
const LOCKER_PARAMS_SIZE: usize = 1 + 1 + 8 + 8 + 8;
const LOCKER_WHITELIST_ENTRY_SIZE: usize = 8 + 1 + 32 + 32 + 32;

async fn process_locker(client: &Client, update: AccountUpdate) -> Result<()> {
    let locker = Locker::try_deserialize_unchecked(&mut update.data.as_slice())
        .context("failed to deserialize locker account!")?;

    locked_voter::process_locker(client, update.key, locker).await
}

async fn process_escrow(client: &Client, update: AccountUpdate) -> Result<()> {
    let escrow = Escrow::try_deserialize_unchecked(&mut update.data.as_slice())
        .context("failed to deserialize escrow account!")?;

    locked_voter::process_escrow(client, update.key, escrow).await
}

async fn process_locker_whitelist_entry(client: &Client, update: AccountUpdate) -> Result<()> {
    let whitelist_entry =
        LockerWhitelistEntry::try_deserialize_unchecked(&mut update.data.as_slice())
            .context("failed to deserialize locker whitelist entry account!")?;

    locked_voter::process_locker_whitelist_entry(client, update.key, whitelist_entry).await
}

pub(crate) async fn process(client: &Client, update: AccountUpdate) -> Result<()> {
    match update.data.len() {
        LOCKER_SIZE => process_locker(client, update).await,
        ESCROW_SIZE => process_escrow(client, update).await,
        LOCKER_WHITELIST_ENTRY_SIZE => process_locker_whitelist_entry(client, update).await,
        _ => Ok(()),
    }
}
