//! JSONRPC server to read data from `metaplex-indexer`

#![deny(
    clippy::suspicious,
    clippy::style,
    missing_debug_implementations,
    missing_copy_implementations
)]
#![warn(clippy::pedantic, clippy::cargo, missing_docs)]

use std::net::SocketAddr;

use indexer_core::{clap, clap::Parser, db, ServerOpts};
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
    #[clap(flatten)]
    server: ServerOpts,
}

fn main() {
    indexer_core::run(|| {
        let Opts {
            server: ServerOpts { port },
        } = Opts::parse();

        let db = db::connect(db::ConnectMode::Read).context("Failed to connect to Postgres")?;

        let mut io = IoHandler::new();
        io.extend_with(rpc::Server::new(db).to_delegate());

        let mut addr: SocketAddr = "0.0.0.0:3000".parse().unwrap();
        addr.set_port(port);

        let server = ServerBuilder::new(io)
            .start_http(&addr)
            .context("Failed to start RPC server")?;

        server.wait();

        Ok(())
    });
}
