use std::collections::HashSet;

use solana_program::instruction::CompiledInstruction;

use crate::{interface::ReplicaAccountInfo, prelude::*};

#[derive(Debug)]
pub struct AccountSelector {
    owners: HashSet<Box<[u8]>>,
}

impl AccountSelector {
    pub fn from_config(owners: HashSet<String>) -> Result<Self> {
        let owners = owners
            .into_iter()
            .map(|s| bs58::decode(s).into_vec().map(Vec::into_boxed_slice))
            .collect::<Result<_, _>>()
            .context("Failed to parse account owner keys")?;

        Ok(Self { owners })
    }

    #[inline]
    pub fn is_selected(&self, acct: &ReplicaAccountInfo) -> bool {
        self.owners.is_empty() || self.owners.contains(acct.pubkey)
    }
}

#[derive(Debug)]
pub struct InstructionSelector {
    programs: HashSet<[u8; 32]>,
}

impl InstructionSelector {
    pub fn from_config(programs: HashSet<String>) -> Result<Self> {
        let programs = programs
            .into_iter()
            .map(|s| s.parse().map(Pubkey::to_bytes))
            .collect::<Result<_, _>>()
            .context("Failed to parse instruction program keys")?;

        Ok(Self { programs })
    }

    #[inline]
    pub fn is_selected(
        &self,
        _ins: &CompiledInstruction,
        pgm: &[u8; 32],
        _accts: &[[u8; 32]],
    ) -> bool {
        self.programs.is_empty() || self.programs.contains(pgm)
    }
}
