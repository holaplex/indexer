use indexer_core::{
    db::{insert_into, models::TwitterHandle, tables::twitter_handle_name_services, update},
    pubkeys::CARDINAL_TWITTER_NAMESPACE,
};
use namespaces::state::Entry;

use super::Client;
use crate::{prelude::*, search_dispatch::TwitterHandleDocument};

pub(crate) async fn process(
    client: &Client,
    key: Pubkey,
    slot: u64,
    write_version: u64,
    entry: Entry,
) -> Result<()> {
    if entry.namespace != CARDINAL_TWITTER_NAMESPACE {
        return Ok(());
    }

    let slot = i64::try_from(slot)?;
    let write_version = i64::try_from(write_version)?;

    let wallet_address: String = if let Some(wallet_address) = entry.data {
        bs58::encode(wallet_address).into_string()
    } else {
        return Ok(());
    };

    let document = TwitterHandleDocument {
        owner: wallet_address.to_string(),
        handle: entry.name.clone(),
    };

    let values = TwitterHandle {
        address: Owned(bs58::encode(key).into_string()),
        wallet_address: Owned(wallet_address.clone()),
        twitter_handle: Owned(entry.name),
        slot,
        from_cardinal: true,
        from_bonfida: false,
        write_version,
    };

    let rows = client
        .db()
        .run({
            let wallet_address = wallet_address.clone();
            move |db| {
                twitter_handle_name_services::table
                    .select(twitter_handle_name_services::all_columns)
                    .filter(twitter_handle_name_services::wallet_address.eq(&wallet_address))
                    .load::<TwitterHandle>(db)
            }
        })
        .await
        .context("failed to load twitter handle name services accounts!")?;

    let search_backfill;

    match rows.get(0) {
        Some(indexed) if (slot, write_version) > (indexed.slot, indexed.write_version) => {
            search_backfill = Some(false);

            client
                .db()
                .run(move |db| {
                    update(
                        twitter_handle_name_services::table.filter(
                            twitter_handle_name_services::wallet_address.eq(&wallet_address),
                        ),
                    )
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
                        .on_conflict(twitter_handle_name_services::wallet_address)
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
