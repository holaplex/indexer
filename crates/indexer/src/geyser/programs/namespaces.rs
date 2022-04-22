use ::namespaces::state::{Entry, ENTRY_SIZE};
use anchor_lang::AccountDeserialize;
use indexer_core::{
    db::{insert_into, models::TwitterHandle, tables::twitter_handle_name_services, update as dbUpdate},
    prelude::*,
};

use super::{AccountUpdate, Client};
use crate::prelude::*;
pub(crate) async fn process(client: &Client, update: AccountUpdate) -> Result<()> {
    if update.data.len() == ENTRY_SIZE {
        let entry: Entry = Entry::try_deserialize(&mut update.data.as_slice())
            .context("Failed to deserialize cardinal entry")?;

        debug!("twitter handle: {:?}", entry.name);
        debug!("pubkey: {:?}", entry.data);

        let slot = i64::try_from(update.slot)?;
        let write_version = i64::try_from(update.write_version)?;

        let wallet_address: String = if let Some(wallet_address) = entry.data {
            bs58::encode(wallet_address).into_string()
        } else {
            return Ok(());
        };
        let address = Owned(bs58::encode(update.key).into_string());
        let insert_wallet_address = Owned(wallet_address.clone());
        let values = TwitterHandle {
            address,
            wallet_address: insert_wallet_address,
            twitter_handle: Owned(entry.name),
            slot,
            from_cardinal: true,
            from_bonfida: false,
            write_version,
        };
        let query_wallet_address = wallet_address.clone();

        let rows = client
            .db()
            .run(move |db| {
                twitter_handle_name_services::table
                    .select(twitter_handle_name_services::all_columns)
                    .filter(
                        twitter_handle_name_services::wallet_address.eq(&wallet_address),
                    )
                    .load::<TwitterHandle>(db)
            })
            .await
            .context("failed to load twitter handle name services accounts!")?;

        let versions = (slot, write_version);

        match rows.get(0) {
            Some(indexed) if versions > (indexed.slot, indexed.write_version) => {
                client
                    .db()
                    .run(move |db| {
                        dbUpdate(
                                twitter_handle_name_services::table
                                .filter(twitter_handle_name_services::wallet_address.eq(&query_wallet_address)),
                        )
                        .set(&values)
                        .execute(db)
                    })
                    .await
                    .context("failed to update twitter handle")?;
            },
            Some(_) => (),
            None => {
                client
                    .db()
                    .run(move |db| {
                        insert_into(twitter_handle_name_services::table)
                            .values(&values)
                            .on_conflict(twitter_handle_name_services::wallet_address)
                            .do_update()
                            .set(&values)
                            .execute(db)
                    })
                    .await
                    .context("failed to insert twitter handle")?;
            },
        }
    }

    Ok(())
}
