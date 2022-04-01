use std::time::Duration;

#[cfg(feature = "producer")]
use lapin::{options::BasicPublishOptions, publisher_confirm::PublisherConfirm, BasicProperties};
#[cfg(feature = "consumer")]
use {
    crate::Error,
    lapin::{
        options::{BasicConsumeOptions, BasicQosOptions, QueueBindOptions, QueueDeclareOptions},
        types::AMQPValue,
        Consumer,
    },
};
#[cfg(any(feature = "producer", feature = "consumer"))]
use {
    crate::Result,
    lapin::{options::ExchangeDeclareOptions, types::FieldTable, Channel, ExchangeKind},
};

/// A trait representing an AMQP queue with a specific message type and AMQP
/// configuration.
pub trait QueueType {
    /// The type of message vendored by this queue
    type Message;

    /// Expose the underlying queue info for this queue
    fn info(&self) -> QueueInfo;
}

#[derive(Debug, Clone)]
pub enum Binding {
    Fanout,
    Direct(String),
}

#[cfg(feature = "consumer")]
impl Binding {
    fn routing_key(&self) -> &str {
        match self {
            Self::Fanout => "",
            Self::Direct(k) => k.as_ref(),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct RetryProps {
    pub max_tries: u64,
    pub delay_hint: Duration,
    pub max_delay: Duration,
}

#[derive(Debug, Clone)]
pub struct QueueProps {
    pub exchange: String,
    pub queue: String,
    pub binding: Binding,
    pub prefetch: u16,
    pub max_len_bytes: i64,
    pub auto_delete: bool,
    pub retry: Option<RetryProps>,
}

#[derive(Debug, Clone, Copy)]
#[repr(transparent)]
pub struct QueueInfo<'a>(&'a QueueProps);

impl<'a> From<&'a QueueProps> for QueueInfo<'a> {
    fn from(props: &'a QueueProps) -> Self {
        Self(props)
    }
}

#[cfg(feature = "consumer")]
pub const DLX_DEAD_KEY: &str = "dead";
#[cfg(feature = "consumer")]
pub const DLX_LIVE_KEY: &str = "live";
#[cfg(feature = "consumer")]
pub const DLX_TRIAGE_KEY: &str = "triage";

#[cfg(any(feature = "producer", feature = "consumer"))]
impl<'a> QueueInfo<'a> {
    async fn exchange_declare(self, chan: &Channel) -> Result<()> {
        chan.exchange_declare(
            self.0.exchange.as_ref(),
            match self.0.binding {
                Binding::Fanout => ExchangeKind::Fanout,
                Binding::Direct(_) => ExchangeKind::Direct,
            },
            ExchangeDeclareOptions::default(),
            FieldTable::default(),
        )
        .await?;

        Ok(())
    }
}

#[cfg(feature = "producer")]
impl<'a> QueueInfo<'a> {
    pub(crate) async fn init_producer(self, chan: &Channel) -> Result<()> {
        self.exchange_declare(chan).await?;

        Ok(())
    }

    pub(crate) async fn publish(self, chan: &Channel, data: &[u8]) -> Result<PublisherConfirm> {
        chan.basic_publish(
            self.0.exchange.as_ref(),
            self.0.queue.as_ref(),
            BasicPublishOptions::default(),
            data,
            BasicProperties::default(),
        )
        .await
        .map_err(Into::into)
    }
}

#[cfg(feature = "consumer")]
impl<'a> QueueInfo<'a> {
    fn dl_exchange(self) -> String {
        format!("dlx.{}", self.0.queue)
    }

    fn dl_queue(self) -> String {
        format!("dlq.{}", self.0.queue)
    }

    fn dl_triage_queue(self) -> String {
        format!("triage.dlq.{}", self.0.queue)
    }

    async fn queue_declare(self, chan: &Channel) -> Result<()> {
        let mut queue_fields = FieldTable::default();

        queue_fields.insert(
            "x-max-length-bytes".into(),
            AMQPValue::LongLongInt(self.0.max_len_bytes),
        );

        queue_fields.insert(
            "x-dead-letter-exchange".into(),
            AMQPValue::LongString(self.dl_exchange().into()),
        );

        queue_fields.insert(
            "x-dead-letter-routing-key".into(),
            AMQPValue::LongString(DLX_TRIAGE_KEY.into()),
        );

        chan.queue_declare(
            self.0.queue.as_ref(),
            QueueDeclareOptions {
                auto_delete: self.0.auto_delete,
                ..QueueDeclareOptions::default()
            },
            queue_fields,
        )
        .await?;

        Ok(())
    }

    /// Returns (`dl_exchange`, `dl_queue`, `dl_triage_queue`)
    async fn dl_exchange_declare(self, chan: &Channel) -> Result<(String, String, String)> {
        let mut exchg_fields = FieldTable::default();

        exchg_fields.insert(
            "x-message-ttl".into(),
            AMQPValue::LongLongInt(
                self.0
                    .retry
                    .as_ref()
                    .ok_or(Error::InvalidQueueType("Missing retry info"))?
                    .max_delay
                    .as_millis()
                    .try_into()
                    .map_err(|_| Error::InvalidQueueType("Max delay overflowed i64"))?,
            ),
        );

        let exchg = self.dl_exchange();

        chan.exchange_declare(
            exchg.as_ref(),
            ExchangeKind::Direct,
            ExchangeDeclareOptions {
                durable: true,
                ..ExchangeDeclareOptions::default()
            },
            exchg_fields,
        )
        .await?;

        Ok((exchg, self.dl_queue(), self.dl_triage_queue()))
    }

    pub(crate) async fn init_consumer(self, chan: &Channel) -> Result<Consumer> {
        self.dl_exchange_declare(chan).await?;
        self.exchange_declare(chan).await?;
        self.queue_declare(chan).await?;

        chan.queue_bind(
            self.0.queue.as_ref(),
            self.0.exchange.as_ref(),
            self.0.binding.routing_key(),
            QueueBindOptions::default(),
            FieldTable::default(),
        )
        .await?;

        chan.basic_qos(self.0.prefetch, BasicQosOptions::default())
            .await?;

        chan.basic_consume(
            self.0.queue.as_ref(),
            self.0.queue.as_ref(),
            BasicConsumeOptions::default(),
            FieldTable::default(),
        )
        .await
        .map_err(Into::into)
    }

    pub(crate) async fn init_dl_consumer(
        self,
        chan: &Channel,
    ) -> Result<(Consumer, DlConsumerInfo)> {
        let (exchange, queue, triage_queue) = self.dl_exchange_declare(chan).await?;

        {
            let mut queue_fields = FieldTable::default();

            queue_fields.insert(
                "x-max-length-bytes".into(),
                AMQPValue::LongLongInt(self.0.max_len_bytes),
            );

            queue_fields.insert(
                "x-dead-letter-exchange".into(),
                AMQPValue::LongString(exchange.clone().into()),
            );

            queue_fields.insert(
                "x-dead-letter-routing-key".into(),
                AMQPValue::LongString(DLX_TRIAGE_KEY.into()),
            );

            chan.queue_declare(
                queue.as_ref(),
                QueueDeclareOptions {
                    auto_delete: self.0.auto_delete,
                    ..QueueDeclareOptions::default()
                },
                queue_fields,
            )
            .await?;

            chan.queue_bind(
                queue.as_ref(),
                exchange.as_ref(),
                DLX_DEAD_KEY,
                QueueBindOptions::default(),
                FieldTable::default(),
            )
            .await?;
        }

        {
            let mut queue_fields = FieldTable::default();
            queue_fields.insert(
                "x-max-length-bytes".into(),
                // Top out length at 100 MiB
                AMQPValue::LongLongInt(self.0.max_len_bytes.min(100 * 1024 * 1024)),
            );

            // TODO: add a true DL queue

            chan.queue_declare(
                triage_queue.as_ref(),
                QueueDeclareOptions {
                    auto_delete: self.0.auto_delete,
                    ..QueueDeclareOptions::default()
                },
                queue_fields,
            )
            .await?;

            chan.queue_bind(
                triage_queue.as_ref(),
                exchange.as_ref(),
                DLX_TRIAGE_KEY,
                QueueBindOptions::default(),
                FieldTable::default(),
            )
            .await?;
        }

        self.queue_declare(chan).await?;
        chan.queue_bind(
            self.0.queue.as_ref(),
            exchange.as_ref(),
            DLX_LIVE_KEY,
            QueueBindOptions::default(),
            FieldTable::default(),
        )
        .await?;

        chan.basic_qos(self.0.prefetch, BasicQosOptions::default())
            .await?;

        let consumer = chan
            .basic_consume(
                triage_queue.as_ref(),
                triage_queue.as_ref(),
                BasicConsumeOptions::default(),
                FieldTable::default(),
            )
            .await?;

        let retry = self
            .0
            .retry
            .ok_or(crate::Error::InvalidQueueType("Missing retry properties"))?;

        Ok((consumer, DlConsumerInfo {
            exchange,
            retry,
            queue: self.0.queue.clone(),
            dl_queue: queue,
        }))
    }
}

#[cfg(feature = "consumer")]
#[derive(Debug, Clone)]
pub(crate) struct DlConsumerInfo {
    exchange: String,
    queue: String,
    dl_queue: String,
    retry: RetryProps,
}

#[cfg(feature = "consumer")]
impl DlConsumerInfo {
    pub fn exchange(&self) -> &str {
        &self.exchange
    }

    pub fn queue(&self) -> &str {
        &self.queue
    }

    pub fn dl_queue(&self) -> &str {
        &self.dl_queue
    }

    pub fn max_tries(&self) -> u64 {
        self.retry.max_tries
    }

    /// Return the retry delay given the retry number
    pub fn get_delay(&self, retry_number: u64) -> Option<u64> {
        let multiplier = 2_u128.checked_pow(retry_number.checked_sub(1)?.try_into().ok()?)?;
        let millis = self.retry.delay_hint.as_millis().checked_mul(multiplier)?;

        millis.try_into().ok()
    }
}
