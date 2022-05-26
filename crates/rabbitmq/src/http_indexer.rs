//! Queue configuration for the HTTP-driven indexer to receive requests from
//! the Geyser consumer.

use std::{marker::PhantomData, time::Duration};

use serde::{Deserialize, Serialize};
use solana_program::pubkey::Pubkey;

use crate::{
    queue_type::{Binding, QueueProps, RetryProps},
    suffix::Suffix,
    Result,
};

/// AMQP configuration for HTTP indexers
#[derive(Debug)]
pub struct QueueType<E> {
    props: QueueProps,
    _p: PhantomData<fn(E) -> ()>,
}

impl<E> Clone for QueueType<E> {
    fn clone(&self) -> Self {
        let Self { props, .. } = self;

        Self {
            props: props.clone(),
            ..*self
        }
    }
}

/// Identifier for an entity type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, strum::EnumString, strum::Display)]
#[strum(serialize_all = "kebab-case")]
pub enum EntityId {
    /// Identifier for [MetadataJson] entities
    MetadataJson,
    /// Identifier for [StoreConfig] entities
    StoreConfig,
}

/// Type hints for declaring and using entity-specific exchanges and queues
pub trait Entity: std::fmt::Debug + Serialize + for<'a> Deserialize<'a> {
    /// The type of the [ID](Self::ID) constant
    type Id: std::fmt::Display;

    /// A name to use when declaring queues and exchanges
    const ID: Self::Id;
}

/// Fetch the off-chain JSON for a metadata account
#[derive(Debug, Serialize, Deserialize)]
pub struct MetadataJson {
    /// The address of the associated account
    pub meta_address: Pubkey,
    /// The URI to retrieve the file from
    pub uri: String,
    /// possibly the first verified creator
    pub first_verified_creator: Option<Pubkey>,
    /// A tuple of (slot, write_version)
    #[serde(default)]
    pub slot_info: (u64, u64),
}

impl Entity for MetadataJson {
    type Id = EntityId;

    const ID: EntityId = EntityId::MetadataJson;
}

/// Fetch the off-chain JSON config for a storefront
#[derive(Debug, Serialize, Deserialize)]
pub struct StoreConfig {
    /// The address of the associated store config
    pub config_address: Pubkey,
    /// The URI to retrieve the file from
    pub uri: String,
}

impl Entity for StoreConfig {
    type Id = EntityId;

    const ID: EntityId = EntityId::StoreConfig;
}

impl<E: Entity> QueueType<E> {
    /// Construct a new queue configuration given the expected sender and
    /// queue suffix configuration
    ///
    /// # Errors
    /// This function fails if the given queue suffix is invalid.
    pub fn new(sender: &str, suffix: &Suffix) -> Result<Self> {
        let exchange = format!("{}.{}.http", sender, E::ID);
        let queue = suffix.format(format!("{}.indexer", exchange))?;

        Ok(Self {
            props: QueueProps {
                exchange,
                queue,
                binding: Binding::Fanout,
                prefetch: 1024,
                max_len_bytes: 100 * 1024 * 1024, // 100 MiB
                auto_delete: suffix.is_debug(),
                retry: Some(RetryProps {
                    max_tries: 8,
                    delay_hint: Duration::from_secs(2),
                    max_delay: Duration::from_secs(6 * 60),
                }),
            },
            _p: PhantomData::default(),
        })
    }
}

impl<E: Entity> crate::QueueType for QueueType<E> {
    type Message = E;

    #[inline]
    fn info(&self) -> crate::queue_type::QueueInfo {
        (&self.props).into()
    }
}

/// The type of an HTTP indexer producer
#[cfg(feature = "producer")]
pub type Producer<E> = crate::producer::Producer<QueueType<E>>;
/// The type of an HTTP indexer consumer
#[cfg(feature = "consumer")]
pub type Consumer<E> = crate::consumer::Consumer<QueueType<E>>;
