use anchor_lang_v0_22_1::AccountDeserialize;
use goki_smart_wallet::{InstructionBuffer, SmartWallet, SubaccountInfo, Transaction};

use super::{accounts::smart_wallet, AccountUpdate, Client};
use crate::prelude::*;

/// 49
const SUBACCOUNT_INFO_SIZE: usize = 8 + SubaccountInfo::LEN;
/// 237
// const MIN_SMART_WALLET_SIZE: usize = 8 + 32 + 1 + 8 + 8 + 8 + 4 + 8 + 32 + 16 * 18;
// /// 42
// const TXACCOUNTMETA_SIZE: usize = 8 + 32 + 1 + 1;
// /// 75
// const MIN_TXINSTRUCTION_SIZE: usize = 32 + TXACCOUNTMETA_SIZE + 1;
// /// 201
// const MIN_TRANSACTION_SIZE: usize = 32 + 8 + 1 + 32 + MIN_TXINSTRUCTION_SIZE + 1 + 4 + 8 + 32 + 8;
// /// 184
// const MIN_INSTRUCTIONBUFFER_SIZE: usize = 4 + 8 + 32 + 32 + 32 + MIN_INS_BUNDLE_SIZE;
// const MIN_INS_BUNDLE_SIZE: usize = 1 + MIN_TXINSTRUCTION_SIZE;

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
