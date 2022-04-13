use derive_more::From;
use indexer_core::db::models;
use juniper::GraphQLUnion;
use objects::{bid_receipt::BidReceipt, nft::Nft};

use super::prelude::*;
use crate::schema::scalars::PublicKey;

#[derive(Debug, Clone)]
pub struct MintEvent {
    created_at: DateTime<Utc>,
    feed_event_id: String,
    metadata_address: PublicKey<Nft>,
}

#[derive(Debug, Clone)]
pub struct OfferEvent {
    created_at: DateTime<Utc>,
    feed_event_id: String,
    bid_receipt_address: PublicKey<BidReceipt>,
    // TODO: Make graphql scalar for the enum
    lifecycle: String,
}

#[graphql_object(Context = AppContext)]
impl OfferEvent {
    fn created_at(&self) -> DateTime<Utc> {
        self.created_at
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

#[graphql_object(Context = AppContext)]
impl MintEvent {
    fn created_at(&self) -> DateTime<Utc> {
        self.created_at
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

#[derive(From, GraphQLUnion)]
#[graphql(
  Context = AppContext,
)]
pub enum FeedEvent {
    MintEvent(MintEvent),
    OfferEvent(OfferEvent),
}

impl<'a>
    TryFrom<(
        models::FeedEvent<'a>,
        Option<models::MintEvent<'a>>,
        Option<models::OfferEvent<'a>>,
    )> for FeedEvent
{
    // TODO: get to work with `type Error = std::num::TryFromIntError;`
    type Error = &'static str;

    fn try_from(
        (models::FeedEvent { id, created_at }, mint_event, offer_event): (
            models::FeedEvent,
            Option<models::MintEvent>,
            Option<models::OfferEvent>,
        ),
    ) -> Result<Self, Self::Error> {
        match (mint_event, offer_event) {
            (
                Some(models::MintEvent {
                    metadata_address, ..
                }),
                None,
            ) => Ok(Self::MintEvent(MintEvent {
                feed_event_id: id.into_owned().to_string(),
                created_at: DateTime::from_utc(created_at, Utc),
                metadata_address: metadata_address.into_owned().into(),
            })),
            (
                None,
                Some(models::OfferEvent {
                    bid_receipt_address,
                    lifecycle,
                    ..
                }),
            ) => Ok(Self::OfferEvent(OfferEvent {
                feed_event_id: id.into_owned().to_string(),
                created_at: DateTime::from_utc(created_at, Utc),
                bid_receipt_address: bid_receipt_address.into_owned().into(),
                lifecycle: lifecycle.to_string(),
            })),
            _ => Err("not a feed event variant"),
        }
    }
}
