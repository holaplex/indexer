use serde::Deserialize;
use url::Url;

use crate::prelude::*;

mod queries;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Boot {
    pub name: String,
    pub config_url: Url,
}

impl Boot {
    pub fn read(path: &str) -> Result<Self> {
        let f = std::fs::File::open(path).context("Failed to open config file")?;
        let cfg = serde_json::from_reader(f).context("Failed to parse config file")?;

        Ok(cfg)
    }

    // pub fn into_parts(self) -> Result<(Amqp, Jobs, Metrics, AccountSelector, InstructionSelector)> {
    //     let Self {
    //         amqp,
    //         jobs,
    //         metrics,
    //         accounts,
    //         instruction_programs,
    //     } = self;

    //     let acct =
    //         AccountSelector::from_config(accounts).context("Failed to create account selector")?;
    //     let ins = InstructionSelector::from_config(instruction_programs)
    //         .context("Failed to create instruction selector")?;

    //     Ok((amqp, jobs, metrics, acct, ins))
    // }
}
