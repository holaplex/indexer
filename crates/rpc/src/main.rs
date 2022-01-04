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

pub mod rpc;
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
        info!("Listening on {}", addr);

        let server = {
            use jsonrpc_http_server::hyper;

            let version_prefix = regex::Regex::new(&format!(
                r"^/?v{}/?(.*)",
                regex::escape(
                    &percent_encoding::utf8_percent_encode(
                        env!("CARGO_PKG_VERSION_MAJOR"),
                        percent_encoding::NON_ALPHANUMERIC
                    )
                    .to_string()
                )
            ))
            .unwrap();

            ServerBuilder::new(io)
                .request_middleware(move |mut r: hyper::Request<_>| {
                    match version_prefix.captures(r.uri().path()) {
                        Some(caps) => {
                            let mut b = hyper::Uri::builder();
                            let hyper::http::uri::Parts {
                                scheme,
                                authority,
                                path_and_query,
                                ..
                            } = r.uri().clone().into_parts();

                            if let Some(scheme) = scheme {
                                b = b.scheme(scheme);
                            }

                            if let Some(authority) = authority {
                                b = b.authority(authority);
                            }

                            let mut path = format!("/{}", &caps[1]);

                            if let Some(query) = path_and_query.unwrap().query() {
                                path = format!("{}?{}", path, query);
                            }

                            *r.uri_mut() = b.path_and_query(path).build().unwrap();

                            r.into()
                        },
                        None => hyper::Response::builder()
                            .status(hyper::StatusCode::BAD_REQUEST)
                            .body(
                                format!("API version mismatch, expected {:?}\n", version_prefix)
                                    .into(),
                            )
                            .unwrap()
                            .into(),
                    }
                })
                .start_http(&addr)
                .context("Failed to start RPC server")?
        };

        server.wait();

        Ok(())
    });
}
