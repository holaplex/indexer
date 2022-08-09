use std::{collections::BinaryHeap, sync::Arc, time::Duration};

use crossbeam::queue::SegQueue;
use indexer_core::{
    assets::AssetProxyArgs,
    clap,
    hash::HashMap,
    meilisearch::{
        self,
        client::Client as MeiliClient,
        tasks::{DocumentAddition, ProcessedTask, Task, TaskType},
    },
    util,
};
use tokio::{
    sync::{mpsc, oneshot, RwLock},
    task,
};

use crate::{db::Pool, prelude::*};

/// Common arguments for internal search indexer usage
#[derive(Debug, clap::Parser)]
pub struct Args {
    /// Maximum number of documents to cache for upsert for a single index
    #[clap(long, env, default_value_t = 1000)]
    upsert_batch: usize,

    /// Sample size to use when approximating upsert interval from completed
    /// tasks
    #[clap(long, env, default_value_t = 30)]
    upsert_interval_sample_size: usize,

    /// Don't perform any upserts, just print what would be upserted
    #[clap(long, short = 'n', env)]
    dry_run: bool,

    #[clap(flatten)]
    meili: meilisearch::Args,

    #[clap(flatten)]
    asset_proxy: AssetProxyArgs,
}

/// Wrapper for handling network logic
#[derive(Debug)]
pub struct Client {
    db: Pool,
    upsert_batch: usize,
    asset_proxy: AssetProxyArgs,
    upsert_queue: RwLock<SegQueue<(String, super::Document)>>,
    trigger_upsert: mpsc::Sender<()>,
}

impl Client {
    /// Construct a new client, wrapped in an `Arc`.
    ///
    /// # Errors
    /// This function fails if the Meilisearch database cannot be initialized.
    pub async fn new_rc(
        db: Pool,
        args: Args,
    ) -> Result<(Arc<Self>, task::JoinHandle<()>, oneshot::Sender<()>)> {
        let Args {
            upsert_batch,
            upsert_interval_sample_size,
            dry_run,
            meili,
            asset_proxy,
        } = args;

        let meili = meili.into_client();

        create_index(meili.clone(), "metadatas", "id")
            .await
            .context("Failed to create metadatas index")?;

        create_index(meili.clone(), "geno_habitats", "id")
            .await
            .context("Failed to create Genopets habitat index")?;

        create_index(meili.clone(), "name_service", "id")
            .await
            .context("Failed to create name service index")?;

        create_index(meili.clone(), "collections", "id")
            .await
            .context("Failed to create collections index")?;

        let (trigger_upsert, upsert_rx) = mpsc::channel(1);
        let (stop_tx, stop_rx) = oneshot::channel();

        let arc_self = Arc::new(Self {
            db,
            upsert_batch,
            asset_proxy,
            upsert_queue: RwLock::new(SegQueue::new()),
            trigger_upsert,
        });

        let upsert_task = task::spawn(arc_self.clone().run_upserts(
            meili.clone(),
            upsert_interval_sample_size,
            upsert_batch,
            dry_run,
            upsert_rx,
            stop_rx,
        ));

        Ok((arc_self, upsert_task, stop_tx))
    }

    async fn run_upserts(
        self: Arc<Self>,
        meili: MeiliClient,
        interval_sample_size: usize,
        batch_size: usize,
        dry_run: bool,
        mut rx: mpsc::Receiver<()>,
        mut stop_rx: oneshot::Receiver<()>,
    ) {
        loop {
            match self
                .try_run_upserts(
                    meili.clone(),
                    interval_sample_size,
                    batch_size,
                    dry_run,
                    &mut rx,
                    &mut stop_rx,
                )
                .await
            {
                Ok(()) => break,
                Err(e) => {
                    error!("Meilisearch upsert task crashed: {:?}", e);
                },
            }

            tokio::time::sleep(Duration::from_secs(5)).await;
        }
    }

    async fn update_upsert_interval(
        meili: &MeiliClient,
        sample_size: usize,
        batch_size: usize,
    ) -> Result<Duration> {
        let start = Local::now();

        let tasks = meili
            .get_tasks()
            .await
            .context("Failed to get Meilisearch task list")?;

        let mut set: BinaryHeap<_> = tasks
            .into_iter()
            .filter_map(|task| {
                let ProcessedTask {
                    duration,
                    finished_at,
                    update_type,
                    ..
                } = match task {
                    Task::Succeeded { content } => content,
                    _ => return None,
                };

                // Reject outliers or non-upsert tasks
                let count = match update_type {
                    TaskType::DocumentAddition {
                        details:
                            Some(DocumentAddition {
                                indexed_documents: Some(count),
                                ..
                            }),
                    } => count,
                    _ => return None,
                };

                let finished_at = finished_at.unix_timestamp_nanos();
                let finished_at = DateTime::<Utc>::from_utc(
                    NaiveDateTime::from_timestamp_opt(
                        (finished_at / 1_000_000_000).try_into().ok()?,
                        finished_at.rem_euclid(1_000_000_000).try_into().ok()?,
                    )?,
                    Utc,
                );

                Some((finished_at, count, duration))
            })
            .collect();

        let mut times: Vec<_> = std::iter::from_fn(|| set.pop())
            .take(sample_size)
            .map(|(_, c, d)| {
                #[allow(clippy::cast_precision_loss)]
                d.mul_f64(batch_size as f64 / c as f64)
            })
            .collect();
        times.sort_unstable();

        let interval;

        {
            #![allow(
                clippy::cast_possible_truncation,
                clippy::cast_precision_loss,
                clippy::cast_sign_loss
            )]

            interval = times
                .get((times.len() as f64 * 0.75).round() as usize)
                .or_else(|| times.get(0))
                .copied()
                .unwrap_or_else(|| Duration::from_secs(30));
        }

        info!(
            "Selected upsert interval: {} duration={}",
            chrono::Duration::from_std(interval)
                .map_or_else(|_| "???".into(), util::duration_hhmmssfff),
            interval.as_secs_f64()
        );

        let elapsed = Local::now() - start;

        if elapsed > chrono::Duration::seconds(30) {
            warn!(
                "Calculating interval took {}",
                util::duration_hhmmssfff(elapsed)
            );
        }

        Ok(interval)
    }

    async fn try_run_upserts(
        &self,
        meili: MeiliClient,
        interval_sample_size: usize,
        batch_size: usize,
        dry_run: bool,
        rx: &mut mpsc::Receiver<()>,
        mut stop_rx: &mut oneshot::Receiver<()>,
    ) -> Result<()> {
        enum Event {
            Rx(Option<()>),
            Stop(Result<(), oneshot::error::RecvError>),
            Tick,
        }

        let mut lock_if_stopping = None;

        let stop_reason = loop {
            use futures_util::StreamExt;

            let interval =
                Self::update_upsert_interval(&meili, interval_sample_size, batch_size).await?;

            let evt = tokio::select! {
                o = rx.recv() => Event::Rx(o),
                r = &mut stop_rx => Event::Stop(r),
                () = tokio::time::sleep(interval) => Event::Tick,
            };

            let stop_reason = match evt {
                Event::Rx(Some(())) | Event::Tick => None,
                Event::Rx(None) => Some("trigger event source closed"),
                Event::Stop(Ok(())) => Some("stop signal received"),
                Event::Stop(Err(e)) => {
                    // Stoplight broke, stop anyway
                    error!("Failed to read upsert stop signal: {}", e);
                    Some("error occurred reading stop signal")
                },
            };

            debug_assert!(lock_if_stopping.is_none());
            let mut lock = self.upsert_queue.write().await;

            if stop_reason.is_none() && lock.len() == 0 {
                continue;
            }

            let queue = std::mem::take(&mut *lock);

            if stop_reason.is_none() {
                std::mem::drop(lock);
            } else {
                lock_if_stopping = Some(lock);
            }

            debug!("Ticking document upsert for {} document(s)...", queue.len());

            let map =
                std::iter::from_fn(|| queue.pop()).fold(HashMap::default(), |mut h, (k, v)| {
                    h.entry(k).or_insert_with(Vec::new).push(v);
                    h
                });

            let mut futures = futures_util::stream::FuturesUnordered::new();

            for (idx, docs) in &map {
                debug!(
                    "{} document(s) in upsert queue flagged for {:?}",
                    docs.len(),
                    idx
                );

                if dry_run {
                    info!("Upsert to {:?} of {:#?}", idx, serde_json::to_value(&docs));
                } else {
                    let meili = meili.clone();
                    futures
                        .push(async move { meili.index(idx).add_or_replace(&*docs, None).await });
                }
            }

            while let Some(res) = futures.next().await {
                res.context("Meilisearch API call failed")?;
            }

            if let Some(reason) = stop_reason {
                break reason;
            }
        };

        info!("Stopping upsert worker: {}", stop_reason);

        debug_assert!(lock_if_stopping.is_some());

        rx.close();
        stop_rx.close();

        Ok(())
    }

    /// Get a reference to the database
    #[must_use]
    pub fn db(&self) -> &Pool {
        &self.db
    }

    /// Get a reference to the asset proxy arguments, used by
    /// [`proxy_url`](indexer_core::assets::proxy_url)
    #[inline]
    pub fn proxy_args(&self) -> &AssetProxyArgs {
        &self.asset_proxy
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
            use mpsc::error::TrySendError;

            match self.trigger_upsert.try_send(()) {
                // TrySendError::Full means an upsert has already been triggered
                Ok(()) | Err(TrySendError::Full(())) => (),
                Err(e) => return Err(e).context("Failed to trigger upsert")?,
            }
        }

        Ok(())
    }
}

async fn create_index(meili: MeiliClient, index_name: &str, primary_key: &str) -> Result<()> {
    if let Ok(mut idx) = meili.get_index(index_name).await {
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
