use indexer_rabbitmq::geyser::{InstructionIndex, InstructionNotify, Message};
use indexer_selector::{InstructionInfo, InstructionSelector};
use solana_program::instruction::CompiledInstruction;
use solana_transaction_status::{UiInstruction, UiParsedInstruction};

use super::Client;
use crate::prelude::*;

struct InstructionShim<'a> {
    program_id_index: u8,
    accounts: Cow<'a, [u8]>,
    data: Cow<'a, [u8]>,
}

impl<'a> From<&'a CompiledInstruction> for InstructionShim<'a> {
    fn from(ins: &'a CompiledInstruction) -> Self {
        Self {
            program_id_index: ins.program_id_index,
            accounts: Borrowed(&ins.accounts),
            data: Borrowed(&ins.data),
        }
    }
}

impl<'a> InstructionShim<'a> {
    fn try_from_ui(ui: &'a UiInstruction, accounts: &[Pubkey]) -> Result<Self> {
        match ui {
            UiInstruction::Compiled(c) => Ok(Self {
                program_id_index: c.program_id_index,
                accounts: Borrowed(&c.accounts),
                data: Owned(
                    bs58::decode(&c.data)
                        .into_vec()
                        .context("Couldn't parse instruction data")?,
                ),
            }),
            UiInstruction::Parsed(UiParsedInstruction::Parsed(_)) => {
                bail!("Cannot use parsed instructions")
            },
            UiInstruction::Parsed(UiParsedInstruction::PartiallyDecoded(d)) => {
                warn!("Re-encoding a partially decoded instruction.  This is slow!");

                let encode = |s: &str, parse, missing, cast| {
                    let key = s.parse().context(parse)?;
                    accounts
                        .iter()
                        .enumerate()
                        .find_map(|(i, a)| (*a == key).then_some(i))
                        .context(missing)
                        .and_then(|i| u8::try_from(i).context(cast))
                };

                Ok(Self {
                    program_id_index: encode(
                        &d.program_id,
                        "Couldn't parse program ID",
                        "Missing program ID in transaction accounts",
                        "Program ID index overflowed u8",
                    )?,
                    accounts: Owned(
                        d.accounts
                            .iter()
                            .map(|a| {
                                encode(
                                    a,
                                    "Couldn't parse input account",
                                    "Missing input account in transaction accounts",
                                    "Input account index overflowed u8",
                                )
                            })
                            .collect::<Result<Vec<_>>>()?,
                    ),
                    data: Owned(
                        bs58::decode(&d.data)
                            .into_vec()
                            .context("Couldn't parse instruction data")?,
                    ),
                })
            },
        }
    }
}

impl<'a> InstructionInfo<'a> for InstructionShim<'a> {
    type AccountIndices = std::iter::Copied<std::slice::Iter<'a, u8>>;

    #[inline]
    fn program_index(&self) -> u8 {
        self.program_id_index
    }

    #[inline]
    fn account_indices(&'a self) -> Self::AccountIndices {
        self.accounts.iter().copied()
    }

    #[inline]
    fn data(&self) -> &[u8] {
        &self.data
    }
}

fn process_instruction(
    sel: &InstructionSelector,
    ins: Result<(InstructionIndex, InstructionShim<'_>)>,
    keys: &[Pubkey],
    slot: u64,
    txn_signature: &[u8],
) -> Result<Option<Message>> {
    let (index, ins) = ins?;

    if !sel.is_selected(|i| keys.get(i as usize), &ins)? {
        return Ok(None);
    }

    let program = *keys
        .get(ins.program_index() as usize)
        .context("Couldn't get program ID for instruction")?;

    let accounts = ins
        .account_indices()
        .map(|i| {
            keys.get(i as usize)
                .copied()
                .context("Couldn't get input account for instruction")
        })
        .collect::<Result<Vec<_>>>()?;

    let data = ins.data().to_vec();

    Ok(Some(Message::InstructionNotify(InstructionNotify {
        program,
        data,
        accounts,
        slot,
        txn_signature: txn_signature.to_vec(),
        index,
    })))
}

pub async fn process(client: &Client, slot: u64) -> Result<()> {
    if client.ins_sel().is_empty() {
        debug!("Skipping block reindex for {slot:?} due to empty selector");
        return Ok(());
    }

    debug!("Reindexing slot {slot:?}");

    let block = client.run_rpc(move |r| r.get_block(slot)).await;

    debug!("Block data: {block:?}");

    let block = block?;

    for tx in block.transactions {
        let Some(meta) = tx.meta else {
            continue;
        };

        if meta.err.is_some() {
            continue;
        }

        let Some(tx) = tx.transaction.decode() else {
            continue;
        };

        let Some(signature) = tx.signatures.get(0) else {
            continue;
        };

        let keys = tx.message.account_keys;

        for ins in tx
            .message
            .instructions
            .iter()
            .enumerate()
            .map(|(i, ins)| Result::<_>::Ok((InstructionIndex::TopLevel(i), ins.into())))
            .chain(meta.inner_instructions.iter().flatten().flat_map(|ins| {
                ins.instructions.iter().enumerate().map(|(i, inner)| {
                    Ok((
                        InstructionIndex::Inner(ins.index, i),
                        InstructionShim::try_from_ui(inner, &keys)?,
                    ))
                })
            }))
        {
            match process_instruction(client.ins_sel(), ins, &keys, slot, signature.as_ref()) {
                Ok(Some(m)) => {
                    todo!("send {m:?}");
                },
                Ok(None) => (),
                Err(e) => {
                    warn!("Error processing instruction: {:?}", e);
                },
            }
        }
    }

    Ok(())
}
