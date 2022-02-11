//! Support features for the HTTP indexer

// mod metadata_json;
// mod store_config_json;

use indexer_rabbitmq::http_indexer::Message;

use crate::prelude::*;

pub async fn process_message(msg: Message) -> Result<()> {
    let _ = msg;

    Ok(())
}
