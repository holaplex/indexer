use anchor_lang_v0_22_1::AccountDeserialize;
use goki_smart_wallet::{InstructionBuffer, SmartWallet, SubaccountInfo, Transaction};

use super::{accounts::smart_wallet, AccountUpdate, Client};
use crate::prelude::*;

const SUBACCOUNT_INFO_SIZE: usize = SubaccountInfo::LEN;
const MIN_SMART_WALLET_SIZE: usize = 32 + 1 + 8 + 8 + 8 + 4 + 8 + 8 * 16 + 32;

async fn process_smart_wallet(client: &Client, update: AccountUpdate) -> Result<()> {
    let smart_wallet = SmartWallet::try_deserialize_unchecked(&mut update.data.as_slice())
        .context("failed to deserialize smart wallet!")?;

    smart_wallet::process_smart_wallet(client, update.key, smart_wallet).await
}

async fn process_transaction(client: &Client, update: AccountUpdate) -> Result<()> {
    let tx = Transaction::try_deserialize_unchecked(&mut update.data.as_slice())
        .context("failed to deserialize transaction!")?;

    smart_wallet::process_transaction(client, update.key, tx).await
}

async fn process_subaccount_info(client: &Client, update: AccountUpdate) -> Result<()> {
    let subaccount = SubaccountInfo::try_deserialize_unchecked(&mut update.data.as_slice())
        .context("failed to deserialize subaccount!")?;

    smart_wallet::process_subaccount_info(client, update.key, subaccount).await
}

async fn process_instruction_buffer(client: &Client, update: AccountUpdate) -> Result<()> {
    let ins = InstructionBuffer::try_deserialize_unchecked(&mut update.data.as_slice())
        .context("failed to deserialize instruction buffer!")?;

    smart_wallet::process_instruction_buffer(client, update.key, ins).await
}

pub(crate) async fn process(client: &Client, update: AccountUpdate) -> Result<()> {
    match update.data.len() {
        _ => Ok(()),
    }
}
