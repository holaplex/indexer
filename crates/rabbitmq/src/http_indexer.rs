//! Queue configuration for the HTTP-driven indexer to receive requests from
//! the `accountsdb` consumer.

use std::{borrow::Cow, marker::PhantomData};

use lapin::{
    options::{
        BasicConsumeOptions, BasicPublishOptions, BasicQosOptions, ExchangeDeclareOptions,
        QueueBindOptions, QueueDeclareOptions,
    },
    types::{AMQPValue, FieldTable},
    BasicProperties, Channel, ExchangeKind,
};
use serde::{Deserialize, Serialize};
use solana_sdk::pubkey::Pubkey;

use crate::Result;

/// AMQP configuration for HTTP indexers
#[derive(Debug, Clone)]
pub struct QueueType<E> {
    suffixed: bool,
    exchange: String,
    queue: String,
    _p: PhantomData<fn(E) -> ()>,
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
    /// Construct a new queue configuration given an optional queue suffix
    #[must_use]
    pub fn new(sender: &str, id: Option<&str>) -> Self {
        let exchange = format!("{}.{}.http", sender, E::ID);
        let mut queue = format!("{}.indexer", exchange);

        if let Some(id) = id {
            queue = format!("{}.{}", queue, id);
        }

        Self {
            suffixed: id.is_some() || cfg!(debug_assertions),
            exchange,
            queue,
            _p: PhantomData::default(),
        }
    }

    async fn exchange_declare(&self, chan: &Channel) -> Result<()> {
        chan.exchange_declare(
            crate::QueueType::exchange(self).as_ref(),
            ExchangeKind::Fanout,
            ExchangeDeclareOptions::default(),
            FieldTable::default(),
        )
        .await?;

        Ok(())
    }
}

#[async_trait::async_trait]
impl<E: Entity> crate::QueueType<E> for QueueType<E> {
    fn exchange(&self) -> Cow<str> {
        Cow::Borrowed(&self.exchange)
    }

    fn queue(&self) -> Cow<str> {
        Cow::Borrowed(&self.queue)
    }

    async fn init_producer(&self, chan: &Channel) -> Result<()> {
        self.exchange_declare(chan).await?;

        Ok(())
    }

    async fn init_consumer(&self, chan: &Channel) -> Result<lapin::Consumer> {
        self.exchange_declare(chan).await?;

        let mut queue_fields = FieldTable::default();
        queue_fields.insert(
            "x-max-length-bytes".into(),
            AMQPValue::LongUInt(100 * 1024 * 1024), // 100 MiB
        );

        let mut queue_options = QueueDeclareOptions::default();

        if self.suffixed {
            queue_options.auto_delete = true;
        }

        chan.queue_declare(self.queue().as_ref(), queue_options, queue_fields)
            .await?;

        chan.queue_bind(
            self.queue().as_ref(),
            self.exchange().as_ref(),
            "",
            QueueBindOptions::default(),
            FieldTable::default(),
        )
        .await?;

        chan.basic_qos(512, BasicQosOptions::default()).await?;

        chan.basic_consume(
            self.queue().as_ref(),
            self.queue().as_ref(),
            BasicConsumeOptions::default(),
            FieldTable::default(),
        )
        .await
        .map_err(Into::into)
    }

    fn publish_opts(&self, _: &E) -> BasicPublishOptions {
        BasicPublishOptions::default()
    }

    fn properties(&self, _: &E) -> BasicProperties {
        BasicProperties::default()
    }
}

/// The type of an HTTP indexer producer
#[cfg(feature = "producer")]
pub type Producer<E> = crate::producer::Producer<E, QueueType<E>>;
/// The type of an HTTP indexer consumer
#[cfg(feature = "consumer")]
pub type Consumer<E> = crate::consumer::Consumer<E, QueueType<E>>;
