//! An AMQP consumer configured from a [`QueueType`]

use std::{collections::BTreeMap, marker::PhantomData, time::Duration};

use futures_util::StreamExt;
use lapin::{
    acker::Acker,
    message::Delivery,
    options::{BasicAckOptions, BasicPublishOptions},
    types::{AMQPValue, FieldTable},
    Connection,
};

use crate::{queue_type::RetryInfo, serialize::deserialize, QueueType, Result};

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
    fn get_delay(info: &RetryInfo, retries_left: u32) -> Option<u32> {
        let try_number = info.max_tries.checked_sub(retries_left)?;

        let multiplier = 2_u128.checked_pow(try_number)?;
        let millis = info.delay_hint.as_millis().checked_mul(multiplier)?;

        millis.try_into().ok()
    }

    async fn try_consume<T, Q: QueueType<T>>(conn: &Connection, ty: &Q) -> Result<()> {
        let chan = conn.create_channel().await?;
        let exchange = ty.exchange();
        let retry_info = ty
            .retry_info()
            .unwrap_or_else(|| unreachable!("Called dl_consume with no retry configured!"));
        let mut consumer = ty.init_dl_consumer(&chan).await?;

        while let Some(del) = consumer.next().await {
            const RETRIES_LEFT: &str = "x-retries-left";

            let del = del?;

            let Delivery {
                delivery_tag: _,
                exchange: _,
                routing_key: del_routing_key,
                redelivered: _,
                mut properties,
                data,
                acker,
            } = del;

            let headers = properties.headers().as_ref().map(FieldTable::inner);
            let retries_left = headers.and_then(|h| h.get(RETRIES_LEFT));

            let mut new_headers = headers.cloned().unwrap_or_else(BTreeMap::new);
            let new_exchange;
            let routing_key;

            let retries_left =
                if let Some(retries_left) = retries_left.and_then(AMQPValue::as_long_uint) {
                    if let Some(retries_left) = retries_left.checked_sub(1) {
                        new_exchange = exchange.as_ref();
                        routing_key = del_routing_key.as_str();

                        retries_left
                    } else {
                        // Bye-bye!
                        acker.ack(BasicAckOptions::default()).await?;

                        continue;
                    }
                } else {
                    new_exchange = retry_info.exchange.as_ref();
                    routing_key = "";

                    retry_info.max_tries
                };

            new_headers.insert(RETRIES_LEFT.into(), AMQPValue::LongUInt(retries_left));

            if let Some(delay) = get_delay(&retry_info, retries_left) {
                new_headers.insert("x-delay".into(), AMQPValue::LongUInt(delay));
            } else {
                log::warn!("Discarding DL delivery due to delay arithmetic error");
                acker.ack(BasicAckOptions::default()).await?;

                continue;
            }

            properties = properties.with_headers(new_headers.into());

            chan.basic_publish(
                new_exchange,
                routing_key,
                BasicPublishOptions::default(),
                &data,
                properties,
            )
            .await?;
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
