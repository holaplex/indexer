use objects::{nft::Nft, storefront::Storefront};
use scalars::U64;
use tables::{auction_caches, auction_datas, auction_datas_ext};

use super::prelude::*;

#[derive(Debug, Clone)]
/// A bid on an NFT listing
pub struct Bid {
    pub listing_address: String,
    pub bidder_address: String,
    pub last_bid_time: String,
    pub last_bid_amount: U64,
    pub cancelled: bool,
}

impl<'a> TryFrom<models::Bid<'a>> for Bid {
    type Error = std::num::TryFromIntError;

    fn try_from(
        models::Bid {
            listing_address,
            bidder_address,
            last_bid_time,
            last_bid_amount,
            cancelled,
            ..
        }: models::Bid,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            listing_address: listing_address.into_owned(),
            bidder_address: bidder_address.into_owned(),
            last_bid_time: last_bid_time.to_string(),
            last_bid_amount: last_bid_amount.try_into()?,
            cancelled,
        })
    }
}

#[graphql_object(Context = AppContext)]
impl Bid {
    pub fn listing_address(&self) -> &str {
        &self.listing_address
    }

    pub fn bidder_address(&self) -> &str {
        &self.bidder_address
    }

    pub fn last_bid_time(&self) -> &str {
        &self.last_bid_time
    }

    pub fn last_bid_amount(&self) -> U64 {
        self.last_bid_amount
    }

    pub fn cancelled(&self) -> bool {
        self.cancelled
    }

    pub async fn listing(&self, ctx: &AppContext) -> FieldResult<Option<Listing>> {
        ctx.listing_loader
            .load(self.listing_address.clone().into())
            .await
            .map_err(Into::into)
    }
}

pub type ListingColumns = (
    auction_datas::address,
    auction_datas_ext::address,
    auction_caches::address,
    auction_caches::store_address,
    auction_datas::token_mint,
    auction_datas::ends_at,
    auction_datas_ext::gap_tick_size,
    auction_datas::last_bid_time,
);

pub type ListingRow = (
    String,                // address
    String,                // ext_address
    String,                // cache_address
    String,                // store_address
    Option<String>,        // token_mint
    Option<NaiveDateTime>, // ends_at
    Option<i32>,           // gap_time
    Option<NaiveDateTime>, // last_bid_time
);

#[derive(Debug, Clone)]
/// A listing of for sale of an NFT
pub struct Listing {
    pub address: String,
    pub ext_address: String,
    pub cache_address: String,
    pub store_address: String,
    pub token_mint: Option<String>,
    pub ends_at: Option<DateTime<Utc>>,
    pub ended: bool,
}

impl Listing {
    pub fn address((address, ..): &ListingRow) -> String {
        address.clone()
    }

    pub fn new(
        (
            address,
            ext_address,
            cache_address,
            store_address,
            token_mint,
            ends_at,
            gap_time,
            last_bid_time,
        ): ListingRow,
        now: NaiveDateTime,
    ) -> Result<Self> {
        let (ends_at, ended) = indexer_core::util::get_end_info(
            ends_at,
            gap_time.map(|i| chrono::Duration::seconds(i.into())),
            last_bid_time,
            now,
        )?;

        Ok(Self {
            address,
            ext_address,
            cache_address,
            store_address,
            token_mint,
            ends_at: ends_at.map(|t| DateTime::from_utc(t, Utc)),
            ended,
        })
    }
}

#[graphql_object(Context = AppContext)]
impl Listing {
    pub fn address(&self) -> &str {
        &self.address
    }

    pub fn ext_address(&self) -> &str {
        &self.ext_address
    }

    pub fn cache_address(&self) -> &str {
        &self.cache_address
    }

    pub fn store_address(&self) -> &str {
        &self.store_address
    }

    pub fn ends_at(&self) -> Option<DateTime<Utc>> {
        self.ends_at
    }

    pub fn ended(&self) -> bool {
        self.ended
    }

    pub async fn storefront(&self, ctx: &AppContext) -> FieldResult<Option<Storefront>> {
        ctx.storefront_loader
            .load(self.store_address.clone().into())
            .await
            .map_err(Into::into)
    }

    pub async fn nfts(&self, ctx: &AppContext) -> FieldResult<Vec<Nft>> {
        ctx.listing_nfts_loader
            .load(self.address.clone().into())
            .await
            .map_err(Into::into)
            .map(|mut v| {
                v.sort_unstable_by_key(|(i, _)| *i);
                v.into_iter().map(|(_, n)| n).collect()
            })
    }

    pub async fn bids(&self, ctx: &AppContext) -> FieldResult<Vec<Bid>> {
        ctx.listing_bids_loader
            .load(self.address.clone().into())
            .await
            .map_err(Into::into)
    }
}
