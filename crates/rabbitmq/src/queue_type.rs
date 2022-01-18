use std::borrow::Cow;

use lapin::{Channel, Connection};

use crate::Result;

#[async_trait::async_trait]
pub trait QueueType<T> {
    fn exchange(&self) -> Cow<str>;
    fn queue(&self) -> Cow<str>;

    async fn init_producer(&self, chan: &Channel) -> Result<()>;
    async fn init_consumer(&self, chan: &Channel) -> Result<lapin::Consumer>;

    fn publish_opts(&self, msg: &T) -> lapin::options::BasicPublishOptions;
    fn properties(&self, msg: &T) -> lapin::BasicProperties;
}

#[cfg(any(test, feature = "producer"))]
#[async_trait::async_trait(?Send)]
pub trait QueueTypeProducerExt<T>: QueueType<T> + Sized {
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

#[cfg(any(test, feature = "consumer"))]
#[async_trait::async_trait(?Send)]
pub trait QueueTypeConsumerExt<T>: QueueType<T> + Sized {
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
