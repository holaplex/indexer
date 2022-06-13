use indexer_core::db::models;
use juniper::GraphQLUnion;
use objects::{
    bid_receipt::BidReceipt, graph_connection::GraphConnection, listing_receipt::ListingReceipt,
    nft::Nft, profile::TwitterProfile, purchase_receipt::PurchaseReceipt, wallet::Wallet,
};

use super::prelude::*;
use crate::schema::scalars::PublicKey;

#[derive(Debug, Clone)]
pub struct MintEvent {
    created_at: DateTime<Utc>,
    feed_event_id: String,
    twitter_handle: Option<String>,
    wallet_address: PublicKey<Wallet>,
    metadata_address: PublicKey<Nft>,
}

#[derive(Debug, Clone)]
pub struct FollowEvent {
    created_at: DateTime<Utc>,
    feed_event_id: String,
    twitter_handle: Option<String>,
    wallet_address: PublicKey<Wallet>,
    graph_connection_address: PublicKey<GraphConnection>,
}

#[graphql_object(Context = AppContext)]
impl FollowEvent {
    fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }

    fn wallet_address(&self) -> &PublicKey<Wallet> {
        &self.wallet_address
    }

    pub async fn profile(&self, ctx: &AppContext) -> FieldResult<Option<TwitterProfile>> {
        let twitter_handle = match self.twitter_handle {
            Some(ref t) => t.clone(),
            None => return Ok(None),
        };

        ctx.twitter_profile_loader
            .load(twitter_handle)
            .await
            .map_err(Into::into)
    }

    fn feed_event_id(&self) -> &str {
        &self.feed_event_id
    }

    fn graph_connection_address(&self) -> &PublicKey<GraphConnection> {
        &self.graph_connection_address
    }

    pub async fn connection(&self, ctx: &AppContext) -> FieldResult<Option<GraphConnection>> {
        ctx.graph_connection_loader
            .load(self.graph_connection_address.clone())
            .await
            .map_err(Into::into)
    }
}

#[derive(Debug, Clone)]
pub struct PurchaseEvent {
    created_at: DateTime<Utc>,
    feed_event_id: String,
    twitter_handle: Option<String>,
    wallet_address: PublicKey<Wallet>,
    purchase_receipt_address: PublicKey<PurchaseReceipt>,
}

#[graphql_object(Context = AppContext)]
impl PurchaseEvent {
    fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }

    fn wallet_address(&self) -> &PublicKey<Wallet> {
        &self.wallet_address
    }

    pub async fn profile(&self, ctx: &AppContext) -> FieldResult<Option<TwitterProfile>> {
        let twitter_handle = match self.twitter_handle {
            Some(ref t) => t.clone(),
            None => return Ok(None),
        };

        ctx.twitter_profile_loader
            .load(twitter_handle)
            .await
            .map_err(Into::into)
    }

    fn feed_event_id(&self) -> &str {
        &self.feed_event_id
    }

    fn purchase_receipt_address(&self) -> &PublicKey<PurchaseReceipt> {
        &self.purchase_receipt_address
    }

    pub async fn purchase(&self, ctx: &AppContext) -> FieldResult<Option<PurchaseReceipt>> {
        ctx.purchase_receipt_loader
            .load(self.purchase_receipt_address.clone())
            .await
            .map_err(Into::into)
    }
}

#[derive(Debug, Clone)]
pub struct OfferEvent {
    created_at: DateTime<Utc>,
    feed_event_id: String,
    twitter_handle: Option<String>,
    wallet_address: PublicKey<Wallet>,
    bid_receipt_address: PublicKey<BidReceipt>,
    lifecycle: String,
}

#[graphql_object(Context = AppContext)]
impl OfferEvent {
    fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }

    fn wallet_address(&self) -> &PublicKey<Wallet> {
        &self.wallet_address
    }

    pub async fn profile(&self, ctx: &AppContext) -> FieldResult<Option<TwitterProfile>> {
        let twitter_handle = match self.twitter_handle {
            Some(ref t) => t.clone(),
            None => return Ok(None),
        };

        ctx.twitter_profile_loader
            .load(twitter_handle)
            .await
            .map_err(Into::into)
    }

    fn feed_event_id(&self) -> &str {
        &self.feed_event_id
    }

    fn lifecycle(&self) -> &str {
        &self.lifecycle
    }

    fn bid_receipt_address(&self) -> &PublicKey<BidReceipt> {
        &self.bid_receipt_address
    }

    pub async fn offer(&self, ctx: &AppContext) -> FieldResult<Option<BidReceipt>> {
        ctx.bid_receipt_loader
            .load(self.bid_receipt_address.clone())
            .await
            .map_err(Into::into)
    }
}

#[derive(Debug, Clone)]
pub struct ListingEvent {
    created_at: DateTime<Utc>,
    feed_event_id: String,
    listing_receipt_address: PublicKey<ListingReceipt>,
    twitter_handle: Option<String>,
    wallet_address: PublicKey<Wallet>,
    lifecycle: String,
}

#[graphql_object(Context = AppContext)]
impl ListingEvent {
    fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }

    fn wallet_address(&self) -> &PublicKey<Wallet> {
        &self.wallet_address
    }

    pub async fn profile(&self, ctx: &AppContext) -> FieldResult<Option<TwitterProfile>> {
        let twitter_handle = match self.twitter_handle {
            Some(ref t) => t.clone(),
            None => return Ok(None),
        };

        ctx.twitter_profile_loader
            .load(twitter_handle)
            .await
            .map_err(Into::into)
    }

    fn feed_event_id(&self) -> &str {
        &self.feed_event_id
    }

    fn lifecycle(&self) -> &str {
        &self.lifecycle
    }

    fn listing_receipt_address(&self) -> &PublicKey<ListingReceipt> {
        &self.listing_receipt_address
    }

    pub async fn listing(&self, ctx: &AppContext) -> FieldResult<Option<ListingReceipt>> {
        ctx.listing_receipt_loader
            .load(self.listing_receipt_address.clone())
            .await
            .map_err(Into::into)
    }
}

#[graphql_object(Context = AppContext)]
impl MintEvent {
    fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }

    fn wallet_address(&self) -> &PublicKey<Wallet> {
        &self.wallet_address
    }

    pub async fn profile(&self, ctx: &AppContext) -> FieldResult<Option<TwitterProfile>> {
        let twitter_handle = match self.twitter_handle {
            Some(ref t) => t.clone(),
            None => return Ok(None),
        };

        ctx.twitter_profile_loader
            .load(twitter_handle)
            .await
            .map_err(Into::into)
    }

    fn feed_event_id(&self) -> &str {
        &self.feed_event_id
    }

    fn metadata_address(&self) -> &PublicKey<Nft> {
        &self.metadata_address
    }

    pub async fn nft(&self, ctx: &AppContext) -> FieldResult<Option<Nft>> {
        ctx.nft_loader
            .load(self.metadata_address.clone())
            .await
            .map_err(Into::into)
    }
}

#[derive(derive_more::From, GraphQLUnion)]
#[graphql(
  Context = AppContext,
)]
pub enum FeedEvent {
    Mint(MintEvent),
    Offer(OfferEvent),
    Listing(ListingEvent),
    Purchase(PurchaseEvent),
    Follow(FollowEvent),
}

#[derive(thiserror::Error, Debug)]
#[error("Invalid feed event variant")]
pub struct TryFromError;

impl TryFrom<models::CompleteFeedEvent> for FeedEvent {
    type Error = TryFromError;

    fn try_from(
        models::CompleteFeedEvent {
            id,
            created_at,
            wallet_address,
            twitter_handle,
            metadata_address,
            purchase_receipt_address,
            bid_receipt_address,
            offer_lifecycle,
            listing_receipt_address,
            listing_lifecycle,
            graph_connection_address,
        }: models::CompleteFeedEvent,
    ) -> Result<Self, Self::Error> {
        match (
            metadata_address,
            (bid_receipt_address, offer_lifecycle),
            (listing_receipt_address, listing_lifecycle),
            purchase_receipt_address,
            graph_connection_address,
        ) {
            (Some(metadata_address), (None, None), (None, None), None, None) => {
                Ok(Self::Mint(MintEvent {
                    feed_event_id: id,
                    created_at: DateTime::from_utc(created_at, Utc),
                    metadata_address: metadata_address.into(),
                    twitter_handle,
                    wallet_address: wallet_address.into(),
                }))
            },
            (None, (Some(bid_receipt_address), Some(lifecycle)), (None, None), None, None) => {
                Ok(Self::Offer(OfferEvent {
                    feed_event_id: id,
                    created_at: DateTime::from_utc(created_at, Utc),
                    bid_receipt_address: bid_receipt_address.into(),
                    lifecycle,
                    twitter_handle,
                    wallet_address: wallet_address.into(),
                }))
            },
            (None, (None, None), (Some(listing_receipt_address), Some(lifecycle)), None, None) => {
                Ok(Self::Listing(ListingEvent {
                    feed_event_id: id,
                    created_at: DateTime::from_utc(created_at, Utc),
                    listing_receipt_address: listing_receipt_address.into(),
                    lifecycle,
                    twitter_handle,
                    wallet_address: wallet_address.into(),
                }))
            },
            (None, (None, None), (None, None), Some(purchase_receipt_address), None) => {
                Ok(Self::Purchase(PurchaseEvent {
                    feed_event_id: id,
                    created_at: DateTime::from_utc(created_at, Utc),
                    purchase_receipt_address: purchase_receipt_address.into(),
                    twitter_handle,
                    wallet_address: wallet_address.into(),
                }))
            },
            (None, (None, None), (None, None), None, Some(graph_connection_address)) => {
                Ok(Self::Follow(FollowEvent {
                    feed_event_id: id,
                    created_at: DateTime::from_utc(created_at, Utc),
                    graph_connection_address: graph_connection_address.into(),
                    twitter_handle,
                    wallet_address: wallet_address.into(),
                }))
            },
            _ => {
                debug!("feed_event_id: {}", id);

                Err(TryFromError)
            },
        }
    }
}
