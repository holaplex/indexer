use anchor_lang_v0_22_1::{solana_program::hash::hash, AccountDeserialize};
use cardinal_paid_claim_approver::state::PaidClaimApprover;

use super::{accounts::paid_claim_approver, AccountUpdate, Client};
use crate::prelude::*;

pub(crate) async fn process(client: &Client, update: AccountUpdate) -> Result<()> {
    let claim_approver_discriminator: &[u8] =
        &hash("account:PaidClaimApprover".as_bytes()).to_bytes()[..8];
    let account_discriminator = &update.data[..8];
    if account_discriminator == claim_approver_discriminator {
        let claim_approver: PaidClaimApprover =
            PaidClaimApprover::try_deserialize(&mut update.data.as_slice())
                .context("Failed to deserialize claim_approver")?;
        paid_claim_approver::process(client, update.key, claim_approver).await?
    }
    Ok(())
}
