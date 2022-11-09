use super::Client;
use crate::prelude::*;

pub async fn process(client: &Client, id: u64) -> Result<()> {
    debug!("Reindexing slot {:?}", id);

    let block = client.run_rpc(move |r| r.get_block(id)).await;

    debug!("Block data: {block:?}");

    Ok(())
}
