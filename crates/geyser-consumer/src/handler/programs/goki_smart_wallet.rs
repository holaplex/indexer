use anchor_lang_v0_22::AccountDeserialize;
use goki_smart_wallet::{InstructionBuffer, SmartWallet, SubaccountInfo, Transaction};

use super::{accounts::smart_wallet, AccountUpdate, Client};
use crate::prelude::*;

const SUBACCOUNT_INFO_SIZE: usize = 8 + SubaccountInfo::LEN;

async fn process_account(client: &Client, update: AccountUpdate) -> Result<()> {
    if let Ok(smart_wallet) = SmartWallet::try_deserialize_unchecked(&mut update.data.as_slice()) {
        return smart_wallet::process_smart_wallet(client, update.key, smart_wallet).await;
    }

    if let Ok(tx) = Transaction::try_deserialize_unchecked(&mut update.data.as_slice()) {
        return smart_wallet::process_transaction(client, update.key, tx).await;
    }

    if let Ok(ins) = InstructionBuffer::try_deserialize_unchecked(&mut update.data.as_slice()) {
        return smart_wallet::process_instruction_buffer(client, update.key, ins).await;
    }
    Ok(())
}

async fn process_subaccount_info(client: &Client, update: AccountUpdate) -> Result<()> {
    let subaccount = SubaccountInfo::try_deserialize_unchecked(&mut update.data.as_slice())
        .context("failed to deserialize subaccount!")?;

    smart_wallet::process_subaccount_info(client, update.key, subaccount).await
}

pub(crate) async fn process(client: &Client, update: AccountUpdate) -> Result<()> {
    match update.data.len() {
        SUBACCOUNT_INFO_SIZE => process_subaccount_info(client, update).await,
        _ => process_account(client, update).await,
    }
}
