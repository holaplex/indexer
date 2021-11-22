use std::env;

use indexer_core::prelude::*;
use serde::Deserialize;
use solana_sdk::pubkey::Pubkey;

use crate::{Job, ThreadPoolHandle};

#[derive(Debug, Deserialize)]
struct StorefrontRecord {
    storefront: Storefront,
}

#[derive(Debug, Deserialize)]
struct Storefront {
    pubkey: String,
    /* TODO
     * subdomain: String, */
}

async fn get_storefronts_async(handle: &ThreadPoolHandle<'_>) -> Result<()> {
    let storefronts: Vec<StorefrontRecord> = reqwest::Client::new()
        .get(
            env::var("HOLAPLEX_STOREFRONTS_ENDPOINT")
                .context("Couldn't get endpoint for Holaplex storefronts")?,
        )
        .send()
        .await
        .context("Couldn't GET Holaplex storefronts")?
        .json()
        .await
        .context("Couldn't parse Holaplex storefronts")?;

    debug!("Loaded {:?} storefront(s)", storefronts.len());

    for StorefrontRecord { storefront } in storefronts {
        match Pubkey::try_from(storefront.pubkey.as_str()) {
            Ok(k) => handle.push(Job::StoreOwner(k)),
            Err(e) => error!("Failed to parse {:?}: {:?}", storefront.pubkey, e),
        }
    }

    Ok(())
}

pub fn run(handle: &ThreadPoolHandle) -> Result<()> {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .context("Failed to create async executor")?
        .block_on(get_storefronts_async(handle))
}
