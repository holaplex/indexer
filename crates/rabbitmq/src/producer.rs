//! An AMQP producer configured from a [`QueueType`]

use std::marker::PhantomData;

use lapin::{Channel, Connection};

use crate::{serialize::serialize, QueueType, Result};

/// A producer consisting of a configured channel and additional queue config
#[derive(Debug)]
pub struct Producer<T, Q> {
    chan: Channel,
    ty: Q,
    _p: PhantomData<T>,
}

impl<T: serde::Serialize, Q: QueueType<T>> Producer<T, Q> {
    /// Construct a new producer from a [`QueueType`]
    ///
    /// # Errors
    /// This function fails if the channel cannot be created and configured
    /// successfully.
    pub async fn new(conn: &Connection, ty: Q) -> Result<Self> {
        let chan = conn.create_channel().await?;

        ty.init_producer(&chan).await?;

        Ok(Self {
            chan,
            ty,
            _p: PhantomData::default(),
        })
    }

    /// Write a single message to this producer
    ///
    /// # Errors
    /// This function fails if the value cannot be serialized or the serialized
    /// payload cannot be transmitted.
    pub async fn write(&self, val: impl std::borrow::Borrow<T>) -> Result<()> {
        let val = val.borrow();

        let mut vec = Vec::new();
        serialize(&mut vec, val)?;

        self.chan
            .basic_publish(
                self.ty.exchange().as_ref(),
                self.ty.queue().as_ref(),
                self.ty.publish_opts(val),
                &vec,
                self.ty.properties(val),
            )
            .await?
            .await?;

        Ok(())
    }
}
