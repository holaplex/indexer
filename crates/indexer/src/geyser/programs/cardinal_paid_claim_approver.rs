use ::cardinal_paid_claim_approver::state::PaidClaimApprover;
use anchor_lang_v0_22_1::{AccountDeserialize, Discriminator};

use super::{accounts::cardinal_paid_claim_approver, AccountUpdate, Client};
use crate::prelude::*;

pub(crate) async fn process(client: &Client, update: AccountUpdate) -> Result<()> {
    let account_discriminator = &update.data[..8];

    if account_discriminator == PaidClaimApprover::discriminator() {
        let claim_approver: PaidClaimApprover =
            PaidClaimApprover::try_deserialize(&mut update.data.as_slice())
                .context("Failed to deserialize claim_approver")?;

        cardinal_paid_claim_approver::process(client, update.key, claim_approver).await?;
    }

    Ok(())
}
