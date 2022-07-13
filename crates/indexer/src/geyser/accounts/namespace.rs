use indexer_core::{
    db::{
        insert_into,
        models::{CardinalEntry, CardinalNamespace, TwitterHandle},
        tables::{cardinal_entries, cardinal_namespaces, twitter_handle_name_services},
        update,
    },
    pubkeys::CARDINAL_TWITTER_NAMESPACE,
};
use namespaces::state::{Entry, Namespace};

use super::Client;
use crate::{prelude::*, search_dispatch::TwitterHandleDocument};

pub(crate) async fn process(
    client: &Client,
    key: Pubkey,
    slot: u64,
    write_version: u64,
    entry: Entry,
) -> Result<()> {
    if entry.namespace == CARDINAL_TWITTER_NAMESPACE {
        process_twitter_namespace(client, key, slot, write_version, entry.clone())
            .await
            .context("failed to process twitter namespace")?;
    }

    let row = CardinalEntry {
        address: Owned(key.to_string()),
        namespace: Owned(entry.namespace.to_string()),
        name: Owned(entry.clone().name),
        data: entry.data.map(|a| Owned(a.to_string())),
        reverse_entry: entry.reverse_entry.map(|a| Owned(a.to_string())),
        mint: Owned(entry.mint.to_string()),
        is_claimed: entry.is_claimed,
        slot: slot.try_into()?,
        write_version: write_version.try_into()?,
    };

    client
        .db()
        .run(move |db| {
            insert_into(cardinal_entries::table)
                .values(&row)
                .on_conflict(cardinal_entries::address)
                .do_update()
                .set(&row)
                .execute(db)
        })
        .await
        .context("failed to insert cardinal entry")?;

    Ok(())
}

pub(crate) async fn process_namespace(
    client: &Client,
    key: Pubkey,
    slot: u64,
    write_version: u64,
    namespace: Namespace,
) -> Result<()> {
    let row = CardinalNamespace {
        address: Owned(key.to_string()),
        name: Owned(namespace.name.to_string()),
        update_authority: Owned(namespace.clone().update_authority.to_string()),
        rent_authority: Owned(namespace.rent_authority.to_string()),
        approve_authority: namespace.approve_authority.map(|a| Owned(a.to_string())),
        schema: namespace.schema.try_into()?,
        payment_amount_daily: namespace.payment_amount_daily.try_into()?,
        payment_mint: Owned(namespace.payment_mint.to_string()),
        min_rental_seconds: namespace.min_rental_seconds,
        max_rental_seconds: namespace.max_rental_seconds,
        transferable_entries: namespace.transferable_entries,
        slot: slot.try_into()?,
        write_version: write_version.try_into()?,
    };

    client
        .db()
        .run(move |db| {
            insert_into(cardinal_namespaces::table)
                .values(&row)
                .on_conflict(cardinal_namespaces::address)
                .do_update()
                .set(&row)
                .execute(db)
        })
        .await
        .context("failed to insert cardinal namespace")?;

    Ok(())
}

async fn process_twitter_namespace(
    client: &Client,
    key: Pubkey,
    slot: u64,
    write_version: u64,
    entry: Entry,
) -> Result<()> {
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
