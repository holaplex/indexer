//! Tribeca Simple-Voter program accounts indexing
use goki_smart_wallet::{InstructionBuffer, SmartWallet, SubaccountInfo, Transaction};
use indexer_core::{db, db::insert_into, prelude::*};

use super::Client;
use crate::prelude::*;

pub(crate) async fn process_smart_wallet(
    client: &Client,
    key: Pubkey,
    sm: SmartWallet,
) -> Result<()> {
    Ok(())
}

pub(crate) async fn process_transaction(
    client: &Client,
    key: Pubkey,
    tx: Transaction,
) -> Result<()> {
    Ok(())
}

pub(crate) async fn process_subaccount_info(
    client: &Client,
    key: Pubkey,
    acc: SubaccountInfo,
) -> Result<()> {
    Ok(())
}

pub(crate) async fn process_instruction_buffer(
    client: &Client,
    key: Pubkey,
    ins: InstructionBuffer,
) -> Result<()> {
    Ok(())
}
