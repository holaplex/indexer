use borsh::BorshDeserialize;
use indexer_core::{
    db::{
        insert_into,
        models::{SolDomain, TwitterHandle},
        tables::{sol_domains, twitter_handle_name_services},
        update,
    },
    prelude::*,
};

use super::Client;
use crate::prelude::*;

#[derive(BorshDeserialize, PartialEq, Debug, Clone)]
struct TwitterHandleAndRegistry {
    registry_key: [u8; 32],
    handle: String,
}

pub(crate) async fn process(
    client: &Client,
    key: Pubkey,
    slot: u64,
    wallet: Pubkey,
    data: Vec<u8>,
) -> Result<()> {
    let th = TwitterHandleAndRegistry::deserialize(&mut data.as_slice())
        .context("failed to deserialize registry key and handle!")?;

    let incoming_slot: i64 = slot.try_into()?;

    let rows = client
        .db()
        .run(move |db| {
            twitter_handle_name_services::table
                .select(twitter_handle_name_services::all_columns)
                .filter(twitter_handle_name_services::address.eq(key.to_string()))
                .load::<TwitterHandle>(db)
        })
        .await
        .context("failed to load twitter handle name service account!")?;

    let pubkey: String = key.to_string();

    let values = TwitterHandle {
        address: Owned(pubkey.clone()),
        wallet_address: Owned(wallet.to_string()),
        twitter_handle: Owned(th.handle),
        slot: slot.try_into()?,
    };

    match rows.get(0) {
        Some(indexed) if incoming_slot > indexed.slot => {
            client
                .db()
                .run(move |db| {
                    update(
                        twitter_handle_name_services::table
                            .filter(twitter_handle_name_services::address.eq(pubkey)),
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
                        .on_conflict(twitter_handle_name_services::address)
                        .do_update()
                        .set(&values)
                        .execute(db)
                })
                .await
                .context("failed to insert twitter handle")?;
        },
    }

    Ok(())
}

pub(crate) async fn process_domain_name(
    client: &Client,
    key: Pubkey,
    slot: u64,
    wallet: Pubkey,
    data: Vec<u8>,
) -> Result<()> {
    let incoming_slot: i64 = slot.try_into()?;

    let rows = client
        .db()
        .run(move |db| {
            sol_domains::table
                .select(sol_domains::all_columns)
                .filter(sol_domains::address.eq(key.to_string()))
                .load::<SolDomain>(db)
        })
        .await
        .context("failed to load sol domain")?;

    let pubkey: String = key.to_string();

    let values = SolDomain {
        address: Owned(pubkey.clone()),
        owner: Owned(wallet.to_string()),
        name: Owned(
            std::str::from_utf8(&data)
                .context("failed to deserialize sol domain")?
                .to_string(),
        ),
        slot: slot.try_into()?,
    };

    match rows.get(0) {
        Some(indexed) if incoming_slot > indexed.slot => {
            client
                .db()
                .run(move |db| {
                    update(sol_domains::table.filter(sol_domains::address.eq(pubkey)))
                        .set(&values)
                        .execute(db)
                })
                .await
                .context("failed to update sol domain")?;
        },
        Some(_) => (),
        None => {
            client
                .db()
                .run(move |db| {
                    insert_into(sol_domains::table)
                        .values(&values)
                        .on_conflict(sol_domains::address)
                        .do_update()
                        .set(&values)
                        .execute(db)
                })
                .await
                .context("failed to insert sol domain")?;
        },
    }

    Ok(())
}
