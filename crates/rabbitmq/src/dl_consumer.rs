//! Handler for an AMQP dead-letter consumer configured from a [`QueueType`]

use std::{collections::BTreeMap, time::Duration};

use futures_util::StreamExt;
use lapin::{
    message::Delivery,
    options::{BasicAckOptions, BasicPublishOptions},
    types::{AMQPValue, FieldTable, ShortString},
    Connection,
};
use log::{debug, error, trace, warn};

use crate::{
    queue_type::{DLX_DEAD_KEY, DLX_LIVE_KEY},
    QueueType, Result,
};

enum RetryAction {
    DropUnexpected,
    DropMaxlen,
    Retry(u64),
    RedeliverLive,
}

#[derive(strum::EnumString, strum::Display)]
#[strum(serialize_all = "snake_case")]
enum DeathReason {
    Rejected,
    Expired,
    Maxlen,
    DeliveryLimit,
}

fn parse_x_death(
    headers: Option<&BTreeMap<ShortString, AMQPValue>>,
    live_queue: &str,
    dl_queue: &str,
) -> RetryAction {
    let mut queue_deaths = 0;
    let mut dlq_deaths = 0;

    for (reason, queue, count) in headers
        .and_then(|h| h.get("x-death"))
        .and_then(AMQPValue::as_array)
        .into_iter()
        .flat_map(|r| {
            r.as_slice().iter().filter_map(|f| {
                f.as_field_table().and_then(|t| {
                    let t = t.inner();
                    let reason: DeathReason = t
                        .get("reason")?
                        .as_long_string()?
                        .to_string()
                        .parse()
                        .ok()?;
                    let queue = t.get("queue")?.as_long_string()?.to_string();
                    let count: u64 = t.get("count")?.as_long_long_int()?.try_into().ok()?;

                    Some((reason, queue, count))
                })
            })
        })
    {
        debug!("Death: {} in queue {:?} (x{})", reason, queue, count);

        match reason {
            DeathReason::Rejected | DeathReason::Expired | DeathReason::DeliveryLimit
                if queue == live_queue =>
            {
                queue_deaths += count;
            },
            DeathReason::Expired if queue == dl_queue => dlq_deaths += count,
            DeathReason::Maxlen => return RetryAction::DropMaxlen,
            _ => return RetryAction::DropUnexpected,
        }
    }

    if queue_deaths > dlq_deaths {
        RetryAction::Retry(queue_deaths)
    } else {
        RetryAction::RedeliverLive
    }
}

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

        match parse_x_death(headers, inf.queue(), inf.dl_queue()) {
            RetryAction::DropUnexpected => {
                warn!("Dropping unexpected message in triage queue");
            },
            RetryAction::DropMaxlen => {
                debug!("Dropping message due to maxlen death");
            },
            RetryAction::Retry(0) => {
                error!("Got 0-death message in triage queue");
            },
            RetryAction::Retry(r) if r < inf.max_tries() => {
                if let Some(delay) = inf.get_delay(r) {
                    trace!("Retry message (retry {}, delay {}ms)", r, delay);

                    properties = properties.with_expiration(delay.to_string().into());

                    chan.basic_publish(
                        inf.exchange(),
                        DLX_DEAD_KEY,
                        BasicPublishOptions::default(),
                        &data,
                        properties,
                    )
                    .await?;
                } else {
                    warn!("Discarding DL delivery due to delay arithmetic error");
                }
            },
            RetryAction::Retry(r) => {
                // We hit the retry limit.  Bye-bye!
                trace!("Dropping dead letter after {} deaths", r);
            },
            RetryAction::RedeliverLive => {
                trace!("Redelivering dead letter");

                chan.basic_publish(
                    inf.exchange(),
                    DLX_LIVE_KEY,
                    BasicPublishOptions::default(),
                    &data,
                    properties,
                )
                .await?;
            },
        };

        acker.ack(BasicAckOptions::default()).await?;
    }

    Ok(())
}

/// Run the dead-letter consumer for a [`QueueType`]
pub async fn run<Q: QueueType, S: std::future::Future<Output = ()>>(
    conn: impl std::borrow::Borrow<Connection>,
    ty: Q,
    sleep: impl Fn(Duration) -> S,
) {
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
