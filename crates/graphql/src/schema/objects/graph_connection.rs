use indexer_core::db::models;
use objects::wallet::Wallet;

use super::prelude::*;

#[derive(Debug, Clone)]
pub struct GraphConnection {
    pub address: String,
    pub from: Wallet,
    pub to: Wallet,
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
}

impl TryFrom<models::TwitterEnrichedGraphConnection> for GraphConnection {
    type Error = std::num::TryFromIntError;

    fn try_from(
        models::TwitterEnrichedGraphConnection {
            connection_address,
            from_account,
            to_account,
            from_twitter_handle,
            to_twitter_handle,
        }: models::TwitterEnrichedGraphConnection,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            address: connection_address,
            from: Wallet::new(from_account, from_twitter_handle),
            to: Wallet::new(to_account, to_twitter_handle),
        })
    }
}
