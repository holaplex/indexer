use std::io::Cursor;

use futures_util::StreamExt;
use lapin::{
    message::Delivery,
    options::{BasicConsumeOptions, BasicPublishOptions, QueueDeclareOptions},
    types::FieldTable,
    BasicProperties, Connection,
};

use super::{check_confirm, handle_delivery, Message, Rpc, RpcError, RpcQueue};
use crate::{serialize, tag, Result};

/// A server consumer for responding to RPC calls
#[derive(Debug)]
pub struct Server<R> {
    name: String,
    rpc: R,
}

impl<R> Server<R> {
    /// Construct a new server using the given RPC listener and server name
    pub fn new(name: String, rpc: R) -> Self {
        Self { name, rpc }
    }
}

impl<R: Rpc> Server<R> {
    /// Listen for incoming messages on the given connection
    ///
    /// # Errors
    /// This function returns early with an error if an AMQP or network error
    /// occurs.
    pub async fn listen(&self, conn: &Connection) -> Result<()> {
        let RpcQueue {
            chan,
            exchange,
            queue,
        } = RpcQueue::new::<R>(conn, &self.name, QueueDeclareOptions {
            passive: false,
            durable: true,
            exclusive: false,
            auto_delete: false,
            nowait: false,
        })
        .await?;

        let mut consumer = chan
            .basic_consume(
                &queue,
                &tag::tag("rpc-server"),
                BasicConsumeOptions {
                    no_local: false,
                    no_ack: false,
                    exclusive: true,
                    nowait: false,
                },
                FieldTable::default(),
            )
            .await?;

        loop {
            let del = match consumer.next().await {
                Some(d) => d,
                None => break Ok(()),
            };

            let Delivery {
                properties,
                data,
                acker,
                ..
            } = del?;

            handle_delivery(acker, || async {
                let (reply_to, correlation) = if let Some((reply, corr)) = properties
                    .reply_to()
                    .as_ref()
                    .zip(properties.correlation_id().as_ref())
                {
                    (reply.as_str(), corr)
                } else {
                    return Err(RpcError::Ignored(
                        "incoming request missing reply-to or correlation-id",
                    ));
                };

                let ret =
                    if let Message::<R>::Call(args) = serialize::deserialize(Cursor::new(data))? {
                        self.rpc.handle(args)
                    } else {
                        return Err(RpcError::Ignored("message body was not Call"));
                    };

                let mut payload = vec![];
                serialize::serialize(&mut payload, &Message::<R>::Ret(ret.0?))?;

                let confirm = chan
                    .basic_publish(
                        &exchange,
                        reply_to,
                        BasicPublishOptions {
                            mandatory: true,
                            immediate: false,
                        },
                        &payload,
                        BasicProperties::default().with_correlation_id(correlation.clone()),
                    )
                    .await?
                    .await?;

                check_confirm(&confirm).map_err(RpcError::Soft)?;

                Ok(())
            })
            .await?;
        }
    }
}
