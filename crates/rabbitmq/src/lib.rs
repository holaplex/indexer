//! RabbitMQ transport components for `metaplex-indexer`.

#![deny(
    clippy::suspicious,
    clippy::style,
    missing_debug_implementations,
    missing_copy_implementations
)]
#![warn(clippy::pedantic, clippy::cargo, missing_docs)]

#[cfg(any(doc, test, feature = "account_db"))]
pub mod account_db;
#[cfg(any(doc, test, feature = "consumer"))]
pub mod consumer;
#[cfg(any(doc, test, feature = "producer"))]
pub mod producer;
mod serialize;
