use indexer_core::db::models;

use super::{nft::Nft, prelude::*, reward_center::RewardCenter, wallet::Wallet};
use crate::schema::scalars::{PublicKey, U64};

#[derive(Debug, Clone)]
pub struct RewardPayout {
    pub purchase_ticket: String,
    pub nft_address: PublicKey<Nft>,
    pub reward_center: PublicKey<RewardCenter>,
    pub buyer: Wallet,
    pub buyer_reward: U64,
    pub seller: Wallet,
    pub seller_reward: U64,
    pub created_at: NaiveDateTime,
    pub slot: U64,
    pub write_version: U64,
}

impl<'a> TryFrom<models::ReadRewardPayout<'a>> for RewardPayout {
    type Error = std::num::TryFromIntError;

    fn try_from(
        models::ReadRewardPayout {
            purchase_ticket,
            metadata,
            reward_center,
            buyer,
            buyer_twitter_handle,
            buyer_reward,
            seller,
            seller_twitter_handle,
            seller_reward,
            created_at,
            slot,
            write_version,
        }: models::ReadRewardPayout,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            purchase_ticket: purchase_ticket.into(),
            nft_address: metadata.into(),
            reward_center: reward_center.into(),
            buyer: Wallet::new(buyer.into(), buyer_twitter_handle),
            buyer_reward: buyer_reward.try_into().unwrap_or_default(),
            seller: Wallet::new(seller.into(), seller_twitter_handle),
            seller_reward: seller_reward.try_into().unwrap_or_default(),
            created_at,
            slot: slot.try_into()?,
            write_version: write_version.try_into()?,
        })
    }
}

#[graphql_object(Context = AppContext)]
impl RewardPayout {
    pub fn purchase_ticket(&self) -> &str {
        &self.purchase_ticket
    }

    pub async fn nft(&self, context: &AppContext) -> FieldResult<Option<Nft>> {
        context
            .nft_loader
            .load(self.nft_address.clone())
            .await
            .map_err(Into::into)
    }

    pub fn reward_center(&self) -> &PublicKey<RewardCenter> {
        &self.reward_center
    }

    pub fn buyer(&self) -> &Wallet {
        &self.buyer
    }

    pub fn buyer_reward(&self) -> U64 {
        self.buyer_reward
    }

    pub fn seller(&self) -> &Wallet {
        &self.seller
    }

    pub fn seller_reward(&self) -> U64 {
        self.seller_reward
    }

    pub fn created_at(&self) -> NaiveDateTime {
        self.created_at
    }

    pub fn slot(&self) -> U64 {
        self.slot
    }

    pub fn write_version(&self) -> U64 {
        self.write_version
    }
}
