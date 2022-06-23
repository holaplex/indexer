use indexer_core::db::models;
use objects::wallet::Wallet;

use super::prelude::*;

#[derive(Debug, Clone)]
pub struct GraphConnection {
    pub address: String,
    pub from: Wallet,
    pub to: Wallet,
    pub connected_at: DateTime<Utc>,
}

#[graphql_object(Context = AppContext)]
impl GraphConnection {
    pub fn address(&self) -> &str {
        &self.address
    }

    pub fn from(&self) -> &Wallet {
        &self.from
    }

    pub fn to(&self) -> &Wallet {
        &self.to
    }

    pub fn connected_at(&self) -> &DateTime<Utc> {
        &self.connected_at
    }
}

impl From<models::TwitterEnrichedGraphConnection> for GraphConnection {
    fn from(
        models::TwitterEnrichedGraphConnection {
            connection_address,
            from_account,
            to_account,
            from_twitter_handle,
            to_twitter_handle,
            connected_at,
            disconnected_at: _,
        }: models::TwitterEnrichedGraphConnection,
    ) -> Self {
        Self {
            address: connection_address,
            from: Wallet::new(from_account.into(), from_twitter_handle),
            to: Wallet::new(to_account.into(), to_twitter_handle),
            connected_at: DateTime::from_utc(connected_at, Utc),
        }
    }
}
