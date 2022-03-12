//! Queue configuration for Solana `accountsdb` plugins intended to communicate
//! with `metaplex-indexer`.

use std::{borrow::Cow, time::Duration};

use lapin::{
    options::{
        BasicConsumeOptions, BasicPublishOptions, BasicQosOptions, ExchangeDeclareOptions,
        QueueBindOptions, QueueDeclareOptions,
    },
    types::{AMQPValue, FieldTable},
    BasicProperties, Channel, ExchangeKind,
};
use serde::{Deserialize, Serialize};
pub use solana_sdk::pubkey::Pubkey;

use crate::{queue_type::RetryInfo, Result};

/// Message data for an account update
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountUpdate {
    /// The account's public key
    pub key: Pubkey,
    /// The lamport balance of the account
    pub lamports: u64,
    /// The Solana program controlling this account
    pub owner: Pubkey,
    /// True if the account's data is an executable smart contract
    pub executable: bool,
    /// The next epoch for which this account will owe rent
    pub rent_epoch: u64,
    /// The binary data stored on this account
    pub data: Vec<u8>,
    /// Monotonic-increasing counter for sequencing on-chain writes
    pub write_version: u64,
    /// The slot in which this account was updated
    pub slot: u64,
    /// True if this update was triggered by a validator startup
    pub is_startup: bool,
}

/// A message transmitted by an `accountsdb` plugin
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Message {
    /// Indicates an account should be updated
    AccountUpdate(AccountUpdate),
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
    suffixed: bool,
    startup_type: StartupType,
    exchange: String,
    queue: String,
    dl_exchange: String,
    dl_queue: String,
    dl_key: String,
}

/// Network hint for declaring exchange and queue names
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, strum::EnumString, strum::Display)]
#[strum(serialize_all = "kebab-case")]
pub enum Network {
    /// Use the network ID `"mainnet"`
    Mainnet,
    /// Use the network ID `"devnet"`
    Devnet,
    /// Use the network ID `"testnet"`
    Testnet,
}

/// Startup message hint for declaring exchanges and queues
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, strum::EnumString, strum::Display)]
#[strum(serialize_all = "kebab-case")]
pub enum StartupType {
    /// Ignore startup messages
    Normal,
    /// Ignore non-startup messages
    Startup,
    /// Include all messages
    All,
}

impl StartupType {
    /// Construct a [`StartupType`] from the accountsdb `startup` filter.
    #[must_use]
    pub fn new(value: Option<bool>) -> Self {
        match value {
            None => Self::All,
            Some(false) => Self::Normal,
            Some(true) => Self::Startup,
        }
    }
}

impl QueueType {
    /// Construct a new queue configuration given the network this validator is
    /// connected to and an optional queue suffix
    #[must_use]
    pub fn new(network: Network, startup_type: StartupType, id: Option<&str>) -> Self {
        let exchange = format!("{}{}.accounts", network, match startup_type {
            StartupType::Normal => "",
            StartupType::Startup => ".startup",
            StartupType::All => ".startup-all",
        });
        let mut queue = format!("{}.indexer", exchange);

        if let Some(id) = id {
            queue = format!("{}.{}", queue, id);
        }

        Self {
            suffixed: id.is_some(),
            startup_type,
            dl_exchange: format!("dlx.{}", exchange),
            dl_queue: format!("dlx.{}", queue),
            dl_key: id.map_or_else(String::new, ToOwned::to_owned),
            exchange,
            queue,
        }
    }

    async fn exchange_declare(&self, chan: &Channel) -> Result<()> {
        let mut exchg_fields = FieldTable::default();

        exchg_fields.insert(
            "x-dead-letter-exchange".into(),
            AMQPValue::LongString(self.dl_exchange.as_str().into()),
        );

        chan.exchange_declare(
            crate::QueueType::exchange(self).as_ref(),
            ExchangeKind::Fanout,
            ExchangeDeclareOptions::default(),
            exchg_fields,
        )
        .await?;

        Ok(())
    }

    async fn dl_exchange_declare(&self, chan: &Channel) -> Result<()> {
        let mut exchg_fields = FieldTable::default();

        exchg_fields.insert(
            "x-delayed-type".into(),
            AMQPValue::LongString("direct".into()),
        );

        exchg_fields.insert(
            "x-dead-letter-exchange".into(),
            AMQPValue::LongString(crate::QueueType::exchange(self).as_ref().into()),
        );

        chan.exchange_declare(
            self.dl_exchange.as_ref(),
            ExchangeKind::Custom("x-delayed-message".into()),
            ExchangeDeclareOptions {
                durable: true,
                ..ExchangeDeclareOptions::default()
            },
            exchg_fields,
        )
        .await?;

        Ok(())
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
        self.exchange_declare(chan).await?;

        Ok(())
    }

    async fn init_consumer(&self, chan: &Channel) -> Result<lapin::Consumer> {
        self.dl_exchange_declare(chan).await?;
        self.exchange_declare(chan).await?;

        let mut queue_fields = FieldTable::default();
        queue_fields.insert(
            "x-max-length-bytes".into(),
            AMQPValue::LongLongInt(
                if self.suffixed || matches!(self.startup_type, StartupType::Normal) {
                    100 * 1024 * 1024 // 100 MiB
                } else {
                    8 * 1024 * 1024 * 1024 // 8 GiB
                },
            ),
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

        chan.basic_qos(4096, BasicQosOptions::default()).await?;

        chan.basic_consume(
            self.queue().as_ref(),
            self.queue().as_ref(),
            BasicConsumeOptions::default(),
            FieldTable::default(),
        )
        .await
        .map_err(Into::into)
    }

    fn retry_info(&self) -> Option<RetryInfo> {
        Some(RetryInfo {
            exchange: self.dl_exchange.clone(),
            routing_key: self.dl_key.clone(),
            max_tries: 5,
            delay_hint: Duration::from_millis(500),
        })
    }

    async fn init_dl_consumer(&self, chan: &Channel) -> Result<lapin::Consumer> {
        self.exchange_declare(chan).await?;
        self.dl_exchange_declare(chan).await?;

        let mut queue_fields = FieldTable::default();
        queue_fields.insert(
            "x-max-length-bytes".into(),
            AMQPValue::LongLongInt(100 * 1024 * 1024), // 100 MiB
        );

        chan.queue_declare(
            self.dl_queue.as_ref(),
            QueueDeclareOptions::default(),
            queue_fields,
        )
        .await?;

        chan.queue_bind(
            self.dl_queue.as_ref(),
            self.dl_exchange.as_ref(),
            self.dl_key.as_ref(),
            QueueBindOptions::default(),
            FieldTable::default(),
        )
        .await?;

        chan.basic_qos(1024, BasicQosOptions::default()).await?;

        chan.basic_consume(
            self.dl_queue.as_ref(),
            self.dl_queue.as_ref(),
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
