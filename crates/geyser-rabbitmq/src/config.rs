use hashbrown::HashSet;
use serde::Deserialize;

use crate::{
    prelude::*,
    selectors::{AccountSelector, InstructionSelector},
};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Config {
    amqp: Amqp,
    jobs: Jobs,

    #[serde(default)]
    metrics: Metrics,

    accounts: Accounts,

    instruction_programs: HashSet<String>,
}

#[serde_with::serde_as]
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Amqp {
    pub address: String,

    #[serde_as(as = "serde_with::DisplayFromStr")]
    pub network: indexer_rabbitmq::geyser::Network,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Jobs {
    pub limit: usize,

    #[serde(default)]
    pub blocking: Option<usize>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Metrics {
    pub config: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Accounts {
    pub owners: HashSet<String>,

    /// Filter for changing how to interpret the `is_startup` flag.
    ///
    /// This option has three states:
    ///  - `None`: Ignore the `is_startup` flag and send all updates.
    ///  - `Some(true)`: Only send updates when `is_startup` is `true`.
    ///  - `Some(false)`: Only send updates when `is_startup` is `false`.
    pub startup: Option<bool>,
}

impl Config {
    pub fn read(path: &str) -> Result<Self> {
        let f = std::fs::File::open(path).context("Failed to open config file")?;
        let cfg = serde_json::from_reader(f).context("Failed to parse config file")?;

        Ok(cfg)
    }

    pub fn into_parts(self) -> Result<(Amqp, Jobs, Metrics, AccountSelector, InstructionSelector)> {
        let Self {
            amqp,
            jobs,
            metrics,
            accounts,
            instruction_programs,
        } = self;

        let acct =
            AccountSelector::from_config(accounts).context("Failed to create account selector")?;
        let ins = InstructionSelector::from_config(instruction_programs)
            .context("Failed to create instruction selector")?;

        Ok((amqp, jobs, metrics, acct, ins))
    }
}
