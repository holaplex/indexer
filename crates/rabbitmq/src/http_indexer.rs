//! Queue configuration for the HTTP-driven indexer to receive requests from
//! the `accountsdb` consumer.

use std::borrow::Cow;

use lapin::{
    options::{
        BasicConsumeOptions, BasicPublishOptions, ExchangeDeclareOptions, QueueBindOptions,
        QueueDeclareOptions,
    },
    types::FieldTable,
    BasicProperties, Channel, ExchangeKind,
};
use serde::{Deserialize, Serialize};
use solana_sdk::pubkey::Pubkey;

use crate::Result;

/// A message sent from an `accountsdb` indexer to an HTTP indexer
#[derive(Debug, Serialize, Deserialize)]
pub enum Message {
    /// Fetch the off-chain JSON for a metadata account
    MetadataJson {
        /// The address of the associated account
        meta_address: Pubkey,
        /// The URI to retrieve the file from
        uri: String,
    },
    /// Fetch the off-chain JSON config for a storefront
    StoreConfig {
        /// The address of the associated store
        store_address: Pubkey,
        /// The URI to retrieve the file from
        uri: String,
    },
}

/// AMQP configuration for HTTP indexers
#[derive(Debug, Clone)]
pub struct QueueType {
    exchange: String,
    queue: String,
}

impl QueueType {
    /// Construct a new queue configuration given an optional queue suffix
    #[must_use]
    pub fn new(id: Option<&str>) -> Self {
        // TODO
        let exchange = format!("indexer.http");
        let mut queue = format!("indexer.http");

        if let Some(id) = id {
            queue = format!("{}.{}", queue, id);
        }

        Self { exchange, queue }
    }
}

#[async_trait::async_trait]
impl crate::QueueType<Message> for QueueType {
    fn exchange(&self) -> Cow<str> {
        Cow::Borrowed(&self.exchange)
    }

    fn queue(&self) -> Cow<str> {
        Cow::Borrowed(&self.queue)
    }

    async fn init_producer(&self, chan: &Channel) -> Result<()> {
        chan.exchange_declare(
            self.exchange().as_ref(),
            ExchangeKind::Fanout,
            ExchangeDeclareOptions::default(),
            FieldTable::default(),
        )
        .await?;

        Ok(())
    }

    async fn init_consumer(&self, chan: &Channel) -> Result<lapin::Consumer> {
        chan.exchange_declare(
            self.exchange().as_ref(),
            ExchangeKind::Fanout,
            ExchangeDeclareOptions::default(),
            FieldTable::default(),
        )
        .await?;

        let queue_options = FieldTable::default();
        // TODO: work out a reasonable TTL
        // queue_options.insert("x-message-ttl".into(), AMQPValue::LongUInt(60000)); // ten minutes

        chan.queue_declare(
            self.queue().as_ref(),
            QueueDeclareOptions::default(),
            queue_options,
        )
        .await?;

        chan.queue_bind(
            self.queue().as_ref(),
            self.exchange().as_ref(),
            "",
            QueueBindOptions::default(),
            FieldTable::default(),
        )
        .await?;

        chan.basic_consume(
            self.queue().as_ref(),
            self.queue().as_ref(),
            BasicConsumeOptions::default(),
            FieldTable::default(),
        )
        .await
        .map_err(Into::into)
    }

    fn publish_opts(&self, _: &Message) -> BasicPublishOptions {
        BasicPublishOptions::default()
    }

    fn properties(&self, _: &Message) -> BasicProperties {
        BasicProperties::default()
    }
}

/// The type of an HTTP indexer producer
#[cfg(feature = "producer")]
pub type Producer = crate::producer::Producer<Message, QueueType>;
/// The type of an HTTP indexer consumer
#[cfg(feature = "consumer")]
pub type Consumer = crate::consumer::Consumer<Message, QueueType>;
