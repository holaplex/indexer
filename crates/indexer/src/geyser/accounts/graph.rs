use graph_program::state::Connection;
use indexer_core::{
    db::{
        exists, insert_into,
        models::{FeedEventWallet, FollowEvent, GraphConnection as DbGraphConnection},
        select,
        tables::{feed_event_wallets, feed_events, follow_events, graph_connections},
        uuid,
    },
    prelude::*,
};

use super::Client;
use crate::prelude::*;

pub(crate) async fn process(client: &Client, key: Pubkey, account_data: Connection) -> Result<()> {
    let row = DbGraphConnection {
        address: Owned(bs58::encode(key).into_string()),
        from_account: Owned(bs58::encode(account_data.from).into_string()),
        to_account: Owned(bs58::encode(account_data.to).into_string()),
    };

    client
        .db()
        .run(move |db| {
            let graph_connection_exists = select(exists(
                graph_connections::table.filter(graph_connections::address.eq(row.address.clone())),
            ))
            .get_result::<bool>(db);

            if Ok(true) == graph_connection_exists {
                return Result::<_>::Ok(());
            }

            db.build_transaction().read_write().run(|| {
                insert_into(graph_connections::table)
                    .values(&row)
                    .on_conflict(graph_connections::address)
                    .do_update()
                    .set(&row)
                    .execute(db)?;

                let feed_event_id = insert_into(feed_events::table)
                    .default_values()
                    .returning(feed_events::id)
                    .get_result::<uuid::Uuid>(db)
                    .context("Failed to insert feed event")?;

                insert_into(follow_events::table)
                    .values(&FollowEvent {
                        feed_event_id: Owned(feed_event_id),
                        graph_connection_address: row.address,
                    })
                    .execute(db)
                    .context("failed to insert follow event")?;

                insert_into(feed_event_wallets::table)
                    .values(&FeedEventWallet {
                        wallet_address: row.to_account,
                        feed_event_id: Owned(feed_event_id),
                    })
                    .execute(db)
                    .context("Failed to insert follow feed event wallet for followed")?;

                insert_into(feed_event_wallets::table)
                    .values(&FeedEventWallet {
                        wallet_address: row.from_account,
                        feed_event_id: Owned(feed_event_id),
                    })
                    .execute(db)
                    .context("Failed to insert follow feed event wallet for the follower")?;

                Result::<_>::Ok(())
            })
        })
        .await
        .context("Failed to insert graph connection")?;

    Ok(())
}
