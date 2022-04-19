use std::sync::Arc;

use indexer_core::clap;
use meilisearch_sdk::{client::Client as MeiliClient, indexes::Index, tasks::Task};

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
    foo: Index,
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

        #[allow(clippy::blacklisted_name)] // :p
        let foo = {
            const NAME: &str = "foo";
            const PKEY: &str = "id";

            if let Ok(idx) = meili.get_index(NAME).await {
                ensure!(
                    idx.get_primary_key()
                        .await
                        .context("Failed to check primary key name")?
                        .map_or(false, |k| k == PKEY),
                    "Primary key mismatch for index {}",
                    NAME
                );

                idx
            } else {
                let task = meili.create_index(NAME, Some(PKEY)).await?;
                meili.wait_for_task(task, None, None).await?;

                meili.index(NAME)
            }
        };

        Ok(Arc::new(Self { db, foo }))
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
    pub async fn upsert_foo(&self, docs: &[super::Document]) -> Result<Task> {
        self.foo
            .add_or_replace(docs, None)
            .await
            .context("Meilisearch API call failed")
    }
}
