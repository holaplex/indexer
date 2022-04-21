use std::sync::Arc;

use indexer_core::clap;
use meilisearch_sdk::{client::Client as MeiliClient, tasks::Task};

use crate::{db::Pool, prelude::*};

/// Common arguments for internal search indexer usage
#[derive(Debug, clap::Parser)]
pub struct Args {
    /// Meilisearch database endpoint
    #[clap(long, env)]
    meili_url: String,

    /// Meilisearch database API key
    #[clap(long, env)]
    meili_key: String,
}

/// Wrapper for handling network logic
#[derive(Debug)]
pub struct Client {
    db: Pool,
    meili: MeiliClient,
}

impl Client {
    /// Construct a new client, wrapped in an `Arc`.
    ///
    /// # Errors
    /// This function fails if the Meilisearch database cannot be initialized.
    pub async fn new_rc(db: Pool, args: Args) -> Result<Arc<Self>> {
        let Args {
            meili_url,
            meili_key,
        } = args;

        let meili = MeiliClient::new(meili_url, meili_key);

        create_index(meili.clone(), "metadatas", "id")
            .await
            .context("failed to create metadatas index")?;

        create_index(meili.clone(), "name_service", "id")
            .await
            .context("failed to create name service index")?;

        Ok(Arc::new(Self { db, meili }))
    }

    /// Get a reference to the database
    #[must_use]
    pub fn db(&self) -> &Pool {
        &self.db
    }

    /// Upsert a document to the `foo` index
    ///
    /// # Errors
    /// This function fails if the HTTP call returns an error
    pub async fn upsert_documents<
        T: serde::Serialize + std::fmt::Debug + serde::de::DeserializeOwned,
    >(
        &self,
        idx: String,
        docs: &[super::Document<T>],
    ) -> Result<Task> {
        self.meili
            .index(idx)
            .add_or_replace(docs, None)
            .await
            .context("Meilisearch API call failed")
    }
}

async fn create_index(meili: MeiliClient, index_name: &str, primary_key: &str) -> Result<()> {
    if let Ok(idx) = meili.get_index(index_name).await {
        ensure!(
            idx.get_primary_key()
                .await
                .context("Failed to check primary key name")?
                .map_or(false, |k| k == primary_key),
            "Primary key mismatch for index {}",
            index_name
        );
    } else {
        let task = meili.create_index(index_name, Some(primary_key)).await?;
        meili.wait_for_task(task, None, None).await?;
    };

    Ok(())
}
