//! An AMQP consumer configured from a [`QueueType`]

use std::marker::PhantomData;

use futures_util::StreamExt;
use lapin::{options::BasicAckOptions, Connection};

use crate::{serialize::deserialize, QueueType, Result};

/// A consumer consisting of a configured AMQP consumer and queue config
#[derive(Debug)]
pub struct Consumer<T, Q> {
    // chan: Channel,
    consumer: lapin::Consumer,
    // ty: Q,
    _p: PhantomData<(T, Q)>,
}

impl<T: for<'a> serde::Deserialize<'a>, Q: QueueType<T>> Consumer<T, Q> {
    /// Construct a new consumer from a [`QueueType`]
    ///
    /// # Errors
    /// This function fails if the consumer cannot be created and configured
    /// successfully.
    pub async fn new(conn: &Connection, ty: Q) -> Result<Self> {
        let chan = conn.create_channel().await?;

        let consumer = ty.init_consumer(&chan).await?;

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
    pub async fn read(&mut self) -> Result<Option<T>> {
        let (_chan, delivery) = match self.consumer.next().await {
            Some(d) => d?,
            None => return Ok(None),
        };

        delivery.ack(BasicAckOptions::default()).await?;

        deserialize(std::io::Cursor::new(delivery.data)).map_err(Into::into)
    }
}
