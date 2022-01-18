use std::marker::PhantomData;

use lapin::{Channel, Connection};

use crate::{serialize::deserialize, QueueType, Result};

#[derive(Debug)]
pub struct Consumer<T, Q> {
    chan: Channel,
    consumer: lapin::Consumer,
    _p: PhantomData<(T, Q)>,
}

impl<'a, T: serde::Deserialize<'a>, Q: QueueType<T>> Consumer<T, Q> {
    pub async fn new(conn: &Connection) -> Result<Self> {
        let chan = conn.create_channel().await?;

        let consumer = Q::init_consumer(&chan).await?;

        Ok(Self {
            chan,
            consumer,
            _p: PhantomData::default(),
        })
    }

    pub async fn consume(&self) -> Result<T> {
        todo!()
    }
}
