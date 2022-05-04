//! Queue configuration for dispatching documents to be added to a search index.

use std::time::Duration;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{
    queue_type::{Binding, QueueProps, RetryProps},
    suffix::Suffix,
    Result,
};

/// Message data for a document upsert request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Document {
    /// The unique ID of the document
    pub id: String,
    /// The body of the document
    pub body: Value,
}

/// Message data for a search index job
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Message {
    /// Upsert a document to the index
    Upsert {
        /// MeiliSearch index name
        /// Index stores set of documents
        index: String,
        /// MeiliSearch Document
        /// contains primary key and content
        document: Document,
    },
}

/// AMQP configuration for search indexers
#[derive(Debug, Clone)]
pub struct QueueType {
    props: QueueProps,
}

impl QueueType {
    /// Construct a new queue configuration given the expected sender and
    /// queue suffix configuration
    ///
    /// # Errors
    /// This function fails if the given queue suffix is invalid.
    pub fn new(sender: &str, suffix: &Suffix) -> Result<Self> {
        let exchange = format!("{}.search", sender);
        let queue = suffix.format(format!("{}.indexer", exchange))?;

        Ok(Self {
            props: QueueProps {
                exchange,
                queue,
                binding: Binding::Fanout,
                prefetch: 4096,
                max_len_bytes: 100 * 1024 * 1024, // 100 MiB
                auto_delete: suffix.is_debug(),
                retry: Some(RetryProps {
                    max_tries: 3,
                    delay_hint: Duration::from_millis(500),
                    max_delay: Duration::from_secs(10 * 60),
                }),
            },
        })
    }
}

impl crate::QueueType for QueueType {
    type Message = Message;

    #[inline]
    fn info(&self) -> crate::queue_type::QueueInfo {
        (&self.props).into()
    }
}

/// The type of an search indexer producer
#[cfg(feature = "producer")]
pub type Producer = crate::producer::Producer<QueueType>;
/// The type of an search indexer consumer
#[cfg(feature = "consumer")]
pub type Consumer = crate::consumer::Consumer<QueueType>;
