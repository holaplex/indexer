//! Miscellaneous utility functions.
#![allow(dead_code)]

use indexer_core::prelude::*;
use solana_program::{account_info::AccountInfo, pubkey::Pubkey};

/// Borrow an account's raw as a `solana-program` account info struct.
///
/// # Errors
/// Fails if Tokio encounters an error spawning a blocking thread.
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
