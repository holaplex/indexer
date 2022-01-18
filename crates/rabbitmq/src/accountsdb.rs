//! Queue configuration for Solana `accountsdb` plugins intended to communicate
//! with `metaplex-indexer`.

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

use crate::Result;

/// A 256-bit Solana public key
pub type Pubkey = [u8; 32];

/// A message transmitted by an `accountsdb` plugin
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Message {
    /// Indicates an account should be updated
    AccountUpdate {
        /// The account's public key
        key: Pubkey,
        /// The Solana program controlling this account
        owner: Pubkey,
        /// The binary data stored on this account
        data: Vec<u8>,
    },
    /// Indicates an instruction was included in a **successful** transaction
    InstructionNotify {
        /// The program this instruction was executed with
        program: Pubkey,
        /// The binary instruction opcode
        data: Vec<u8>,
        /// The account inputs to this instruction
        accounts: Vec<Pubkey>,
    },
}

/// AMQP configuration for `accountsdb` plugins
#[derive(Debug, Clone)]
pub struct QueueType {
    exchange: String,
    queue: String,
}

/// Network hint for declaring exchange and queue names
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, strum::EnumString, strum::Display)]
#[strum(serialize_all = "camelCase")]
pub enum Network {
    /// Use the network ID `"mainnet"`
    Mainnet,
    /// Use the network ID `"devnet"`
    Devnet,
    /// Use the network ID `"testnet"`
    Testnet,
}

impl QueueType {
    /// Construct a new queue configuration given the network this validator is
    /// connected to.
    #[must_use]
    pub fn new(network: Network) -> Self {
        let exchange = format!("{}.accounts", network);
        let queue = format!("{}.accounts.indexer", network);

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

        chan.queue_declare(
            self.queue().as_ref(),
            QueueDeclareOptions::default(),
            FieldTable::default(),
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

/// The type of an `accountsdb` producer
#[cfg(feature = "producer")]
pub type Producer = crate::producer::Producer<Message, QueueType>;
/// The type of an `accountsdb` consumer
#[cfg(feature = "consumer")]
pub type Consumer = crate::consumer::Consumer<Message, QueueType>;
