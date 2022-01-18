use std::marker::PhantomData;

use futures_util::StreamExt;
use lapin::{options::BasicAckOptions, Channel, Connection};

use crate::{serialize::deserialize, QueueType, Result};

#[derive(Debug)]
pub struct Consumer<T, Q> {
    chan: Channel,
    consumer: lapin::Consumer,
    _p: PhantomData<(T, Q)>,
}

impl<T: for<'a> serde::Deserialize<'a>, Q: QueueType<T>> Consumer<T, Q> {
    pub async fn new(conn: &Connection) -> Result<Self> {
        let chan = conn.create_channel().await?;

        let consumer = Q::init_consumer(&chan).await?;

        Ok(Self {
            chan,
            consumer,
            _p: PhantomData::default(),
        })
    }

    pub async fn consume(&mut self) -> Result<Option<T>> {
        let (_chan, delivery) = match self.consumer.next().await {
            Some(d) => d?,
            None => return Ok(None),
        };

        delivery.ack(BasicAckOptions::default()).await?;

        deserialize(std::io::Cursor::new(delivery.data)).map_err(Into::into)
    }
}
