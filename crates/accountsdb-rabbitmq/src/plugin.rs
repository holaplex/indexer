use indexer_rabbitmq::{
    accountsdb::{Message, Producer},
    lapin::{Connection, ConnectionProperties},
};
use lapinou::LapinSmolExt;

use crate::interface::{
    AccountsDbPlugin, AccountsDbPluginError, ReplicaAccountInfoVersions, Result,
};

fn custom_err(e: impl std::error::Error + Send + Sync + 'static) -> AccountsDbPluginError {
    AccountsDbPluginError::Custom(Box::new(e))
}

/// An instance of the plugin
#[derive(Debug, Default)]
pub struct AccountsDbPluginRabbitMq {
    producer: Option<Producer>,
}

impl AccountsDbPlugin for AccountsDbPluginRabbitMq {
    fn name(&self) -> &'static str {
        "AccountsDbPluginRabbitMq"
    }

    fn on_load(&mut self, _cfg: &str) -> Result<()> {
        smol::block_on(async {
            let conn = Connection::connect("todo", ConnectionProperties::default().with_smol())
                .await
                .map_err(custom_err)?;

            self.producer = Some(Producer::new(&conn).await.map_err(custom_err)?);

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
}
