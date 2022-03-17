//! Queue configuration for Solana `accountsdb` plugins intended to communicate
//! with `metaplex-indexer`.

use std::time::Duration;

use serde::{Deserialize, Serialize};
pub use solana_sdk::pubkey::Pubkey;

use crate::queue_type::{Binding, QueueInfo, QueueProps, RetryProps};

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
    props: QueueProps,
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

        let auto_delete = id.is_some() || cfg!(debug_assertions);

        Self {
            props: QueueProps {
                exchange,
                queue,
                binding: Binding::Fanout,
                prefetch: 4096,
                max_len_bytes: if auto_delete || matches!(startup_type, StartupType::Normal) {
                    100 * 1024 * 1024 // 100 MiB
                } else {
                    8 * 1024 * 1024 * 1024 // 8 GiB
                },
                auto_delete,
                retry: Some(RetryProps {
                    max_tries: 3,
                    delay_hint: Duration::from_millis(500),
                }),
            },
        }
    }
}

impl crate::QueueType for QueueType {
    type Message = Message;

    #[inline]
    fn info(&self) -> QueueInfo {
        (&self.props).into()
    }
}

/// The type of an `accountsdb` producer
#[cfg(feature = "producer")]
pub type Producer = crate::producer::Producer<QueueType>;
/// The type of an `accountsdb` consumer
#[cfg(feature = "consumer")]
pub type Consumer = crate::consumer::Consumer<QueueType>;
