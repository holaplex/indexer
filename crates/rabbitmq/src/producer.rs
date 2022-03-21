//! An AMQP producer configured from a [`QueueType`]

use lapin::{Channel, Connection};

use crate::{serialize::serialize, QueueType, Result};

/// A producer consisting of a configured channel and additional queue config
#[derive(Debug)]
pub struct Producer<Q> {
    chan: Channel,
    ty: Q,
}

impl<Q: QueueType> Producer<Q>
where
    Q::Message: serde::Serialize,
{
    /// Construct a new producer from a [`QueueType`]
    ///
    /// # Errors
    /// This function fails if the channel cannot be created and configured
    /// successfully.
    pub async fn new(conn: &Connection, ty: Q) -> Result<Self> {
        let chan = conn.create_channel().await?;

        ty.info().init_producer(&chan).await?;

        Ok(Self { chan, ty })
    }

    /// Write a single message to this producer
    ///
    /// # Errors
    /// This function fails if the value cannot be serialized or the serialized
    /// payload cannot be transmitted.
    pub async fn write(&self, val: impl std::borrow::Borrow<Q::Message>) -> Result<()> {
        let val = val.borrow();

        let mut vec = Vec::new();
        serialize(&mut vec, val)?;

        self.ty.info().publish(&self.chan, &vec).await?.await?;

        Ok(())
    }
}
