//! Support features for the HTTP indexer

// mod metadata_json;
// mod store_config_json;

use indexer_rabbitmq::http_indexer::Entity;

use crate::prelude::*;

pub async fn process_message<E: Entity>(msg: E) -> Result<()> {
    let _ = msg;

    Ok(())
}
