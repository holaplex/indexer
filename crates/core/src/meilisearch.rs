//! Re-exports and common logic for the Meilisearch SDK

pub use meilisearch_sdk::*;

/// Arguments for constructing a Meilisearch client
#[derive(Debug, Clone, clap::Parser)]
pub struct Args {
    /// Meilisearch database endpoint
    #[clap(long, env)]
    meili_url: String,

    /// Meilisearch database API key
    #[clap(long, env)]
    meili_key: String,
}

impl Args {
    /// Construct a Meilisearch client from the provided arguments
    #[must_use]
    pub fn into_client(self) -> client::Client {
        let Self {
            meili_url,
            meili_key,
        } = self;

        client::Client::new(meili_url, meili_key)
    }
}
