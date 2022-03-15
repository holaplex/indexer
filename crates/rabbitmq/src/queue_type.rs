use std::{borrow::Cow, time::Duration};

use lapin::{Channel, Connection};

use crate::Result;

#[derive(Debug)]
pub struct RetryInfo<'a> {
    pub exchange: Cow<'a, str>,
    pub routing_key: Cow<'a, str>,
    pub max_tries: u32,
    pub delay_hint: Duration,
}

impl<'a> RetryInfo<'a> {
    pub fn into_owned(self) -> RetryInfo<'static> {
        let Self {
            exchange,
            routing_key,
            max_tries,
            delay_hint,
        } = self;

        RetryInfo {
            exchange: Cow::Owned(exchange.into_owned()),
            routing_key: Cow::Owned(routing_key.into_owned()),
            max_tries,
            delay_hint,
        }
    }
}

/// Trait containing the required infrastructure to create AMQP producers and
/// consumers conformant to a specific protocol.
#[async_trait::async_trait]
pub trait QueueType<T> {
    /// The expected exchange name of participating channels
    fn exchange(&self) -> Cow<str>;
    /// The expected queue name of participating channels
    fn queue(&self) -> Cow<str>;

    /// Initialize a new producer with the correct queue config
    async fn init_producer(&self, chan: &Channel) -> Result<()>;
    /// Initialize and return a consumer with the correct queue config
    async fn init_consumer(&self, chan: &Channel) -> Result<lapin::Consumer>;

    /// Information for controlling consumer retries
    fn retry_info(&self) -> Option<RetryInfo>;
    /// Initialize the dead letter consumer
    async fn init_dl_consumer(&self, chan: &Channel) -> Result<lapin::Consumer>;

    /// Publish options for producer basic_publish calls
    fn publish_opts(&self, msg: &T) -> lapin::options::BasicPublishOptions;
    /// Properties for producer basic_publish calls
    fn properties(&self, msg: &T) -> lapin::BasicProperties;
}

/// Helper trait for constructing a producer from a [`QueueType`]
#[cfg(any(test, feature = "producer"))]
#[async_trait::async_trait(?Send)]
#[allow(clippy::module_name_repetitions)]
pub trait QueueTypeProducerExt<T>: QueueType<T> + Sized {
    /// Create a new [`Producer`](crate::producer::Producer)
    async fn producer(self, conn: &Connection) -> Result<crate::producer::Producer<T, Self>>;
}

#[cfg(any(test, feature = "producer"))]
#[async_trait::async_trait(?Send)]
impl<T: serde::Serialize, Q: QueueType<T> + Sized> QueueTypeProducerExt<T> for Q {
    #[inline]
    async fn producer(self, conn: &Connection) -> Result<crate::producer::Producer<T, Self>> {
        crate::producer::Producer::new(conn, self).await
    }
}

/// Helper trait for constructing a consumer from a [`QueueType`]
#[cfg(any(test, feature = "consumer"))]
#[async_trait::async_trait(?Send)]
#[allow(clippy::module_name_repetitions)]
pub trait QueueTypeConsumerExt<T>: QueueType<T> + Sized {
    /// Create a new [`Consumer`](crate::consumer::Consumer)
    async fn consumer(self, conn: &Connection) -> Result<crate::consumer::Consumer<T, Self>>;

    /// Run the dead-letter consumer for this queue type
    async fn dl_consume<S: std::future::Future<Output = ()>>(
        self,
        conn: &Connection,
        sleep: impl Fn(Duration) -> S + 'async_trait,
    );
}

#[cfg(any(test, feature = "consumer"))]
#[async_trait::async_trait(?Send)]
impl<T: for<'a> serde::Deserialize<'a>, Q: QueueType<T> + Sized> QueueTypeConsumerExt<T> for Q {
    #[inline]
    async fn consumer(self, conn: &Connection) -> Result<crate::consumer::Consumer<T, Self>> {
        crate::consumer::Consumer::new(conn, self).await
    }

    #[inline]
    async fn dl_consume<S: std::future::Future<Output = ()>>(
        self,
        conn: &Connection,
        sleep: impl Fn(Duration) -> S + 'async_trait,
    ) {
        crate::consumer::dl_consume(conn, self, sleep).await;
    }
}
