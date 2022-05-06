use std::{env, sync::Arc};

use hashbrown::HashSet;
use indexer_rabbitmq::geyser::{AccountUpdate, Message};
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
        GeyserPlugin, GeyserPluginError, ReplicaAccountInfo, ReplicaAccountInfoVersions,
        ReplicaTransactionInfoVersions, Result,
    },
    metrics::{Counter, Metrics},
    prelude::*,
    selectors::{AccountSelector, InstructionSelector},
    sender::Sender,
};

#[inline]
fn custom_err<'a, E: Into<Box<dyn std::error::Error + Send + Sync + 'static>>>(
    counter: &'a Counter,
) -> impl FnOnce(E) -> GeyserPluginError + 'a {
    |e| {
        counter.log();
        GeyserPluginError::Custom(e.into())
    }
}

/// An instance of the plugin
#[derive(Debug, Default)]
pub struct GeyserPluginRabbitMq {
    producer: Option<Sender>,
    acct_sel: Option<AccountSelector>,
    ins_sel: Option<InstructionSelector>,
    metrics: Option<Arc<Metrics>>,
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

impl GeyserPluginRabbitMq {
    const TOKEN_REG_URL: &'static str = "https://raw.githubusercontent.com/solana-labs/token-list/main/src/tokens/solana.tokenlist.json";

    fn load_token_reg() -> StdResult<HashSet<Pubkey>, anyhow::Error> {
        // We use `smol` as an executor, and reqwest's async backend doesn't like that
        let res: TokenList = reqwest::blocking::get(Self::TOKEN_REG_URL)?.json()?;

        res.tokens
            .into_iter()
            .map(|TokenItem { address }| address.parse())
            .collect::<StdResult<_, _>>()
            .map_err(Into::into)
    }
}

impl GeyserPlugin for GeyserPluginRabbitMq {
    fn name(&self) -> &'static str {
        "GeyserPluginRabbitMq"
    }

    fn on_load(&mut self, cfg: &str) -> Result<()> {
        solana_logger::setup_with_default("info");

        let metrics = Metrics::new_rc();

        let version;
        let host;

        {
            let ver = env!("CARGO_PKG_VERSION");
            let git = option_env!("META_GIT_HEAD");
            // TODO
            // let rem = option_env!("META_GIT_REMOTE");

            {
                use std::fmt::Write;

                let mut s = format!("v{}", ver);

                if let Some(git) = git {
                    write!(s, "+git.{}", git).unwrap();
                }

                version = s;
            }

            // TODO
            // let rustc_ver = env!("META_RUSTC_VERSION");
            // let build_host = env!("META_BUILD_HOST");
            // let target = env!("META_BUILD_TARGET");
            // let profile = env!("META_BUILD_PROFILE");
            // let platform = env!("META_BUILD_PLATFORM");

            host = hostname::get()
                .map_err(custom_err(&metrics.errs))?
                .into_string()
                .map_err(|_| anyhow!("Failed to parse system hostname"))
                .map_err(custom_err(&metrics.errs))?;
        }

        let (amqp, jobs, metrics_conf, acct, ins) = Config::read(cfg)
            .and_then(Config::into_parts)
            .map_err(custom_err(&metrics.errs))?;

        let startup_type = acct.startup();

        if let Some(config) = metrics_conf.config {
            const VAR: &str = "SOLANA_METRICS_CONFIG";

            if env::var_os(VAR).is_some() {
                warn!("Overriding existing value for {}", VAR);
            }

            env::set_var(VAR, config);
        }

        self.acct_sel = Some(acct);
        self.ins_sel = Some(ins);

        self.token_addresses = Self::load_token_reg().map_err(custom_err(&metrics.errs))?;

        self.metrics = Some(Arc::clone(&metrics));

        smol::block_on(async {
            self.producer = Some(
                Sender::new(
                    amqp,
                    format!("geyser-rabbitmq-{}@{}", version, host),
                    &jobs,
                    startup_type,
                    Arc::clone(&metrics),
                )
                .await
                .map_err(custom_err(&metrics.errs))?,
            );

            Ok(())
        })
    }

    fn update_account(
        &mut self,
        account: ReplicaAccountInfoVersions,
        slot: u64,
        is_startup: bool,
    ) -> Result<()> {
        #[inline]
        fn uninit<'a>(
            counter: impl Into<Option<&'a Counter>> + 'a,
        ) -> impl FnOnce() -> GeyserPluginError + 'a {
            || {
                counter.into().map(Counter::log);

                GeyserPluginError::AccountsUpdateError {
                    msg: "RabbitMQ plugin not initialized yet!".into(),
                }
            }
        }

        smol::block_on(async {
            let metrics = self.metrics.as_ref().ok_or_else(uninit(None))?;

            metrics.recvs.log();

            match account {
                ReplicaAccountInfoVersions::V0_0_1(acct) => {
                    if !self
                        .acct_sel
                        .as_ref()
                        .ok_or_else(uninit(&metrics.errs))?
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

                    let key = Pubkey::new_from_array(
                        pubkey.try_into().map_err(custom_err(&metrics.errs))?,
                    );
                    let owner = Pubkey::new_from_array(
                        owner.try_into().map_err(custom_err(&metrics.errs))?,
                    );
                    let data = data.to_owned();

                    self.producer
                        .as_ref()
                        .ok_or_else(uninit(&metrics.errs))?
                        .send(Message::AccountUpdate(AccountUpdate {
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
                        .await;

                    metrics.sends.log();
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
        #[inline]
        fn uninit<'a>(
            counter: impl Into<Option<&'a Counter>> + 'a,
        ) -> impl FnOnce() -> GeyserPluginError + 'a {
            || {
                counter.into().map(Counter::log);

                GeyserPluginError::Custom(anyhow!("RabbitMQ plugin not initialized yet!").into())
            }
        }

        #[inline]
        async fn process_instruction(
            ins: &CompiledInstruction,
            sel: &InstructionSelector,
            msg: &SanitizedMessage,
            prod: &Sender,
            metrics: &Metrics,
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

            prod.send(Message::InstructionNotify {
                program,
                data,
                accounts,
            })
            .await;

            metrics.sends.log();

            Ok(())
        }

        let metrics = self.metrics.as_ref().ok_or_else(uninit(None))?;
        let ins_sel = self.ins_sel.as_ref().ok_or_else(uninit(&metrics.errs))?;

        if ins_sel.is_empty() {
            return Ok(());
        }

        smol::block_on(async {
            metrics.recvs.log();

            match transaction {
                ReplicaTransactionInfoVersions::V0_0_1(tx) => {
                    if matches!(tx.transaction_status_meta.status, Err(..)) {
                        return Ok(());
                    }

                    let prod = self.producer.as_ref().ok_or_else(uninit(&metrics.errs))?;
                    let msg = tx.transaction.message();

                    for ins in msg.instructions().iter().chain(
                        tx.transaction_status_meta
                            .inner_instructions
                            .iter()
                            .flatten()
                            .flat_map(|i| i.instructions.iter()),
                    ) {
                        process_instruction(ins, ins_sel, msg, prod, metrics)
                            .await
                            .map_err(|e| {
                                debug!("Error processing instruction: {:?}", e);
                                metrics.errs.log();
                            })
                            .ok();
                    }
                },
            }

            Ok(())
        })
    }

    fn account_data_notifications_enabled(&self) -> bool {
        true
    }

    fn transaction_notifications_enabled(&self) -> bool {
        !self
            .ins_sel
            .as_ref()
            .expect("Plugin isn't initialized yet!")
            .is_empty()
    }
}
