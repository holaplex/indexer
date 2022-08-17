//! Support features for the job runner

use std::fmt;

use indexer_rabbitmq::job_runner::Message;

use crate::prelude::*;

/// Message identifier
#[derive(Debug, Clone, Copy)]
pub enum MessageId {}

impl fmt::Display for MessageId {
    fn fmt(&self, _: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {}
    }
}

/// Process a message from the background job RabbitMQ queue
///
/// # Errors
/// This function fails if an error occurs processing the message body.
pub async fn process_message(msg: Message) -> MessageResult<MessageId> {
    match msg {}
    std::future::pending().await
}
