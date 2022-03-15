use anchor_lang_v0_22_1::AccountDeserialize;
use graph_program::state::Connection;

use super::{accounts::graph, AccountUpdate, Client};
use crate::prelude::*;

pub(crate) async fn process(client: &Client, update: AccountUpdate) -> Result<()> {
    let connection: Connection = Connection::try_deserialize(&mut update.data.as_slice())
        .context("Failed to deserialize graph program Connection data")?;
    graph::process(client, update.key, connection).await
}
