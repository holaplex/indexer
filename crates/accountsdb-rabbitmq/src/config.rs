use std::collections::HashSet;

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
    account_owners: HashSet<String>,

    #[serde(default)]
    instruction_programs: HashSet<String>,
}

#[serde_with::serde_as]
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Amqp {
    pub address: String,

    #[serde_as(as = "serde_with::DisplayFromStr")]
    pub network: indexer_rabbitmq::accountsdb::Network,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Jobs {
    pub limit: usize,
}

impl Config {
    pub fn read(path: &str) -> Result<Self> {
        let f = std::fs::File::open(path).context("Failed to open config file")?;
        let cfg = serde_json::from_reader(f).context("Failed to parse config file")?;

        Ok(cfg)
    }

    pub fn into_parts(self) -> Result<(Amqp, Jobs, AccountSelector, InstructionSelector)> {
        let Self {
            amqp,
            jobs,
            account_owners,
            instruction_programs,
        } = self;

        let acct = AccountSelector::from_config(account_owners)
            .context("Failed to create account selector")?;
        let ins = InstructionSelector::from_config(instruction_programs)
            .context("Failed to create instruction selector")?;

        Ok((amqp, jobs, acct, ins))
    }
}
