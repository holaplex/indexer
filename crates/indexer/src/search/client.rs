use std::{sync::Arc, time::Duration};

use crossbeam::queue::SegQueue;
use indexer_core::{clap, hash::HashMap};
use meilisearch_sdk::client::Client as MeiliClient;
use tokio::{
    sync::{mpsc, RwLock},
    task,
};

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

    /// Maximum number of documents to cache for upsert for a single index
    #[clap(long, env, default_value_t = 1000)]
    upsert_batch: usize,

    /// Maximum commit interval between upserts, in seconds
    #[clap(long, env, default_value_t = 30.0)]
    upsert_interval: f64,
}

/// Wrapper for handling network logic
#[derive(Debug)]
pub struct Client {
    db: Pool,
    upsert_batch: usize,
    upsert_queue: RwLock<SegQueue<(String, super::Document)>>,
    trigger_upsert: mpsc::Sender<()>,
}

impl Client {
    /// Construct a new client, wrapped in an `Arc`.
    ///
    /// # Errors
    /// This function fails if the Meilisearch database cannot be initialized.
    pub async fn new_rc(db: Pool, args: Args) -> Result<(Arc<Self>, task::JoinHandle<()>)> {
        let Args {
            meili_url,
            meili_key,
            upsert_batch,
            upsert_interval,
        } = args;

        let meili = MeiliClient::new(meili_url, meili_key);
        let upsert_interval = Duration::from_secs_f64(upsert_interval);

        create_index(meili.clone(), "metadatas", "id")
            .await
            .context("failed to create metadatas index")?;

        create_index(meili.clone(), "name_service", "id")
            .await
            .context("failed to create name service index")?;

        let (trigger_upsert, upsert_rx) = mpsc::channel(1);

        let arc_self = Arc::new(Self {
            db,
            upsert_batch,
            upsert_queue: RwLock::new(SegQueue::new()),
            trigger_upsert,
        });

        let upsert_task = task::spawn(arc_self.clone().run_upserts(
            meili.clone(),
            upsert_interval,
            upsert_rx,
        ));

        Ok((arc_self, upsert_task))
    }

    async fn run_upserts(
        self: Arc<Self>,
        meili: MeiliClient,
        interval: Duration,
        mut rx: mpsc::Receiver<()>,
    ) {
        loop {
            match self.try_run_upserts(meili.clone(), interval, &mut rx).await {
                Ok(()) => warn!("Meilisearch upsert task stopped early"),
                Err(e) => {
                    error!("Meilisearch upsert task crashed: {:?}", e);
                },
            }

            tokio::time::sleep(Duration::from_secs(5)).await;
        }
    }

    async fn try_run_upserts(
        &self,
        meili: MeiliClient,
        interval: Duration,
        rx: &mut mpsc::Receiver<()>,
    ) -> Result<()> {
        let mut timer = tokio::time::interval(interval);
        timer.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);

        loop {
            let evt = tokio::select! {
                e = rx.recv() => e,
                _ = timer.tick() => Some(()),
            };

            if let Some(()) = evt {
                // TODO: parse any event info if we need to pass it in
            } else {
                break Ok(());
            };

            let queue = {
                let mut lock = self.upsert_queue.write().await;

                if lock.len() == 0 {
                    continue;
                }

                std::mem::take(&mut *lock)
            };

            debug!("Ticking document upsert for {} document(s)...", queue.len());

            let map =
                std::iter::from_fn(|| queue.pop()).fold(HashMap::default(), |mut h, (k, v)| {
                    h.entry(k).or_insert_with(Vec::new).push(v);
                    h
                });

            for (idx, docs) in map {
                meili
                    .index(idx)
                    .add_or_replace(&*docs, None)
                    .await
                    .context("Meilisearch API call failed")?;
            }
        }
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
    pub async fn upsert_documents<D: IntoIterator<Item = super::Document>>(
        &self,
        idx: String,
        docs: D,
    ) -> Result<()> {
        let q = self.upsert_queue.read().await;
        std::iter::repeat(idx).zip(docs).for_each(|p| q.push(p));

        if q.len() >= self.upsert_batch {
            self.trigger_upsert
                .send(())
                .await
                .context("Failed to trigger upsert")?;
        }

        Ok(())
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
