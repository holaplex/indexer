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

/// Document added to an index by an `IndirectMetadata` message
#[derive(Debug, serde::Serialize, serde::Deserialize, Clone)]
pub struct IndirectMetadataDocument {
    /// The address of the Metaplex metadata account
    pub metadata_address: String,
    /// The address of the NFT mint
    pub mint_address: String,
    /// The name of the metadata account
    pub name: String,
    /// The image associated with the metadata account
    pub image: Option<String>,
    /// The first listed creator on the metadata account
    pub creator_address: String,
    /// The Twitter handle associated with `creator_address`
    pub creator_twitter_handle: Option<String>,
    /// The certified collection address of the metadata account
    pub collection_address: Option<String>,
}
