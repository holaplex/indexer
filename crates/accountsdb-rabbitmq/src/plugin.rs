use crate::interface::{AccountsDbPlugin, ReplicaAccountInfoVersions, Result};

/// An instance of the plugin
#[derive(Debug, Default)]
pub struct AccountsDbPluginRabbitMq {
    todo: Vec<()>, // TODO
}

impl AccountsDbPlugin for AccountsDbPluginRabbitMq {
    fn name(&self) -> &'static str {
        "AccountsDbPluginRabbitMq"
    }

    fn update_account(
        &mut self,
        account: ReplicaAccountInfoVersions,
        slot: u64,
        is_startup: bool,
    ) -> Result<()> {
        Ok(())
    }
}
