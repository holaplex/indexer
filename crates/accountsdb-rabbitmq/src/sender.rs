use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};

use indexer_rabbitmq::{
    accountsdb::{Message, Producer, QueueType, StartupType},
    lapin::{Connection, ConnectionProperties},
};
use smol::{
    channel,
    lock::{RwLock, RwLockUpgradableReadGuard, RwLockWriteGuard},
    Executor,
};

use crate::{
    config,
    metrics::{Counter, Metrics},
};

#[derive(Debug)]
struct Inner {
    amqp: config::Amqp,
    startup_type: StartupType,
    producer: RwLock<Producer>,
    metrics: Arc<Metrics>,
}

impl Inner {
    async fn new(
        amqp: config::Amqp,
        startup_type: StartupType,
        metrics: Arc<Metrics>,
    ) -> Result<Self, indexer_rabbitmq::Error> {
        let producer = Self::create_producer(&amqp, startup_type).await?;

        Ok(Self {
            amqp,
            startup_type,
            producer: RwLock::new(producer),
            metrics,
        })
    }

    async fn create_producer(
        amqp: &config::Amqp,
        startup_type: StartupType,
    ) -> Result<Producer, indexer_rabbitmq::Error> {
        let conn = Connection::connect(
            &amqp.address,
            ConnectionProperties::default().with_executor(smol_executor_trait::Smol),
        )
        .await?;

        Producer::new(&conn, QueueType::new(amqp.network, startup_type, None)).await
    }

    async fn connect<'a>(
        &self,
        prod: RwLockUpgradableReadGuard<'a, Producer>,
    ) -> Result<RwLockUpgradableReadGuard<'a, Producer>, indexer_rabbitmq::Error> {
        let mut prod = RwLockUpgradableReadGuard::upgrade(prod).await;

        *prod = Self::create_producer(&self.amqp, self.startup_type).await?;

        Ok(RwLockWriteGuard::downgrade_to_upgradable(prod))
    }

    async fn send(self: Arc<Self>, msg: Message) {
        #[inline]
        fn log_err<E: std::fmt::Debug>(counter: &'_ Counter) -> impl FnOnce(E) + '_ {
            |err| {
                counter.log();
                log::error!("{:?}", err);
            }
        }

        let metrics = self.metrics.as_ref();
        let prod = self.producer.upgradable_read().await;

        match prod.write(&msg).await.map_err(log_err(&metrics.errs)) {
            Ok(()) => return,
            Err(()) => (),
        }

        metrics.reconnects.log();
        let prod = match self.connect(prod).await.map_err(log_err(&metrics.errs)) {
            Ok(p) => p,
            Err(()) => return,
        };

        match prod.write(&msg).await.map_err(log_err(&metrics.errs)) {
            Ok(()) | Err(()) => (), // Type-level assertion that we consumed the error
        }
    }
}

#[derive(Debug)]
pub struct Sender {
    inner: Arc<Inner>,
    executor: Arc<Executor<'static>>,
    _stop: channel::Sender<()>,
    background_count: AtomicUsize,
    limit: usize,
}

impl Sender {
    pub async fn new(
        amqp: config::Amqp,
        jobs: &config::Jobs,
        startup_type: StartupType,
        metrics: Arc<Metrics>,
    ) -> Result<Self, indexer_rabbitmq::Error> {
        let executor = Arc::new(Executor::new());
        let (stop_tx, stop_rx) = channel::bounded(1);

        std::thread::spawn({
            let executor = executor.clone();

            move || smol::block_on(executor.run(stop_rx.recv()))
        });

        Ok(Self {
            inner: Arc::new(Inner::new(amqp, startup_type, metrics).await?),
            executor,
            _stop: stop_tx,
            background_count: AtomicUsize::new(0),
            limit: jobs.limit,
        })
    }

    pub async fn send(&self, msg: Message) {
        let new_count =
            self.background_count
                .fetch_update(Ordering::SeqCst, Ordering::SeqCst, |c| {
                    if c < self.limit { Some(c + 1) } else { None }
                });

        let inner = Arc::clone(&self.inner);

        if new_count.is_ok() {
            self.executor.spawn(inner.send(msg)).detach();

            assert!(self.background_count.fetch_sub(1, Ordering::SeqCst) > 0);
        } else {
            inner.send(msg).await;
        }
    }
}
