//! RabbitMQ transport components for `metaplex-indexer`.

#![deny(
    clippy::suspicious,
    clippy::style,
    missing_debug_implementations,
    missing_copy_implementations
)]
#![warn(clippy::pedantic, clippy::cargo, missing_docs)]

pub extern crate lapin;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("A RabbitMQ error occurred")]
    Lapin(#[from] lapin::Error),
    #[error("An error occurred while encoding a message")]
    MsgEncode(#[from] rmp_serde::encode::Error),
    #[error("An error occurred while decoding a message")]
    MsgDecode(#[from] rmp_serde::decode::Error),
}

pub type Result<T, E = Error> = std::result::Result<T, E>;

#[cfg(any(test, feature = "accountsdb"))]
pub mod accountsdb;
#[cfg(any(test, feature = "consumer"))]
pub mod consumer;
#[cfg(any(test, feature = "producer"))]
pub mod producer;
mod queue_type;
mod serialize;

pub use queue_type::QueueType;
