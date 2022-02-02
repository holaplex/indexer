//! RabbitMQ transport components for `metaplex-indexer`.

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
    #[cfg(any(test, feature = "consumer"))]
    pub use crate::queue_type::QueueTypeConsumerExt;
    #[cfg(any(test, feature = "producer"))]
    pub use crate::queue_type::QueueTypeProducerExt;
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
}

pub(crate) type Result<T, E = Error> = std::result::Result<T, E>;

#[cfg(any(test, feature = "accountsdb"))]
pub mod accountsdb;
#[cfg(any(test, feature = "consumer"))]
pub mod consumer;
#[cfg(any(test, feature = "producer"))]
pub mod producer;
mod queue_type;
mod serialize;

pub use queue_type::QueueType;
