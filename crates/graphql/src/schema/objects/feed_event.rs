use indexer_core::{db::models, uuid::Uuid};
use juniper::GraphQLUnion;
use objects::{
    ah_listing::AhListing, ah_offer::Offer, ah_purchase::Purchase,
    graph_connection::GraphConnection, nft::BaseNft, profile::TwitterProfile, wallet::Wallet,
};

use super::prelude::*;
use crate::schema::scalars::PublicKey;

#[derive(Debug, Clone)]
pub struct MintEvent {
    created_at: DateTime<Utc>,
    feed_event_id: String,
    twitter_handle: Option<String>,
    wallet_address: PublicKey<Wallet>,
    metadata_address: PublicKey<BaseNft>,
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

    pub async fn wallet(&self, ctx: &AppContext) -> FieldResult<Wallet> {
        ctx.wallet(self.wallet_address.clone())
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
    purchase_id: Uuid,
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

    fn purchase_id(&self) -> &Uuid {
        &self.purchase_id
    }

    pub async fn purchase(&self, ctx: &AppContext) -> FieldResult<Option<Purchase>> {
        ctx.purchase_loader
            .load(self.purchase_id)
            .await
            .map_err(Into::into)
    }

    pub async fn wallet(&self, ctx: &AppContext) -> FieldResult<Wallet> {
        ctx.wallet(self.wallet_address.clone())
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
    offer_id: Uuid,
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

    fn offer_id(&self) -> &Uuid {
        &self.offer_id
    }

    pub async fn offer(&self, ctx: &AppContext) -> FieldResult<Option<Offer>> {
        ctx.offer_loader
            .load(self.offer_id)
            .await
            .map_err(Into::into)
    }

    pub async fn wallet(&self, ctx: &AppContext) -> FieldResult<Wallet> {
        ctx.wallet(self.wallet_address.clone())
            .await
            .map_err(Into::into)
    }
}

#[derive(Debug, Clone)]
pub struct ListingEvent {
    created_at: DateTime<Utc>,
    feed_event_id: String,
    listing_id: Uuid,
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

    fn listing_id(&self) -> &Uuid {
        &self.listing_id
    }

    pub async fn listing(&self, ctx: &AppContext) -> FieldResult<Option<AhListing>> {
        ctx.ah_listing_loader
            .load(self.listing_id)
            .await
            .map_err(Into::into)
    }

    pub async fn wallet(&self, ctx: &AppContext) -> FieldResult<Wallet> {
        ctx.wallet(self.wallet_address.clone())
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

    fn metadata_address(&self) -> &PublicKey<BaseNft> {
        &self.metadata_address
    }

    pub async fn nft(&self, ctx: &AppContext) -> FieldResult<Option<BaseNft>> {
        ctx.nft_loader
            .load(self.metadata_address.clone())
            .await
            .map_err(Into::into)
    }

    pub async fn wallet(&self, ctx: &AppContext) -> FieldResult<Wallet> {
        ctx.wallet(self.wallet_address.clone())
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
            purchase_id,
            offer_id,
            offer_lifecycle,
            listing_id,
            listing_lifecycle,
            graph_connection_address,
        }: models::CompleteFeedEvent,
    ) -> Result<Self, Self::Error> {
        match (
            metadata_address,
            (offer_id, offer_lifecycle),
            (listing_id, listing_lifecycle),
            purchase_id,
            graph_connection_address,
        ) {
            (Some(metadata_address), (None, None), (None, None), None, None) => {
                Ok(Self::Mint(MintEvent {
                    feed_event_id: id.to_string(),
                    created_at: DateTime::from_utc(created_at, Utc),
                    metadata_address: metadata_address.into(),
                    twitter_handle,
                    wallet_address: wallet_address.into(),
                }))
            },
            (None, (Some(offer_id), Some(lifecycle)), (None, None), None, None) => {
                Ok(Self::Offer(OfferEvent {
                    feed_event_id: id.to_string(),
                    created_at: DateTime::from_utc(created_at, Utc),
                    offer_id,
                    lifecycle: lifecycle.to_string(),
                    twitter_handle,
                    wallet_address: wallet_address.into(),
                }))
            },
            (None, (None, None), (Some(listing_id), Some(lifecycle)), None, None) => {
                Ok(Self::Listing(ListingEvent {
                    feed_event_id: id.to_string(),
                    created_at: DateTime::from_utc(created_at, Utc),
                    listing_id,
                    lifecycle: lifecycle.to_string(),
                    twitter_handle,
                    wallet_address: wallet_address.into(),
                }))
            },
            (None, (None, None), (None, None), Some(purchase_id), None) => {
                Ok(Self::Purchase(PurchaseEvent {
                    feed_event_id: id.to_string(),
                    created_at: DateTime::from_utc(created_at, Utc),
                    purchase_id,
                    twitter_handle,
                    wallet_address: wallet_address.into(),
                }))
            },
            (None, (None, None), (None, None), None, Some(graph_connection_address)) => {
                Ok(Self::Follow(FollowEvent {
                    feed_event_id: id.to_string(),
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
