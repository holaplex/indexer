//! An interface for communicating with validators running the Holaplex
//! Geyser plugin.

use lapin::Connection;

use super::Return;
use crate::Result;

type Client = super::Client<Rpc>;
/// RPC server for the `holaplex-geyser` RPC interface
pub type Server<T> = super::Server<Handler<T>>;

/// Type-safe server handler for the `holaplex-geyser` RPC interface
pub trait Backend {
    /// Return the current config
    fn get_config(&self);
}

/// `holaplex-geyser` RPC interface declaration
#[derive(Debug, Clone, Copy)]
#[repr(transparent)]
pub struct Rpc;

/// Argument payload for `holaplex-geyser` RPC calls
#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize)]
#[repr(transparent)]
pub struct Args(Method);

#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize)]
enum Method {
    GetConfig,
}

/// Client proxy for the `holaplex-geyser` RPC interface
#[derive(Debug)]
#[repr(transparent)]
pub struct Proxy(Client);

/// Handler implementation for the `holaplex-geyser` RPC interface
#[derive(Debug, Clone, Copy)]
#[repr(transparent)]
pub struct Handler<T>(T);

impl super::Rpc for Rpc {
    const ID: &'static str = "geyser";

    type Args = Args;
    type Proxy = Proxy;
}

impl<T: Backend> super::Handler for Handler<T> {
    type Rpc = Rpc;

    fn handle(&self, Args(method): Args) -> Return {
        match method {
            Method::GetConfig => self.0.get_config().into(),
        }
    }
}

impl From<Client> for Proxy {
    fn from(client: Client) -> Self {
        Self(client)
    }
}

impl Proxy {
    #[inline]
    async fn send(&self, conn: &Connection, to: &str, method: Method) -> Result<()> {
        self.0.send(conn, to, Args(method)).await
    }

    /// Request the loaded configuration from a node
    /// # Errors
    /// See [`Client::send`]
    pub async fn get_config(&self, conn: &Connection, to: &str) -> Result<()> {
        self.send(conn, to, Method::GetConfig).await
    }
}
