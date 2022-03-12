//! An AMQP consumer configured from a [`QueueType`]

use std::{marker::PhantomData, time::Duration};

use futures_util::StreamExt;
use lapin::{acker::Acker, Connection};

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
    pub async fn read(&mut self) -> Result<Option<(T, Acker)>> {
        let delivery = match self.consumer.next().await {
            Some(d) => d?,
            None => return Ok(None),
        };

        let data = deserialize(std::io::Cursor::new(delivery.data))?;

        Ok(Some((data, delivery.acker)))
    }
}

/// Run the dead-letter consumer for a [`QueueType`]
pub async fn dl_consume<T, Q: QueueType<T>, S: std::future::Future<Output = ()>>(
    conn: impl std::borrow::Borrow<Connection>,
    ty: Q,
    sleep: impl Fn(Duration) -> S,
) {
    async fn try_consume<T, Q: QueueType<T>>(conn: &Connection, ty: &Q) -> Result<()> {
        let chan = conn.create_channel().await?;
        let mut consumer = ty.init_dl_consumer(&chan).await?;

        while let Some(del) = consumer.next().await {
            let del = del?;

            todo!("Process delivery {:?}", del);
        }

        Ok(())
    }

    loop {
        match try_consume(conn.borrow(), &ty).await {
            Ok(()) => (),
            Err(e) => {
                log::error!("Dead-letter consumer failed: {:?}", e);
                sleep(Duration::from_secs(5)).await;
            },
        }
    }
}
