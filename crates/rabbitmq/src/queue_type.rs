use std::borrow::Cow;

use lapin::{Channel, Connection};

use crate::Result;

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
}

#[cfg(any(test, feature = "consumer"))]
#[async_trait::async_trait(?Send)]
impl<T: for<'a> serde::Deserialize<'a>, Q: QueueType<T> + Sized> QueueTypeConsumerExt<T> for Q {
    #[inline]
    async fn consumer(self, conn: &Connection) -> Result<crate::consumer::Consumer<T, Self>> {
        crate::consumer::Consumer::new(conn, self).await
    }
}
