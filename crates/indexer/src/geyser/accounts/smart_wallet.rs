//! Tribeca Simple-Voter program accounts indexing
use goki_smart_wallet::{
    InstructionBuffer, InstructionBundle, SmartWallet, SubaccountInfo, SubaccountType,
    TXAccountMeta, TXInstruction, Transaction,
};
use indexer_core::{
    db::{
        insert_into,
        models::{
            InsBufferBundle, InsBufferBundleInsKey, InsBuffferBundleInstruction,
            InstructionBuffer as InstructionBufferModel, SmartWallet as SmartWalletModel,
            SmartWalletOwner, SubAccountInfo as SubaccountInfoModel,
            TXInstruction as TXInstructionModel, TXInstructionKey, Transaction as TransactionModel,
        },
        tables::{
            ins_buffer_bundle_ins_keys, ins_buffer_bundle_instructions, ins_buffer_bundles,
            instruction_buffers, smart_wallet_owners, smart_wallets, sub_account_infos,
            transactions, tx_instruction_keys, tx_instructions,
        },
    },
    prelude::*,
};

use super::Client;
use crate::prelude::*;

pub(crate) async fn process_smart_wallet(
    client: &Client,
    key: Pubkey,
    sm: SmartWallet,
) -> Result<()> {
    let row = SmartWalletModel {
        address: Owned(key.to_string()),
        base: Owned(sm.base.to_string()),
        bump: sm.bump.try_into()?,
        threshold: sm.threshold.try_into()?,
        minimum_delay: sm.minimum_delay,
        grace_period: sm.grace_period,
        owner_set_seqno: sm.owner_set_seqno.try_into()?,
        num_transactions: sm.num_transactions.try_into()?,
    };

    client
        .db()
        .run(move |db| {
            insert_into(smart_wallets::table)
                .values(&row)
                .on_conflict(smart_wallets::address)
                .do_update()
                .set(&row)
                .execute(db)
        })
        .await
        .context("failed to insert smart wallet ")?;

    process_smart_wallet_owner(client, key, sm.owners).await
}

async fn process_smart_wallet_owner(
    client: &Client,
    key: Pubkey,
    owners: Vec<Pubkey>,
) -> Result<()> {
    for (i, owner) in owners.iter().enumerate() {
        let o = SmartWalletOwner {
            smart_wallet_address: Owned(key.to_string()),
            owner_address: Owned(owner.to_string()),
            index: i.try_into()?,
        };

        client
            .db()
            .run(move |db| {
                insert_into(smart_wallet_owners::table)
                    .values(&o)
                    .on_conflict((
                        smart_wallet_owners::smart_wallet_address,
                        smart_wallet_owners::owner_address,
                    ))
                    .do_update()
                    .set(&o)
                    .execute(db)
            })
            .await
            .context("failed to insert smart wallet owner")?;
    }
    Ok(())
}

pub(crate) async fn process_transaction(
    client: &Client,
    key: Pubkey,
    tx: Transaction,
) -> Result<()> {
    let row = TransactionModel {
        address: Owned(key.to_string()),
        smart_wallet: Owned(tx.smart_wallet.to_string()),
        index: tx.index.try_into()?,
        bump: tx.bump.try_into()?,
        proposer: Owned(tx.proposer.to_string()),
        signers: tx.signers,
        owner_set_seqno: tx.owner_set_seqno.try_into()?,
        eta: tx.eta,
        executor: Owned(tx.executor.to_string()),
        executed_at: tx.executed_at,
    };

    client
        .db()
        .run(move |db| {
            insert_into(transactions::table)
                .values(&row)
                .on_conflict(transactions::address)
                .do_update()
                .set(&row)
                .execute(db)
        })
        .await
        .context("failed to insert transaction ")?;
    process_intructions(client, key, tx.instructions).await
}

async fn process_intructions(client: &Client, key: Pubkey, ins: Vec<TXInstruction>) -> Result<()> {
    for i in ins {
        let row = TXInstructionModel {
            transaction_address: Owned(key.to_string()),
            program_id: Owned(i.program_id.to_string()),
            data: i.data,
        };

        client
            .db()
            .run(move |db| {
                insert_into(tx_instructions::table)
                    .values(&row)
                    .on_conflict((
                        tx_instructions::transaction_address,
                        tx_instructions::program_id,
                    ))
                    .do_update()
                    .set(&row)
                    .execute(db)
            })
            .await
            .context("failed to insert transaction instruction")?;

        process_tx_instruction_keys(client, key, i.program_id, i.keys).await?;
    }

    Ok(())
}

async fn process_tx_instruction_keys(
    client: &Client,
    tx_addr: Pubkey,
    id: Pubkey,
    keys: Vec<TXAccountMeta>,
) -> Result<()> {
    for key in keys {
        let k = TXInstructionKey {
            transaction_address: Owned(tx_addr.to_string()),
            program_id: Owned(id.to_string()),
            pubkey: Owned(key.pubkey.to_string()),
            is_signer: key.is_signer,
            is_writable: key.is_writable,
        };

        client
            .db()
            .run(move |db| {
                insert_into(tx_instruction_keys::table)
                    .values(&k)
                    .on_conflict((
                        tx_instruction_keys::transaction_address,
                        tx_instruction_keys::program_id,
                        tx_instruction_keys::pubkey,
                    ))
                    .do_update()
                    .set(&k)
                    .execute(db)
            })
            .await
            .context("failed to insert transaction instruction account metadata")?;
    }

    Ok(())
}

pub(crate) async fn process_subaccount_info(
    client: &Client,
    key: Pubkey,
    acc: SubaccountInfo,
) -> Result<()> {
    let s = SubaccountInfoModel {
        address: Owned(key.to_string()),
        smart_wallet: Owned(acc.smart_wallet.to_string()),
        subaccount_type: match acc.subaccount_type {
            SubaccountType::Derived => 0,
            SubaccountType::OwnerInvoker => 1,
        },
        index: acc.index.try_into()?,
    };

    client
        .db()
        .run(move |db| {
            insert_into(sub_account_infos::table)
                .values(&s)
                .on_conflict(sub_account_infos::address)
                .do_update()
                .set(&s)
                .execute(db)
        })
        .await
        .context("failed to insert subaccount info")?;

    Ok(())
}

pub(crate) async fn process_instruction_buffer(
    client: &Client,
    ib_addr: Pubkey,
    ib: InstructionBuffer,
) -> Result<()> {
    let ins_buffer = InstructionBufferModel {
        address: Owned(ib_addr.to_string()),
        owner_set_seqno: ib.owner_set_seqno.try_into()?,
        eta: ib.eta,
        authority: Owned(ib.authority.to_string()),
        executor: Owned(ib.executor.to_string()),
        smart_wallet: Owned(ib.smart_wallet.to_string()),
    };

    client
        .db()
        .run(move |db| {
            insert_into(instruction_buffers::table)
                .values(&ins_buffer)
                .on_conflict(instruction_buffers::address)
                .do_update()
                .set(&ins_buffer)
                .execute(db)
        })
        .await
        .context("failed to insert instruction buffer")?;

    process_ins_buffer_bundles(client, ib_addr, ib.bundles).await
}

async fn process_ins_buffer_bundles(
    client: &Client,
    ib_addr: Pubkey,
    bundles: Vec<InstructionBundle>,
) -> Result<()> {
    for bundle in bundles {
        let b = InsBufferBundle {
            instruction_buffer_address: Owned(ib_addr.to_string()),
            is_executed: bundle.is_executed,
        };

        client
            .db()
            .run(move |db| {
                insert_into(ins_buffer_bundles::table)
                    .values(&b)
                    .on_conflict(ins_buffer_bundles::instruction_buffer_address)
                    .do_update()
                    .set(&b)
                    .execute(db)
            })
            .await
            .context("failed to insert instruction buffer bundle")?;
        process_ins_buffer_bundle_instructions(client, ib_addr, bundle.instructions).await?;
    }

    Ok(())
}

async fn process_ins_buffer_bundle_instructions(
    client: &Client,
    ib_addr: Pubkey,
    bundle_instructions: Vec<TXInstruction>,
) -> Result<()> {
    for ins in bundle_instructions {
        let bundle_ins = InsBuffferBundleInstruction {
            instruction_buffer_address: Owned(ib_addr.to_string()),
            program_id: Owned(ins.program_id.to_string()),
            data: ins.data,
        };
        client
            .db()
            .run(move |db| {
                insert_into(ins_buffer_bundle_instructions::table)
                    .values(&bundle_ins)
                    .on_conflict((
                        ins_buffer_bundle_instructions::instruction_buffer_address,
                        ins_buffer_bundle_instructions::program_id,
                    ))
                    .do_update()
                    .set(&bundle_ins)
                    .execute(db)
            })
            .await
            .context("failed to insert instruction buffer bundle instruction")?;
        process_ins_buffer_bundle_ins_keys(client, ib_addr, ins.program_id, ins.keys).await?;
    }
    Ok(())
}
async fn process_ins_buffer_bundle_ins_keys(
    client: &Client,
    ib_addr: Pubkey,
    program_id: Pubkey,
    keys: Vec<TXAccountMeta>,
) -> Result<()> {
    for key in keys {
        let k = InsBufferBundleInsKey {
            instruction_buffer_address: Owned(ib_addr.to_string()),
            program_id: Owned(program_id.to_string()),
            pubkey: Owned(key.pubkey.to_string()),
            is_signer: key.is_signer,
            is_writable: key.is_writable,
        };

        client
            .db()
            .run(move |db| {
                insert_into(ins_buffer_bundle_ins_keys::table)
                    .values(&k)
                    .on_conflict((
                        ins_buffer_bundle_ins_keys::instruction_buffer_address,
                        ins_buffer_bundle_ins_keys::program_id,
                        ins_buffer_bundle_ins_keys::pubkey,
                    ))
                    .do_update()
                    .set(&k)
                    .execute(db)
            })
            .await
            .context("failed to insert instruction buffer bundle instruction account metadata")?;
    }
    Ok(())
}
