//! RabbitMQ transport components for `holaplex-indexer`.

#![deny(
    clippy::suspicious,
    clippy::style,
    missing_debug_implementations,
    missing_copy_implementations
)]
#![warn(clippy::pedantic, clippy::cargo, missing_docs)]

pub extern crate lapin;

/// Common traits and re-exports
pub mod prelude {
    pub use lapin;

    pub use crate::queue_type::QueueType;
}

/// An error originating in this crate
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// An error propagated from [`lapin`]
    #[error("AMQP error: {0:?}")]
    Lapin(#[from] lapin::Error),
    /// An error propagated from [`rmp_serde`] during encoding
    #[error("MsgPack encode error: {0:?}")]
    MsgEncode(#[from] rmp_serde::encode::Error),
    /// An error propagated from [`rmp_serde`] during decoding
    #[error("MsgPack decode error: {0:?}")]
    MsgDecode(#[from] rmp_serde::decode::Error),
    /// An error occurred related to a queue's configuration
    #[error("Invalid queue type for operation: {0}")]
    InvalidQueueType(&'static str),
}

#[allow(dead_code)]
pub(crate) type Result<T, E = Error> = std::result::Result<T, E>;

#[cfg(feature = "consumer")]
pub mod consumer;
#[cfg(feature = "consumer")]
pub mod dl_consumer;
#[cfg(feature = "geyser")]
pub mod geyser;
#[cfg(feature = "http-indexer")]
pub mod http_indexer;
#[cfg(feature = "producer")]
pub mod producer;
mod queue_type;
mod serialize;
#[cfg(feature = "suffix")]
pub mod suffix;

pub use queue_type::QueueType;
