use std::sync::Arc;

use indexer_rabbitmq::{
    geyser::{Message, Producer, QueueType, StartupType},
    lapin::{Connection, ConnectionProperties},
    suffix::Suffix,
};
use tokio::sync::{RwLock, RwLockReadGuard};

use crate::{
    config,
    metrics::{Counter, Metrics},
};

#[derive(Debug)]
pub struct Sender {
    amqp: config::Amqp,
    name: String,
    startup_type: StartupType,
    producer: RwLock<Producer>,
    metrics: Arc<Metrics>,
}

impl Sender {
    pub async fn new(
        amqp: config::Amqp,
        name: String,
        startup_type: StartupType,
        metrics: Arc<Metrics>,
    ) -> Result<Self, indexer_rabbitmq::Error> {
        let producer = Self::create_producer(&amqp, name.as_ref(), startup_type).await?;

        Ok(Self {
            amqp,
            name,
            startup_type,
            producer: RwLock::new(producer),
            metrics,
        })
    }

    async fn create_producer(
        amqp: &config::Amqp,
        name: impl Into<indexer_rabbitmq::lapin::types::LongString>,
        startup_type: StartupType,
    ) -> Result<Producer, indexer_rabbitmq::Error> {
        let conn = Connection::connect(
            &amqp.address,
            ConnectionProperties::default()
                .with_connection_name(name.into())
                .with_executor(tokio_executor_trait::Tokio::current())
                .with_reactor(tokio_reactor_trait::Tokio),
        )
        .await?;

        Producer::new(
            &conn,
            QueueType::new(amqp.network, startup_type, &Suffix::Production)?,
        )
        .await
    }

    async fn connect<'a>(
        &'a self,
        prod: RwLockReadGuard<'a, Producer>,
    ) -> Result<RwLockReadGuard<'a, Producer>, indexer_rabbitmq::Error> {
        // Anti-deadlock safeguard - force the current reader to hand us their
        // lock so we can make sure it's destroyed.
        std::mem::drop(prod);
        let mut prod = self.producer.write().await;

        *prod = Self::create_producer(&self.amqp, self.name.as_ref(), self.startup_type).await?;

        Ok(prod.downgrade())
    }

    pub async fn send(&self, msg: Message) {
        #[inline]
        fn log_err<E: std::fmt::Debug>(counter: &'_ Counter) -> impl FnOnce(E) + '_ {
            |err| {
                counter.log(1);
                log::error!("{:?}", err);
            }
        }

        let metrics = &self.metrics;
        let prod = self.producer.read().await;

        match prod.write(&msg).await.map_err(log_err(&metrics.errs)) {
            Ok(()) => return,
            Err(()) => (),
        }

        metrics.reconnects.log(1);
        let prod = match self.connect(prod).await.map_err(log_err(&metrics.errs)) {
            Ok(p) => p,
            Err(()) => return,
        };

        match prod.write(&msg).await.map_err(log_err(&metrics.errs)) {
            Ok(()) | Err(()) => (), // Type-level assertion that we consumed the error
        }
    }
}
