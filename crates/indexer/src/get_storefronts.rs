use std::env;

use baby_pool::ThreadPoolHandle;
use indexer_core::prelude::*;
use serde::Deserialize;

use crate::Job;

#[derive(Debug, Deserialize)]
struct StorefrontRecord {
    storefront: Storefront,
}

#[derive(Debug, Deserialize)]
struct Storefront {
    pubkey: String,
    subdomain: String,
}

async fn get_storefronts_async(_handle: ThreadPoolHandle<'_, Job>) -> Result<()> {
    let storefronts: Vec<Storefront> = reqwest::Client::new()
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

    debug!("Storefronts: {:?}", storefronts);

    Ok(())
}

pub fn run(handle: ThreadPoolHandle<Job>) -> Result<()> {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .context("Failed to create async executor")?
        .block_on(get_storefronts_async(handle))
}
