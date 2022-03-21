use std::{
    collections::HashSet,
    env,
    sync::atomic::{AtomicUsize, Ordering},
    time::{Duration, Instant},
};

use indexer_rabbitmq::{
    accountsdb::{AccountUpdate, Message, Producer, QueueType},
    lapin::{Connection, ConnectionProperties},
};
use smol::lock::Mutex;
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

#[derive(Debug)]
struct Metrics {
    last_send: Mutex<Instant>,
    last_recv: Mutex<Instant>,
    send_count: AtomicUsize,
    recv_count: AtomicUsize,
}

impl Metrics {
    async fn try_submit(last: &Mutex<Instant>, f: impl FnOnce()) {
        let now = Instant::now();
        let mut time = last.lock().await;

        if now - *time >= Duration::from_secs(30) {
            *time = now;
            f();
        }
    }

    async fn log_send(&self) {
        let n = self.send_count.fetch_add(1, Ordering::SeqCst) + 1;

        Self::try_submit(&self.last_send, || {
            solana_metrics::datapoint_info!(
                "accountsdb_rabbitmq",
                ("msgs_sent", i64::try_from(n).unwrap(), i64),
            );
        })
        .await;
    }

    async fn log_recv(&self) {
        let n = self.recv_count.fetch_add(1, Ordering::SeqCst) + 1;

        Self::try_submit(&self.last_recv, || {
            solana_metrics::datapoint_info!(
                "accountsdb_rabbitmq",
                ("msgs_sent", i64::try_from(n).unwrap(), i64),
            );
        })
        .await;
    }
}

/// An instance of the plugin
#[derive(Debug, Default)]
pub struct AccountsDbPluginRabbitMq {
    producer: Option<Sender<Producer>>,
    acct_sel: Option<AccountSelector>,
    ins_sel: Option<InstructionSelector>,
    metrics: Option<Metrics>,
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

        let (amqp, jobs, metrics, acct, ins) = Config::read(cfg)
            .and_then(Config::into_parts)
            .map_err(custom_err)?;

        let startup_type = acct.startup();

        if let Some(config) = metrics.config {
            const VAR: &str = "SOLANA_METRICS_CONFIG";

            if env::var_os(VAR).is_some() {
                warn!("Overriding existing value for {}", VAR);
            }

            env::set_var(VAR, config);
        }

        self.acct_sel = Some(acct);
        self.ins_sel = Some(ins);

        self.token_addresses = Self::load_token_reg()?;

        self.metrics = Some(Metrics {
            last_send: Mutex::new(Instant::now()),
            last_recv: Mutex::new(Instant::now()),
            send_count: AtomicUsize::new(0),
            recv_count: AtomicUsize::new(0),
        });

        smol::block_on(async {
            let conn = Connection::connect(
                &amqp.address,
                ConnectionProperties::default().with_executor(smol_executor_trait::Smol),
            )
            .await
            .map_err(custom_err)?;

            self.producer = Some(Sender::new(
                Producer::new(&conn, QueueType::new(amqp.network, startup_type, None))
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
            let metrics = self.metrics.as_ref().ok_or_else(uninit)?;

            metrics.log_recv().await;

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

                    metrics.log_send().await;
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
