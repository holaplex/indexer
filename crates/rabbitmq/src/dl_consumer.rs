//! Handler for an AMQP dead-letter consumer configured from a [`QueueType`]

use std::time::Duration;

use futures_util::StreamExt;
use lapin::{
    message::Delivery,
    options::{BasicAckOptions, BasicPublishOptions},
    types::{AMQPValue, FieldTable},
    Connection,
};
use log::{error, trace, warn};

use crate::{queue_type::DLX_LIVE_KEY, QueueType, Result};

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
            let del = del?;

            let Delivery {
                mut properties,
                data,
                acker,
                ..
            } = del;

            let headers = properties.headers().as_ref().map(FieldTable::inner);

            // TODO
            let retry_number: Option<_> = headers
                .and_then(|h| h.get("x-death"))
                .and_then(AMQPValue::as_array)
                .map(|r| {
                    r.as_slice()
                        .iter()
                        .map(|f| {
                            f.as_field_table()
                                .and_then(|t| t.inner().get("count"))
                                .and_then(AMQPValue::as_long_long_int)
                                .and_then(|i| i.try_into().ok())
                                .unwrap_or(0_u64)
                        })
                        .sum()
                });

            match retry_number {
                None | Some(0) => {
                    error!("Got unexpected message in DLQ");
                },
                Some(r) if r < inf.max_tries() => {
                    if let Some(delay) = inf.get_delay(r) {
                        trace!("Retry message (retry {}, delay {}ms)", r, delay);

                        properties = properties.with_expiration(delay.to_string().into());

                        chan.basic_publish(
                            inf.exchange(),
                            DLX_LIVE_KEY,
                            BasicPublishOptions::default(),
                            &data,
                            properties,
                        )
                        .await?;
                    } else {
                        warn!("Discarding DL delivery due to delay arithmetic error");
                    }
                },
                Some(r) => {
                    // We hit the retry limit.  Bye-bye!
                    trace!("Dropping dead letter after {} deaths", r);
                },
            };

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
