use borsh::BorshDeserialize;
use indexer_core::{
    db::{insert_into, models::TwitterHandle, tables::twitter_handle_name_services, update},
    prelude::*,
};

use super::Client;
use crate::{prelude::*, search_dispatch::TwitterHandleDocument};

#[derive(BorshDeserialize, PartialEq, Debug, Clone)]
struct TwitterHandleAndRegistry {
    registry_key: [u8; 32],
    handle: String,
}

pub(crate) async fn process(
    client: &Client,
    key: Pubkey,
    slot: u64,
    write_version: u64,
    wallet: Pubkey,
    data: Vec<u8>,
) -> Result<()> {
    let th = TwitterHandleAndRegistry::deserialize(&mut data.as_slice())
        .context("failed to deserialize registry key and handle!")?;

    let incoming_slot: i64 = slot.try_into()?;

    let document = TwitterHandleDocument {
        owner: wallet.to_string(),
        handle: th.clone().handle,
    };

    let rows = client
        .db()
        .run(move |db| {
            twitter_handle_name_services::table
                .select(twitter_handle_name_services::all_columns)
                .filter(twitter_handle_name_services::address.eq(key.to_string()))
                .load::<TwitterHandle>(db)
        })
        .await
        .context("failed to load twitter handle name services accounts!")?;

    let pubkey: String = key.to_string();
    let write_version = i64::try_from(write_version)?;

    let values = TwitterHandle {
        address: Owned(pubkey.clone()),
        wallet_address: Owned(wallet.to_string()),
        twitter_handle: Owned(th.handle),
        slot: slot.try_into()?,
        from_bonfida: true,
        from_cardinal: false,
        write_version,
    };

    let search_backfill;

    match rows.get(0) {
        Some(indexed) if (incoming_slot, write_version) > (indexed.slot, indexed.write_version) => {
            search_backfill = Some(false);

            client
                .db()
                .run(move |db| {
                    update(twitter_handle_name_services::table.filter(
                        twitter_handle_name_services::wallet_address.eq(wallet.to_string()),
                    ))
                    .set(&values)
                    .execute(db)
                })
                .await
                .context("failed to update twitter handle")?;
        },
        Some(_) => search_backfill = None,
        None => {
            search_backfill = Some(true);

            client
                .db()
                .run(move |db| {
                    insert_into(twitter_handle_name_services::table)
                        .values(&values)
                        .on_conflict(twitter_handle_name_services::address)
                        .do_update()
                        .set(&values)
                        .execute(db)
                })
                .await
                .context("failed to insert twitter handle")?;
        },
    }

    if let Some(backfill) = search_backfill {
        client
            .search()
            .upsert_twitter_handle(backfill, key, document)
            .await
            .context("Failed to dispatch upsert twitter handle document job")?;
    }

    Ok(())
}
