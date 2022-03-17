//! Handler for an AMQP dead-letter consumer configured from a [`QueueType`]

use std::{collections::BTreeMap, time::Duration};

use futures_util::StreamExt;
use lapin::{
    message::Delivery,
    options::{BasicAckOptions, BasicPublishOptions},
    types::{AMQPValue, FieldTable},
    Connection,
};

use crate::{QueueType, Result};

/// Run the dead-letter consumer for a [`QueueType`]
pub async fn run<Q: QueueType, S: std::future::Future<Output = ()>>(
    conn: impl std::borrow::Borrow<Connection>,
    ty: Q,
    sleep: impl Fn(Duration) -> S,
) {
    async fn try_consume<Q: QueueType>(conn: &Connection, ty: &Q) -> Result<()> {
        let chan = conn.create_channel().await?;
        let (mut consumer, inf) = ty.info().init_dl_consumer(&chan).await?;

        while let Some(del) = consumer.next().await {
            const RETRIES_LEFT: &str = "x-retries-left";

            let del = del?;

            let Delivery {
                mut properties,
                data,
                acker,
                ..
            } = del;

            let headers = properties.headers().as_ref().map(FieldTable::inner);
            let retries_left = headers.and_then(|h| h.get(RETRIES_LEFT));

            let mut new_headers = headers.cloned().unwrap_or_else(BTreeMap::new);
            let new_exchange;
            let routing_key;

            let retries_left =
                if let Some(retries_left) = retries_left.and_then(AMQPValue::as_long_uint) {
                    if let Some(retries_left) = retries_left.checked_sub(1) {
                        new_exchange = inf.live_exchange();
                        routing_key = inf.live_routing_key();

                        retries_left
                    } else {
                        // We hit 0 retries left.  Bye-bye!
                        acker.ack(BasicAckOptions::default()).await?;

                        continue;
                    }
                } else {
                    new_exchange = inf.dl_exchange();
                    routing_key = inf.dl_routing_key();

                    inf.max_tries()
                };

            new_headers.insert(RETRIES_LEFT.into(), AMQPValue::LongUInt(retries_left));

            if let Some(delay) = inf.get_delay(retries_left) {
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

            acker.ack(BasicAckOptions::default()).await?;
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
