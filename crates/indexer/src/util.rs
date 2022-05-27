//! Miscellaneous utility functions.
#![allow(dead_code)]

use indexer_core::prelude::*;
use mpl_token_metadata::state::{Key, MasterEditionV1, MasterEditionV2};
use solana_program::{account_info::AccountInfo, entrypoint::ProgramResult, pubkey::Pubkey};

/// Borrow an account's raw as a `solana-program` account info struct.
#[inline]
pub async fn account_data_as_info<T: Send + 'static>(
    key: Pubkey,
    mut data: Vec<u8>,
    owner: Pubkey,
    mut lamports: u64,
    f: impl Send + FnOnce(AccountInfo<'_>) -> T + 'static,
) -> Result<T> {
    // NOTE: this is here because metaplex_auction only allows parsing via
    //       AccountInfo, which stores lamports as Rc<RefCell<&mut u64>>, which
    //       does not implement Send and therefore cannot ever be used in
    //       multithreaded code.  Thus the only way to access it is to spawn a
    //       dedicated thread in Tokio's thread pool to run all code that needs
    //       to use the account info and prevent the need for a Send impl.

    tokio::task::spawn_blocking(move || {
        let inf = AccountInfo::new(
            &key,
            false,
            false,
            &mut lamports,
            &mut data,
            &owner,
            false,
            0,
        );

        f(inf)
    })
    .await
    .context("Failed to spawn dedicated thread for AccountInfo processing")
}

/// Convenience wrapper for Metaplex's [`MasterEdition`] trait and structs
#[derive(Debug)]
pub enum MasterEdition {
    /// A v1 master edition
    V1(MasterEditionV1),
    /// A v2 master edition
    V2(MasterEditionV2),
}

impl MasterEdition {
    /// Construct the correct master edition from an account
    ///
    /// # Errors
    /// This function fails if the account cannot be parsed as a v1 account or a v2 account.
    pub fn from_account_info(
        info: &AccountInfo,
    ) -> Result<Self, solana_program::program_error::ProgramError> {
        MasterEditionV2::from_account_info(info)
            .map(Self::V2)
            .or_else(|e| {
                debug!("Couldn't parse MasterEditionV2: {:?}", e);
                MasterEditionV1::from_account_info(info).map(Self::V1)
            })
    }
}

impl mpl_token_metadata::state::MasterEdition for MasterEdition {
    fn key(&self) -> Key {
        match self {
            Self::V1(m) => m.key(),
            Self::V2(m) => m.key(),
        }
    }

    fn supply(&self) -> u64 {
        match self {
            Self::V1(m) => m.supply(),
            Self::V2(m) => m.supply(),
        }
    }

    fn set_supply(&mut self, supply: u64) {
        match self {
            Self::V1(m) => m.set_supply(supply),
            Self::V2(m) => m.set_supply(supply),
        }
    }

    fn max_supply(&self) -> Option<u64> {
        match self {
            Self::V1(m) => m.max_supply(),
            Self::V2(m) => m.max_supply(),
        }
    }

    fn save(&self, account: &AccountInfo) -> ProgramResult {
        match self {
            Self::V1(m) => m.save(account),
            Self::V2(m) => m.save(account),
        }
    }
}
