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

pub type Pubkey = [u8; 32];

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Message {
    AccountUpdate {
        key: Pubkey,
        owner: Pubkey,
        data: Vec<u8>,
    },
    InstructionNotify {
        program: Pubkey,
        data: Vec<u8>,
        accounts: Vec<Pubkey>,
    },
}

#[derive(Debug, Clone)]
pub struct QueueType {
    exchange: String,
    queue: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, strum::EnumString, strum::Display)]
#[strum(serialize_all = "camelCase")]
pub enum Network {
    Mainnet,
    Devnet,
    Testnet,
}

impl QueueType {
    pub fn new(network: Network) -> Self {
        let exchange = todo!();
        let queue = todo!();

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

#[cfg(feature = "producer")]
pub type Producer = crate::producer::Producer<Message, QueueType>;
#[cfg(feature = "consumer")]
pub type Consumer = crate::consumer::Consumer<Message, QueueType>;
