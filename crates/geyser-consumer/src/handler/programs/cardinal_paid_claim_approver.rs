use ::cardinal_paid_claim_approver::state::PaidClaimApprover;
use anchor_lang_v0_24::{AccountDeserialize, Discriminator};
use indexer::prelude::*;

use super::{accounts::cardinal_paid_claim_approver, AccountUpdate, Client};

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
