//! JSONRPC server to read data from `metaplex-indexer`

#![deny(
    clippy::suspicious,
    clippy::style,
    missing_debug_implementations,
    missing_copy_implementations
)]
#![warn(clippy::pedantic, clippy::cargo, missing_docs)]

use std::{env, net::SocketAddr};

use clap::Parser;
use indexer_core::db;
use jsonrpc_core::IoHandler;
use jsonrpc_http_server::ServerBuilder;
use prelude::*;
use rpc::Rpc;

mod prelude {
    pub use indexer_core::prelude::*;
    pub use solana_sdk::{bs58, pubkey::Pubkey};
}

mod rpc;
mod rpc_models;

#[derive(Parser)]
struct Opts {
    /// The port to listen on.
    #[clap(short, long, default_value_t = 3000, env = "PORT")]
    port: u16,
}

fn main() {
    indexer_core::run(|| {
        let Opts { port } = Opts::parse();

        let db = db::connect(
            env::var_os("DATABASE_READ_URL")
                .or_else(|| env::var_os("DATABASE_URL"))
                .ok_or_else(|| anyhow!("No value found for DATABASE_READ_URL or DATABASE_URL"))
                .map(move |v| v.to_string_lossy().into_owned())?,
        )
        .context("Failed to connect to Postgres")?;

        let mut io = IoHandler::new();
        io.extend_with(rpc::Server::new(db).to_delegate());

        let mut addr: SocketAddr = "127.0.0.1:3000".parse().unwrap();
        addr.set_port(port);

        let server = ServerBuilder::new(io)
            .start_http(&addr)
            .context("Failed to start RPC server")?;

        server.wait();

        Ok(())
    });
}
