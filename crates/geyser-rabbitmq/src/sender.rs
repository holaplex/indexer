use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};

use indexer_rabbitmq::{
    geyser::{Message, Producer, QueueType, StartupType},
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
    background_count: AtomicUsize,

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
            background_count: AtomicUsize::new(0),
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
            ConnectionProperties::default()
                .with_connection_name(amqp.name.as_str().into())
                .with_executor(smol_executor_trait::Smol),
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

    async fn send_internal(&self, msg: Message) {
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

    async fn send(self: Arc<Self>, msg: Message, backgrounded: bool) {
        self.send_internal(msg).await;

        if backgrounded {
            assert!(self.background_count.fetch_sub(1, Ordering::SeqCst) > 0);
        }
    }
}

#[derive(Debug)]
pub struct Sender {
    inner: Arc<Inner>,
    executor: Arc<Executor<'static>>,
    _stop: channel::Sender<()>,
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
            limit: jobs.limit,
        })
    }

    pub async fn send(&self, msg: Message) {
        let inner = Arc::clone(&self.inner);
        let new_count =
            inner
                .background_count
                .fetch_update(Ordering::SeqCst, Ordering::SeqCst, |c| {
                    if c < self.limit { Some(c + 1) } else { None }
                });

        if new_count.is_ok() {
            self.executor.spawn(inner.send(msg, true)).detach();
        } else {
            inner.metrics.fg_sends.log();
            inner.send(msg, false).await;
        }
    }
}
