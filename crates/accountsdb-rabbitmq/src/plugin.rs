use indexer_rabbitmq::{
    accountsdb::{Message, Producer, QueueType},
    lapin::{Connection, ConnectionProperties},
    prelude::*,
};
use lapinou::LapinSmolExt;
use solana_program::{instruction::CompiledInstruction, message::SanitizedMessage};

use crate::{
    config::Config,
    interface::{
        AccountsDbPlugin, AccountsDbPluginError, ReplicaAccountInfoVersions,
        ReplicaTransactionInfoVersions, Result,
    },
    prelude::*,
    selectors::{AccountSelector, InstructionSelector},
};

fn custom_err(
    e: impl Into<Box<dyn std::error::Error + Send + Sync + 'static>>,
) -> AccountsDbPluginError {
    AccountsDbPluginError::Custom(e.into())
}

/// An instance of the plugin
#[derive(Debug, Default)]
pub struct AccountsDbPluginRabbitMq {
    producer: Option<Producer>,
    acct_sel: Option<AccountSelector>,
    ins_sel: Option<InstructionSelector>,
}

impl AccountsDbPlugin for AccountsDbPluginRabbitMq {
    fn name(&self) -> &'static str {
        "AccountsDbPluginRabbitMq"
    }

    fn on_load(&mut self, cfg: &str) -> Result<()> {
        solana_logger::setup_with_default("info");

        let (amqp, acct, ins) = Config::read(cfg)
            .and_then(Config::into_parts)
            .map_err(custom_err)?;

        self.acct_sel = Some(acct);
        self.ins_sel = Some(ins);

        smol::block_on(async {
            let conn =
                Connection::connect(&amqp.address, ConnectionProperties::default().with_smol())
                    .await
                    .map_err(custom_err)?;

            self.producer = Some(
                QueueType::new(amqp.network)
                    .producer(&conn)
                    .await
                    .map_err(custom_err)?,
            );

            Ok(())
        })
    }

    fn update_account(
        &mut self,
        account: ReplicaAccountInfoVersions,
        // TODO: is this the account slot or the current slot?
        _slot: u64,
        _is_startup: bool,
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

                    self.producer
                        .as_ref()
                        .ok_or_else(uninit)?
                        .write(Message::AccountUpdate {
                            key: acct.pubkey.try_into().map_err(custom_err)?,
                            owner: acct.owner.try_into().map_err(custom_err)?,
                            data: acct.data.to_owned(),
                        })
                        .await
                        .map_err(custom_err)?;
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
            prod: &Producer,
        ) -> StdResult<(), anyhow::Error> {
            // TODO: no clue if this is right.
            let program = msg
                .get_account_key(ins.program_id_index as usize)
                .ok_or_else(|| anyhow!("Couldn't get program ID for instruction"))?
                .to_bytes();

            // TODO: ...or this.
            let accounts = ins
                .accounts
                .iter()
                .map(|i| {
                    msg.get_account_key(*i as usize).map_or_else(
                        || Err(anyhow!("Couldn't get input account for instruction")),
                        |k| Ok(k.to_bytes()),
                    )
                })
                .collect::<StdResult<Vec<_>, _>>()?;

            if !sel.is_selected(ins, &program, &accounts) {
                return Ok(());
            }

            prod.write(Message::InstructionNotify {
                program,
                data: ins.data.clone(),
                accounts,
            })
            .await
            .map_err(custom_err)?;

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
