use indexer_rabbitmq::geyser::{SlotStatus, SlotStatusUpdate};

use super::Client;
use crate::prelude::*;

pub async fn process(client: &Client, slot: SlotStatusUpdate) -> Result<()> {
    let SlotStatusUpdate {
        slot,
        parent: _,
        status,
    } = slot;

    match status {
        SlotStatus::Confirmed => client
            .dispatch_block_reindex(slot)
            .await
            .context("Failed to dispatch block reindex request"),
        _ => Ok(()),
    }
}
