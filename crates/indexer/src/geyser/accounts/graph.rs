use graph_program::state::ConnectionV2;
use indexer_core::{
    db::{
        insert_into,
        models::{FeedEventWallet, FollowEvent, GraphConnection as DbGraphConnection},
        select,
        tables::{feed_event_wallets, feed_events, follow_events, graph_connections},
    },
    prelude::*,
    util,
    uuid::Uuid,
};

use super::Client;
use crate::prelude::*;

pub(crate) async fn process(
    client: &Client,
    key: Pubkey,
    account_data: ConnectionV2,
) -> Result<()> {
    let row = DbGraphConnection {
        address: Owned(bs58::encode(key).into_string()),
        from_account: Owned(bs58::encode(account_data.from).into_string()),
        to_account: Owned(bs58::encode(account_data.to).into_string()),
        connected_at: util::unix_timestamp(account_data.connected_at)?,
        disconnected_at: account_data
            .disconnected_at
            .map(util::unix_timestamp)
            .transpose()?,
    };

    client
        .db()
        .run(move |db| {
            let graph_connection_exists = select(exists(
                graph_connections::table.filter(graph_connections::address.eq(row.address.clone())),
            ))
            .get_result::<bool>(db);

            insert_into(graph_connections::table)
                .values(&row)
                .on_conflict(graph_connections::address)
                .do_update()
                .set(&row)
                .execute(db)?;

            if Ok(true) == graph_connection_exists {
                return Ok(());
            }

            db.build_transaction().read_write().run(|| {
                let feed_event_id = insert_into(feed_events::table)
                    .default_values()
                    .returning(feed_events::id)
                    .get_result::<Uuid>(db)
                    .context("Failed to insert feed event")?;

                insert_into(follow_events::table)
                    .values(&FollowEvent {
                        feed_event_id,
                        graph_connection_address: row.address,
                    })
                    .execute(db)
                    .context("Failed to insert follow event")?;

                insert_into(feed_event_wallets::table)
                    .values(&FeedEventWallet {
                        wallet_address: row.from_account,
                        feed_event_id,
                    })
                    .execute(db)
                    .context("Failed to insert follow feed event wallet")?;

                Result::<_>::Ok(())
            })
        })
        .await
        .context("Failed to insert graph connection")?;

    Ok(())
}
