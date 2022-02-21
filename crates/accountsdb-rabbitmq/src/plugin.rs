use std::{collections::HashMap, io::Read};

use indexer_rabbitmq::{
    accountsdb::{AccountUpdate, Message, Producer, QueueType},
    lapin::{Connection, ConnectionProperties},
    prelude::*,
};
use lapinou::LapinSmolExt;
use solana_program::{
    instruction::CompiledInstruction, message::SanitizedMessage, program_pack::Pack,
};
use spl_token::state::Account as TokenAccount;

mod ids {
    #![allow(missing_docs)]
    use solana_sdk::pubkeys;
    pubkeys!(token, "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA");
}

use serde::Deserialize;

use crate::{
    config::Config,
    interface::{
        AccountsDbPlugin, AccountsDbPluginError, ReplicaAccountInfo, ReplicaAccountInfoVersions,
        ReplicaTransactionInfoVersions, Result,
    },
    prelude::*,
    selectors::{AccountSelector, InstructionSelector},
    sender::Sender,
};

fn custom_err(
    e: impl Into<Box<dyn std::error::Error + Send + Sync + 'static>>,
) -> AccountsDbPluginError {
    AccountsDbPluginError::Custom(e.into())
}

/// An instance of the plugin
#[derive(Debug, Default)]
pub struct AccountsDbPluginRabbitMq {
    producer: Option<Sender<Producer>>,
    acct_sel: Option<AccountSelector>,
    ins_sel: Option<InstructionSelector>,
    token_addresses: HashMap<String, bool>,
}

#[derive(Deserialize)]
struct TokenItem {
    address: String,
}

#[derive(Deserialize)]
struct TokenList {
    tokens: Vec<TokenItem>,
}

impl AccountsDbPlugin for AccountsDbPluginRabbitMq {
    fn name(&self) -> &'static str {
        "AccountsDbPluginRabbitMq"
    }

    fn on_load(&mut self, cfg: &str) -> Result<()> {
        solana_logger::setup_with_default("info");

        let (amqp, jobs, acct, ins) = Config::read(cfg)
            .and_then(Config::into_parts)
            .map_err(custom_err)?;

        self.acct_sel = Some(acct);
        self.ins_sel = Some(ins);

        let mut res = reqwest::blocking::get("https://raw.githubusercontent.com/solana-labs/token-list/main/src/tokens/solana.tokenlist.json")
            .expect("couldn't fetch token list");
        let mut body = String::new();
        res.read_to_string(&mut body)?;

        let json: TokenList = serde_json::from_str(&body).expect("file should be proper JSON");

        let mut token_addresses: HashMap<String, bool> = HashMap::new();
        for token_data in json.tokens {
            let address = token_data.address;
            token_addresses.insert(address, true);
        }

        self.token_addresses = token_addresses;

        smol::block_on(async {
            let conn =
                Connection::connect(&amqp.address, ConnectionProperties::default().with_smol())
                    .await
                    .map_err(custom_err)?;

            self.producer = Some(Sender::new(
                QueueType::new(amqp.network, None)
                    .producer(&conn)
                    .await
                    .map_err(custom_err)?,
                jobs.limit,
            ));

            Ok(())
        })
    }

    fn update_account(
        &mut self,
        account: ReplicaAccountInfoVersions,
        slot: u64,
        is_startup: bool,
    ) -> Result<()> {
        fn uninit() -> AccountsDbPluginError {
            AccountsDbPluginError::AccountsUpdateError {
                msg: "RabbitMQ plugin not initialized yet!".into(),
            }
        }

        smol::block_on(async {
            match account {
                ReplicaAccountInfoVersions::V0_0_1(acct) => {
                    if !self.acct_sel.as_ref().ok_or_else(uninit)?.is_selected(acct) {
                        return Ok(());
                    }

                    let ReplicaAccountInfo {
                        pubkey,
                        lamports,
                        owner,
                        executable,
                        rent_epoch,
                        data,
                        write_version,
                    } = *acct;

                    let key = Pubkey::new_from_array(pubkey.try_into().map_err(custom_err)?);
                    let owner = Pubkey::new_from_array(owner.try_into().map_err(custom_err)?);
                    let data = data.to_owned();

                    if owner == ids::token() && data.len() == TokenAccount::get_packed_len() {
                        let token_account = TokenAccount::unpack_from_slice(&data);

                        if let Ok(token_account) = token_account {
                            if token_account.amount > 1 {
                                return Ok(());
                            }
                            let mint = token_account.mint.to_string();
                            if self.token_addresses.contains_key(&mint) {
                                return Ok(());
                            }
                        }
                    }

                    self.producer
                        .as_ref()
                        .ok_or_else(uninit)?
                        .run(move |prod| async move {
                            prod.write(Message::AccountUpdate(AccountUpdate {
                                key,
                                lamports,
                                owner,
                                executable,
                                rent_epoch,
                                data,
                                write_version,
                                slot,
                                is_startup,
                            }))
                            .await
                            .map_err(Into::into)
                        })
                        .await;
                },
            }

            Ok(())
        })
    }

    fn notify_transaction(
        &mut self,
        transaction: ReplicaTransactionInfoVersions,
        _slot: u64,
    ) -> Result<()> {
        fn uninit() -> AccountsDbPluginError {
            AccountsDbPluginError::Custom(anyhow!("RabbitMQ plugin not initialized yet!").into())
        }

        #[inline]
        async fn process_instruction(
            ins: &CompiledInstruction,
            sel: &InstructionSelector,
            msg: &SanitizedMessage,
            prod: &Sender<Producer>,
        ) -> StdResult<(), anyhow::Error> {
            // TODO: no clue if this is right.
            let program = *msg
                .get_account_key(ins.program_id_index as usize)
                .ok_or_else(|| anyhow!("Couldn't get program ID for instruction"))?;

            // TODO: ...or this.
            let accounts = ins
                .accounts
                .iter()
                .map(|i| {
                    msg.get_account_key(*i as usize).map_or_else(
                        || Err(anyhow!("Couldn't get input account for instruction")),
                        |k| Ok(*k),
                    )
                })
                .collect::<StdResult<Vec<_>, _>>()?;

            if !sel.is_selected(ins, &program, &accounts) {
                return Ok(());
            }

            let data = ins.data.clone();

            prod.run(|prod| async move {
                prod.write(Message::InstructionNotify {
                    program,
                    data,
                    accounts,
                })
                .await
                .map_err(Into::into)
            })
            .await;

            Ok(())
        }

        smol::block_on(async {
            match transaction {
                ReplicaTransactionInfoVersions::V0_0_1(tx) => {
                    let ins_sel = self.ins_sel.as_ref().ok_or_else(uninit)?;

                    if matches!(tx.transaction_status_meta.status, Err(..)) {
                        return Ok(());
                    }

                    let prod = self.producer.as_ref().ok_or_else(uninit)?;
                    let msg = tx.transaction.message();

                    for ins in msg.instructions().iter().chain(
                        tx.transaction_status_meta
                            .inner_instructions
                            .iter()
                            .flatten()
                            .flat_map(|i| i.instructions.iter()),
                    ) {
                        process_instruction(ins, ins_sel, msg, prod)
                            .await
                            .map_err(|e| debug!("Error processing instruction: {:?}", e))
                            .ok();
                    }
                },
            }

            Ok(())
        })
    }
}
