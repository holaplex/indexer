use indexer_core::{
    db::{
        insert_into,
        models::CurrentMetadataOwner,
        tables::{current_metadata_owners, metadatas},
        update,
    },
    prelude::*,
};
use spl_token::state::Account as TokenAccount;

use super::Client;
use crate::prelude::*;

pub async fn process(
    client: &Client,
    key: Pubkey,
    token_account: TokenAccount,
    slot: u64,
) -> Result<()> {
    let pubkey = key.to_string();

    let amount: i64 = token_account
        .amount
        .try_into()
        .context("Token amount was too big to store")?;

    if amount != 1 {
        return Ok(());
    }

    let owner = token_account.owner.to_string();
    let mint_address = token_account.mint.to_string();
    let incoming_slot: i64 = slot.try_into()?;

    let values = CurrentMetadataOwner {
        mint_address: Owned(mint_address),
        owner_address: Owned(owner),
        token_account_address: Owned(pubkey),
        slot: incoming_slot,
    };

    client
        .db()
        .run(move |db| {
            let rows = current_metadata_owners::table
                .select((
                    current_metadata_owners::mint_address,
                    current_metadata_owners::owner_address,
                    current_metadata_owners::token_account_address,
                    current_metadata_owners::slot,
                ))
                .filter(current_metadata_owners::mint_address.eq(token_account.mint.to_string()))
                .load::<CurrentMetadataOwner>(db)
                .context("failed to load metadata owner!")?;

            match rows.get(0) {
                Some(r) if incoming_slot > r.slot => {
                    db.build_transaction().read_write().run(|| {
                        update(
                            current_metadata_owners::table
                                .filter(current_metadata_owners::mint_address.eq(values.clone().mint_address)),
                        )
                        .set(&values)
                        .execute(db)
                        .context("transaction failed! unable to update metadata_owners when incoming slot > indexed slot")
                        .map(|_| ())
                    })
                },
                Some(_) => Ok(()),
                None => {
                    db.build_transaction()
                        .read_write()
                        .run(|| {
                            insert_into(current_metadata_owners::table)
                                .values(&values)
                                .on_conflict(current_metadata_owners::mint_address)
                                .do_update()
                                .set(&values)
                                .execute(db)
                                .map(|_| ())
                        })
                        .context("transaction failed! unable to insert metadata owner")?;

                    Ok(())
                },
            }
        })
        .await
        .context("failed to insert token metadata owner!")?;
    Ok(())
}

pub(crate) async fn process_burn_instruction(
    client: &Client,
    accounts: &[Pubkey],
    slot: u64,
) -> Result<()> {
    if accounts.len() != 3 {
        return Ok(());
    }

    let mint = accounts[2].to_string();
    let slot = i64::try_from(slot)?;

    client
        .db()
        .run(move |db| {
            update(metadatas::table.filter(metadatas::mint_address.eq(mint)))
                .set((metadatas::burned.eq(true), metadatas::slot.eq(slot)))
                .execute(db)
        })
        .await
        .context("failed to update metadata")?;

    Ok(())
}
