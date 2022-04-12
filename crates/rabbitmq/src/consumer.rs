//! An AMQP consumer configured from a [`QueueType`]

use std::marker::PhantomData;

use futures_util::StreamExt;
use lapin::{acker::Acker, Connection};

use crate::{serialize::deserialize, QueueType, Result};

/// A consumer consisting of a configured AMQP consumer and queue config
#[derive(Debug)]
pub struct Consumer<Q> {
    // chan: Channel,
    consumer: lapin::Consumer,
    // ty: Q,
    _p: PhantomData<Q>,
}

impl<Q> Clone for Consumer<Q> {
    fn clone(&self) -> Self {
        let Self { consumer, .. } = self;

        Self {
            consumer: consumer.clone(),
            ..*self
        }
    }
}

impl<Q: QueueType> Consumer<Q>
where
    Q::Message: for<'a> serde::Deserialize<'a>,
{
    /// Construct a new consumer from a [`QueueType`]
    ///
    /// # Errors
    /// This function fails if the consumer cannot be created and configured
    /// successfully.
    pub async fn new(conn: &Connection, ty: Q, tag: impl AsRef<str>) -> Result<Self> {
        let chan = conn.create_channel().await?;

        let consumer = ty.info().init_consumer(&chan, tag).await?;

        Ok(Self {
            // chan,
            consumer,
            // ty,
            _p: PhantomData::default(),
        })
    }

    /// Receive a single message from this consumer
    ///
    /// # Errors
    /// This function fails if the delivery cannot be successfully performed or
    /// the payload cannot be deserialized.
    pub async fn read(&mut self) -> Result<Option<(Q::Message, Acker)>> {
        let delivery = match self.consumer.next().await {
            Some(d) => d?,
            None => return Ok(None),
        };

        let data = deserialize(std::io::Cursor::new(delivery.data))?;

        Ok(Some((data, delivery.acker)))
    }
}
