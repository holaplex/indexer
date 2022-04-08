use cardinal_paid_claim_approver::state::PaidClaimApprover as PaidClaimApproverAccount;
use indexer_core::{
    db::{insert_into, models::CardinalPaidClaimApprover, tables::cardinal_paid_claim_approvers},
    prelude::*,
};

use super::Client;
use crate::prelude::*;

pub(crate) async fn process(
    client: &Client,
    key: Pubkey,
    paid_claim_approver: PaidClaimApproverAccount,
) -> Result<()> {
    let row = CardinalPaidClaimApprover {
        address: Owned(bs58::encode(key).into_string()),
        bump: paid_claim_approver.bump.try_into()?,
        token_manager_address: Owned(bs58::encode(paid_claim_approver.token_manager).into_string()),
        paid_claim_approver_payment_manager: Owned(
            bs58::encode(paid_claim_approver.payment_manager).into_string(),
        ),
        paid_claim_approver_collector: Owned(
            bs58::encode(paid_claim_approver.collector).into_string(),
        ),
        paid_claim_approver_payment_amount: paid_claim_approver.payment_amount.try_into()?,
        paid_claim_approver_payment_mint: Owned(
            bs58::encode(paid_claim_approver.payment_mint).into_string(),
        ),
    };
    debug!("Paid Claim Approver {:?}", row);
    client
        .db()
        .run(move |db| {
            insert_into(cardinal_paid_claim_approvers::table)
                .values(&row)
                .on_conflict(cardinal_paid_claim_approvers::address)
                .do_update()
                .set(&row)
                .execute(db)
        })
        .await
        .context("Failed to insert PaidClaimApprover")?;

    Ok(())
}
