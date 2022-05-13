use hashbrown::HashSet;
use indexer_rabbitmq::geyser::StartupType;

use super::config::Accounts;
use crate::{interface::ReplicaAccountInfo, prelude::*};

#[derive(Debug)]
pub struct AccountSelector {
    owners: HashSet<[u8; 32]>,
    startup: Option<bool>,
}

impl AccountSelector {
    pub fn from_config(config: Accounts) -> Result<Self> {
        let Accounts { owners, startup } = config;

        let owners = owners
            .into_iter()
            .map(|s| s.parse().map(Pubkey::to_bytes))
            .collect::<Result<_, _>>()
            .context("Failed to parse account owner keys")?;

        Ok(Self { owners, startup })
    }

    #[inline]
    pub fn startup(&self) -> StartupType {
        StartupType::new(self.startup)
    }

    #[inline]
    pub fn is_selected(&self, acct: &ReplicaAccountInfo, is_startup: bool) -> bool {
        self.startup.map_or(true, |s| is_startup == s) && self.owners.contains(acct.owner)
    }
}

#[derive(Debug)]
pub struct InstructionSelector {
    programs: HashSet<Pubkey>,
}

impl InstructionSelector {
    pub fn from_config(programs: HashSet<String>) -> Result<Self> {
        let programs = programs
            .into_iter()
            .map(|s| s.parse())
            .collect::<Result<_, _>>()
            .context("Failed to parse instruction program keys")?;

        Ok(Self { programs })
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.programs.is_empty()
    }

    #[inline]
    pub fn is_selected(&self, pgm: &Pubkey) -> bool {
        self.programs.contains(pgm)
    }
}
