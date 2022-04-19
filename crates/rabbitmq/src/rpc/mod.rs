//! Basic implementation of RPC-over-AMQP.

use std::future::Future;

use lapin::{
    acker::Acker,
    options::{
        BasicAckOptions, BasicRejectOptions, ConfirmSelectOptions, ExchangeDeclareOptions,
        QueueBindOptions, QueueDeclareOptions,
    },
    publisher_confirm::Confirmation,
    types::FieldTable,
    Channel, Connection, ExchangeKind,
};
use log::warn;
use serde::{Deserialize, Serialize};
use serde_value::Value;

use crate::{Error, Result};

mod client;
mod server;

pub use client::Client;
pub use server::Server;

/// Return value of RPC calls
#[derive(Debug)]
#[repr(transparent)]
pub struct Return(Result<Value, serde_value::SerializerError>);

impl<T: Serialize> From<T> for Return {
    fn from(val: T) -> Self {
        Self(serde_value::to_value(val))
    }
}

/// An RPC interface, providing an exchange and handler
pub trait Rpc: Sized {
    /// ID hint for the exchange to which consumers should bind
    const ID: &'static str;

    /// The parameter type of call messages
    type Args: Serialize + for<'de> Deserialize<'de>;

    /// The type of a client proxy for marshalling call and return types
    type Proxy: From<Client<Self>>;

    /// Handler method for parsing an RPC call and producing a return
    fn handle(&self, args: Self::Args) -> Return;

    /// Construct a client for this interface
    #[must_use]
    fn client() -> Self::Proxy {
        Client::new().into()
    }

    /// Construct a server for this interface
    #[must_use]
    fn server(self, name: String) -> Server<Self> {
        Server::new(name, self)
    }
}

#[derive(Debug, Serialize, Deserialize)]
enum Message<R: Rpc> {
    Call(R::Args),
    Ret(Value),
}

struct RpcQueue {
    chan: Channel,
    exchange: String,
    queue: String,
}

impl RpcQueue {
    #[inline]
    async fn new<R: Rpc>(
        conn: &Connection,
        routing_key: &str,
        opts: QueueDeclareOptions,
    ) -> Result<Self> {
        let exchange = format!("rpc.{}", R::ID);
        let queue = format!("{}.{}", exchange, routing_key);
        let chan = conn.create_channel().await?;

        chan.exchange_declare(
            &exchange,
            ExchangeKind::Direct,
            ExchangeDeclareOptions {
                passive: false,
                durable: true,
                auto_delete: true,
                internal: false,
                nowait: false,
            },
            FieldTable::default(),
        )
        .await?;

        chan.queue_declare(&queue, opts, FieldTable::default())
            .await?;

        chan.queue_bind(
            &queue,
            &exchange,
            routing_key,
            QueueBindOptions { nowait: false },
            FieldTable::default(),
        )
        .await?;

        chan.confirm_select(ConfirmSelectOptions { nowait: false })
            .await?;

        Ok(Self {
            chan,
            exchange,
            queue,
        })
    }
}

#[inline]
async fn handle_delivery<F: Future<Output = Result<(), RpcError>>>(
    acker: Acker,
    f: impl FnOnce() -> F,
) -> Result<()> {
    let ret = f().await;

    match ret {
        Ok(()) => acker
            .ack(BasicAckOptions { multiple: false })
            .await
            .map_err(Into::into),
        Err(e) => {
            let ret = match e {
                // Explicitly non-fatal errors
                e @ (RpcError::Ignored(_) | RpcError::Soft(_)) => {
                    warn!("{}", e);
                    Ok(())
                },
                // Fatal errors (i.e. network or AMQP errors)
                RpcError::Other(e @ (Error::Lapin(_) | Error::MsgEncode(_))) => Err(e),
                // Non-fatal misc. errors
                RpcError::Other(e) => {
                    warn!("Handling RPC message failed: {}", e);
                    Ok(())
                },
            };

            acker
                .reject(BasicRejectOptions {
                    requeue: ret.is_err(),
                })
                .await?;

            ret
        },
    }
}

#[inline]
fn check_confirm(confirm: &Confirmation) -> Result<()> {
    match confirm {
        Confirmation::Ack(ref a) => a,
        Confirmation::Nack(ref n) => n,
        Confirmation::NotRequested => &None,
    }
    .as_ref()
    .and_then(|m| m.error())
    .map_or(Ok(()), |e| Err(lapin::Error::ProtocolError(e).into()))
}

#[derive(Debug, thiserror::Error)]
enum RpcError {
    #[error("Ignoring RPC message: {0}")]
    Ignored(&'static str),
    #[error("Soft error: {0}")]
    Soft(#[source] Error),
    #[error("{0}")]
    Other(#[source] Error),
}

impl<E: Into<Error>> From<E> for RpcError {
    fn from(err: E) -> Self {
        Self::Other(err.into())
    }
}
