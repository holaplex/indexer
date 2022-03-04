use std::{
    collections::HashSet,
    env,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Mutex,
    },
    time::{Duration, Instant},
};

use indexer_rabbitmq::{
    accountsdb::{AccountUpdate, Message, Producer, QueueType},
    lapin::{Connection, ConnectionProperties},
    prelude::*,
};
use lapinou::LapinSmolExt;
use lazy_static::lazy_static;
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

lazy_static! {
    static ref MSG: Mutex<Instant> = Mutex::new(Instant::now());
}

static MESSAGES_SENT: AtomicUsize = AtomicUsize::new(0);
static MESSAGES_RECEIVED: AtomicUsize = AtomicUsize::new(0);

fn update_message_count() {
    *MSG.lock().unwrap() = Instant::now();
}

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
    token_addresses: HashSet<Pubkey>,
}

#[derive(Deserialize)]
struct TokenItem {
    address: String,
}

#[derive(Deserialize)]
struct TokenList {
    tokens: Vec<TokenItem>,
}

impl AccountsDbPluginRabbitMq {
    const TOKEN_REG_URL: &'static str = "https://raw.githubusercontent.com/solana-labs/token-list/main/src/tokens/solana.tokenlist.json";

    fn load_token_reg() -> Result<HashSet<Pubkey>> {
        // We use `smol` as an executor, and reqwest's async backend doesn't like that
        let res: TokenList = reqwest::blocking::get(Self::TOKEN_REG_URL)
            .map_err(custom_err)?
            .json()
            .map_err(custom_err)?;

        res.tokens
            .into_iter()
            .map(|TokenItem { address }| address.parse())
            .collect::<StdResult<_, _>>()
            .map_err(custom_err)
    }
}

impl AccountsDbPlugin for AccountsDbPluginRabbitMq {
    fn name(&self) -> &'static str {
        "AccountsDbPluginRabbitMq"
    }

    fn on_load(&mut self, cfg: &str) -> Result<()> {
        solana_logger::setup_with_default("info");

        let (amqp, metric_config, jobs, acct, ins) = Config::read(cfg)
            .and_then(Config::into_parts)
            .map_err(custom_err)?;

        let startup_type = acct.startup();

        env::set_var("SOLANA_METRICS_CONFIG", metric_config.config.to_string());

        self.acct_sel = Some(acct);
        self.ins_sel = Some(ins);

        self.token_addresses = Self::load_token_reg()?;

        smol::block_on(async {
            let conn =
                Connection::connect(&amqp.address, ConnectionProperties::default().with_smol())
                    .await
                    .map_err(custom_err)?;

            self.producer = Some(Sender::new(
                QueueType::new(amqp.network, startup_type, None)
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

        messages_received();

        smol::block_on(async {
            match account {
                ReplicaAccountInfoVersions::V0_0_1(acct) => {
                    if !self
                        .acct_sel
                        .as_ref()
                        .ok_or_else(uninit)?
                        .is_selected(acct, is_startup)
                    {
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

                    if owner == ids::token().as_ref()
                        && data.len() == TokenAccount::get_packed_len()
                    {
                        let token_account = TokenAccount::unpack_from_slice(data);

                        if let Ok(token_account) = token_account {
                            if token_account.amount > 1
                                || self.token_addresses.contains(&token_account.mint)
                            {
                                return Ok(());
                            }
                        }
                    }

                    let key = Pubkey::new_from_array(pubkey.try_into().map_err(custom_err)?);
                    let owner = Pubkey::new_from_array(owner.try_into().map_err(custom_err)?);
                    let data = data.to_owned();

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
                    messages_sent();
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

fn messages_sent() {
    MESSAGES_SENT.fetch_add(1, Ordering::SeqCst);

    if MSG.lock().unwrap().elapsed() >= Duration::from_secs(30) {
        solana_metrics::submit(
            solana_metrics::datapoint::DataPoint::new("accountdb")
                .add_field_i64(
                    "msgs_sent",
                    MESSAGES_SENT.load(Ordering::SeqCst).try_into().unwrap(),
                )
                .to_owned(),
            log::Level::Info,
        );
    }
}

fn messages_received() {
    MESSAGES_RECEIVED.fetch_add(1, Ordering::SeqCst);

    if MSG.lock().unwrap().elapsed() >= Duration::from_secs(30) {
        update_message_count();
    }

    solana_metrics::submit(
        solana_metrics::datapoint::DataPoint::new("accountdb")
            .add_field_i64(
                "msgs_rcvd",
                MESSAGES_RECEIVED.load(Ordering::SeqCst).try_into().unwrap(),
            )
            .to_owned(),
        log::Level::Info,
    );
}
