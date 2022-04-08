use cardinal_use_invalidator::state::UseInvalidator as UseInvalidatorAccount;
use indexer_core::{
    db::{insert_into, models::CardinalUseInvalidator, tables::cardinal_use_invalidators},
    prelude::*,
};

use super::Client;
use crate::prelude::*;

pub(crate) async fn process(
    client: &Client,
    key: Pubkey,
    use_invalidator: UseInvalidatorAccount,
) -> Result<()> {
    let row = CardinalUseInvalidator {
        address: Owned(bs58::encode(key).into_string()),
        bump: use_invalidator.bump.try_into()?,
        token_manager_address: Owned(bs58::encode(use_invalidator.token_manager).into_string()),
        use_invalidator_payment_manager: Owned(
            bs58::encode(use_invalidator.payment_manager).into_string(),
        ),
        use_invalidator_collector: Owned(bs58::encode(use_invalidator.collector).into_string()),
        use_invalidator_use_authority: use_invalidator
            .use_authority
            .map(|m| Owned(bs58::encode(m).into_string())),
        use_invalidator_usages: use_invalidator.usages.try_into()?,
        use_invalidator_total_usages: use_invalidator
            .total_usages
            .map(TryFrom::try_from)
            .transpose()?,
        use_invalidator_extension_payment_amount: use_invalidator
            .extension_payment_amount
            .map(TryFrom::try_from)
            .transpose()?,
        use_invalidator_extension_payment_mint: use_invalidator
            .extension_payment_mint
            .map(|m| Owned(bs58::encode(m).into_string())),
        use_invalidator_extension_usages: use_invalidator
            .extension_usages
            .map(TryFrom::try_from)
            .transpose()?,
        use_invalidator_max_usages: use_invalidator
            .max_usages
            .map(TryFrom::try_from)
            .transpose()?,
    };
    client
        .db()
        .run(move |db| {
            insert_into(cardinal_use_invalidators::table)
                .values(&row)
                .on_conflict(cardinal_use_invalidators::address)
                .do_update()
                .set(&row)
                .execute(db)
        })
        .await
        .context("Failed to insert UseInvalidator")?;

    Ok(())
}
