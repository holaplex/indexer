//! Miscellaneous utility functions.

use solana_program::account_info::AccountInfo;
use solana_sdk::{account::Account, pubkey::Pubkey};

/// Borrow a `solana-sdk` account as a `solana-program` account info struct.
pub fn account_as_info<'a>(
    key: &'a Pubkey,
    is_signer: bool,
    is_writable: bool,
    acct: &'a mut Account,
) -> AccountInfo<'a> {
    AccountInfo::new(
        key,
        is_signer,
        is_writable,
        &mut acct.lamports,
        &mut *acct.data,
        &acct.owner,
        acct.executable,
        acct.rent_epoch,
    )
}
