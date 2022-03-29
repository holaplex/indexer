use graph_program::state::Connection;
use indexer_core::{
    db::{insert_into, models::GraphConnection as DbGraphConnection, tables::graph_connections},
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
            insert_into(graph_connections::table)
                .values(&row)
                .on_conflict(graph_connections::address)
                .do_update()
                .set(&row)
                .execute(db)
        })
        .await
        .context("Failed to insert graph connection")?;

    Ok(())
}
