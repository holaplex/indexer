use anchor_lang_v0_24::AccountDeserialize;
use graph_program::state::ConnectionV2;
use indexer::prelude::*;

use super::{accounts::graph, AccountUpdate, Client};

pub(crate) async fn process(client: &Client, update: AccountUpdate) -> Result<()> {
    let connection: ConnectionV2 = ConnectionV2::try_deserialize(&mut update.data.as_slice())
        .context("Failed to deserialize graph program Connection data")?;
    graph::process(
        client,
        update.key,
        connection,
        update.slot,
        update.write_version,
    )
    .await
}
