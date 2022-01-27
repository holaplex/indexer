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
            .map(|s| {
                s.parse()
                    .map(|k: Pubkey| k.to_bytes().to_vec().into_boxed_slice())
            })
            .collect::<Result<_, _>>()
            .context("Failed to parse account owner keys")?;

        Ok(Self { owners })
    }

    #[inline]
    pub fn is_selected(&self, acct: &ReplicaAccountInfo) -> bool {
        self.owners.contains(acct.owner)
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
    pub fn is_selected(&self, _ins: &CompiledInstruction, pgm: &Pubkey, _accts: &[Pubkey]) -> bool {
        self.programs.contains(pgm)
    }
}
