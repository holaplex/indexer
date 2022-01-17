use lapin::{
    options::{BasicPublishOptions, ExchangeDeclareOptions},
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

#[derive(Debug, Clone, Copy)]
pub struct QueueType;

#[async_trait::async_trait]
impl crate::QueueType<Message> for QueueType {
    const EXCHANGE: &'static str = "accounts";
    const QUEUE: &'static str = "";

    async fn init_producer(chan: &Channel) -> Result<()> {
        chan.exchange_declare(
            Self::EXCHANGE,
            ExchangeKind::Fanout,
            ExchangeDeclareOptions::default(),
            FieldTable::default(),
        )
        .await?;

        Ok(())
    }

    async fn init_consumer(chan: &Channel) -> Result<()> {
        todo!();

        Ok(())
    }

    fn publish_opts(_: &Message) -> BasicPublishOptions {
        BasicPublishOptions::default()
    }

    fn properties(_: &Message) -> BasicProperties {
        BasicProperties::default()
    }
}

#[cfg(feature = "producer")]
pub type Producer = crate::producer::Producer<Message, QueueType>;
#[cfg(feature = "consumer")]
pub type Consumer = crate::consumer::Consumer<Message, QueueType>;
