use std::marker::PhantomData;

use lapin::{Channel, Connection};

use crate::{serialize::serialize, QueueType, Result};

#[derive(Debug)]
pub struct Producer<T, Q> {
    chan: Channel,
    _p: PhantomData<(T, Q)>,
}

impl<T: serde::Serialize, Q: QueueType<T>> Producer<T, Q> {
    pub async fn new(conn: &Connection) -> Result<Self> {
        let chan = conn.create_channel().await?;

        Q::init_producer(&chan).await?;

        Ok(Self {
            chan,
            _p: PhantomData::default(),
        })
    }

    pub async fn write(&self, val: impl std::borrow::Borrow<T>) -> Result<()> {
        let val = val.borrow();

        let mut vec = Vec::new();
        serialize(&mut vec, val)?;

        self.chan
            .basic_publish(
                Q::EXCHANGE,
                Q::QUEUE,
                Q::publish_opts(val),
                vec,
                Q::properties(val),
            )
            .await?
            .await?;

        Ok(())
    }
}
