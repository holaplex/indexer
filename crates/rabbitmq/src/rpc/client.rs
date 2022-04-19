use std::{io::Cursor, marker::PhantomData, sync::Arc};

use dashmap::DashMap;
use futures_util::{FutureExt, StreamExt};
use lapin::{
    message::Delivery,
    options::{BasicConsumeOptions, BasicPublishOptions, QueueDeclareOptions},
    types::{FieldTable, ShortString},
    BasicProperties, Channel, Connection, Consumer,
};
use log::warn;
use serde::Deserialize;
use serde_value::Value;
use tokio::sync::{oneshot, Mutex};

use super::{check_confirm, handle_delivery, Message, Rpc, RpcError, RpcQueue};
use crate::{serialize, tag, tag::Tag, Error, Result};

/// A client for an RPC interface, capable of calling methods.
#[derive(Debug)]
pub struct Client<R: ?Sized>(Mutex<Option<Arc<ClientInner<R>>>>);

impl<R: ?Sized> Client<R> {
    /// Construct a new client
    #[must_use]
    pub fn new() -> Self {
        Self(Mutex::new(None))
    }
}

impl<R: ?Sized> Default for Client<R> {
    fn default() -> Self {
        Self::new()
    }
}

impl<R: Rpc + ?Sized> Client<R> {
    /// Send an RPC call to the specified target over the given connection.
    ///
    /// **NOTE:** This function returns a future that may never resolve, and it
    /// is advisable to pair it with a timeout
    ///
    /// # Errors
    /// This function fails if a fatal network error occurs.  Note that it does
    /// not fail if the requested target does not respond.
    pub async fn send<T: for<'de> Deserialize<'de>>(
        &self,
        conn: &Connection,
        target: &str,
        args: R::Args,
    ) -> Result<T> {
        let inner = {
            let mut inner = self.0.lock().await;

            if let Some(ref strong) = *inner {
                strong.clone()
            } else {
                let new = Arc::new(ClientInner::new(conn).await?);

                *inner = Some(Arc::clone(&new));

                new
            }
        };

        let ret = inner.dispatch_request(target, args).await;

        // If we encounter a network error, rotate inner
        if matches!(ret, Err(Error::Lapin(_))) {
            *self.0.lock().await = None;
        }

        ret
    }
}

#[derive(Debug)]
struct ClientInner<R: ?Sized> {
    exchange: String,
    tag: Tag<&'static str>,
    chan: Channel,
    consumer: Consumer,
    replies: DashMap<ShortString, oneshot::Sender<Value>>,
    _r: PhantomData<fn(&R)>,
}

impl<R: Rpc + ?Sized> ClientInner<R> {
    async fn new(conn: &Connection) -> Result<Self> {
        let (tag, q) = loop {
            use lapin::protocol::{AMQPErrorKind, AMQPSoftError};

            let tag = Tag::new("client");

            let q = match RpcQueue::new::<R>(conn, &tag.to_string(), QueueDeclareOptions {
                passive: false,
                durable: false,
                exclusive: true,
                auto_delete: true,
                nowait: false,
            })
            .await
            {
                Ok(q) => q,
                Err(Error::Lapin(lapin::Error::ProtocolError(e)))
                    if matches!(
                        e.kind(),
                        AMQPErrorKind::Soft(
                            AMQPSoftError::RESOURCELOCKED | AMQPSoftError::PRECONDITIONFAILED
                        )
                    ) =>
                {
                    warn!("Error acquiring queue for {}: {}", tag, e);
                    continue;
                },
                Err(e) => return Err(e),
            };

            break (tag, q);
        };

        let RpcQueue {
            chan,
            exchange,
            queue,
        } = q;

        let consumer = chan
            .basic_consume(
                &queue,
                &tag::tag("rpc-client"),
                BasicConsumeOptions {
                    no_local: false,
                    no_ack: false,
                    exclusive: true,
                    nowait: false,
                },
                FieldTable::default(),
            )
            .await?;

        Ok(Self {
            exchange,
            tag,
            chan,
            consumer,
            replies: DashMap::new(),
            _r: PhantomData::default(),
        })
    }

    fn handle_reply(&self, id: &ShortString, body: Value) {
        if let Some((_id, tx)) = self.replies.remove(id) {
            match tx.send(body) {
                Ok(()) => (),
                Err(b) => warn!(
                    "Failed to submit reply {:?} to {} due to dead channel",
                    b, id
                ),
            }
        } else {
            warn!(
                "Failed to submit reply {:?} to {} due to unregistered handler",
                body, id
            );
        }
    }

    // TODO: T should be ! but that's not stable
    async fn consumer_loop<T>(&self) -> Result<T> {
        let mut consumer = self.consumer.clone();

        loop {
            let del = match consumer.next().await {
                Some(d) => d,
                None => break Err(Error::NoResponse),
            };

            let Delivery {
                properties,
                data,
                acker,
                ..
            } = del?;

            handle_delivery(acker, || async {
                let incoming_id = if let Some(id) = properties.correlation_id() {
                    id
                } else {
                    return Err(RpcError::Ignored(
                        "incoming response missing correlation-id",
                    ));
                };

                let ret = if let Message::<R>::Ret(ret) = serialize::deserialize(Cursor::new(data))?
                {
                    ret
                } else {
                    return Err(RpcError::Ignored("message body was not Ret"));
                };

                self.handle_reply(incoming_id, ret);

                Ok(())
            })
            .await?;
        }
    }

    async fn dispatch_request<T: for<'de> Deserialize<'de>>(
        &self,
        target: &str,
        args: R::Args,
    ) -> Result<T> {
        let mut payload = vec![];
        serialize::serialize(&mut payload, &Message::<R>::Call(args))?;

        let (correlation_id, rx) = {
            use dashmap::mapref::entry::Entry;
            use rand::prelude::*;

            let vacant = loop {
                let id = ShortString::from(format!("{:04x}", rand::thread_rng().gen::<u16>()));

                match self.replies.entry(id) {
                    Entry::Occupied(_) => (),
                    Entry::Vacant(v) => break v,
                }
            };

            let id = vacant.key().clone();
            let (tx, rx) = oneshot::channel();
            vacant.insert(tx);

            (id, rx)
        };

        // Ensure this gets removed
        let cleanup = dispose::defer(|| {
            self.replies.remove(&correlation_id);
        });

        let confirm = self
            .chan
            .basic_publish(
                &self.exchange,
                target,
                BasicPublishOptions {
                    mandatory: true,
                    immediate: false,
                },
                &payload,
                BasicProperties::default()
                    .with_correlation_id(correlation_id.clone())
                    .with_reply_to(self.tag.to_string().into()),
            )
            .await?
            .await?;

        check_confirm(&confirm)?;

        let val = futures_util::select! {
            reply = rx.fuse() => reply.map_err(|_| Error::NoResponse),
            res = self.consumer_loop().fuse() => res,
        }?;

        std::mem::drop(cleanup);

        T::deserialize(serde_value::ValueDeserializer::<rmp_serde::decode::Error>::new(val))
            .map_err(Into::into)
    }
}
