use indexer_core::{chrono::prelude::*, db::models::ListingsTripleJoinRow};
use serde::{Deserialize, Serialize};

/// Wrapper to ensure timestamps returned from the indexer are properly
/// formatted
#[derive(Debug, Serialize, Deserialize)]
#[repr(transparent)]
#[serde(transparent)]
pub struct Timestamp(String);

impl Timestamp {
    pub fn from_utc(utc: NaiveDateTime) -> Self {
        Self::from(DateTime::from_utc(utc, Utc))
    }
}

impl From<DateTime<Utc>> for Timestamp {
    fn from(dt: DateTime<Utc>) -> Self {
        Self(dt.to_rfc3339_opts(SecondsFormat::Secs, true))
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Listing {
    #[serde(rename = "listingAddress")]
    pub address: String,
    pub ends_at: Option<Timestamp>,
    pub created_at: Timestamp,
    pub ended: bool,
    pub highest_bid: Option<i64>,
    pub last_bid_time: Option<Timestamp>,
    pub price_floor: Option<i64>,
    pub total_uncancelled_bids: Option<i32>,
    pub instant_sale_price: Option<i64>,
    pub subdomain: String,
    pub store_title: String,
    pub items: Vec<ListingItem>,
}

impl From<ListingsTripleJoinRow> for Listing {
    fn from(
        ListingsTripleJoinRow {
            address,
            ends_at,
            created_at,
            ended,
            highest_bid,
            last_bid_time,
            price_floor,
            total_uncancelled_bids,
            instant_sale_price,
            subdomain,
            store_title,
            meta_address,
            name,
            uri,
        }: ListingsTripleJoinRow,
    ) -> Self {
        Self {
            address,
            ends_at: ends_at.map(Timestamp::from_utc),
            created_at: Timestamp::from_utc(created_at),
            ended,
            highest_bid,
            last_bid_time: last_bid_time.map(Timestamp::from_utc),
            price_floor,
            total_uncancelled_bids,
            instant_sale_price,
            subdomain,
            store_title,
            items: vec![ListingItem {
                address: meta_address,
                name,
                uri,
            }],
        }
    }
}

impl Extend<ListingsTripleJoinRow> for Listing {
    fn extend<I: IntoIterator<Item = ListingsTripleJoinRow>>(&mut self, rows: I) {
        self.items.extend(rows.into_iter().map(
            |ListingsTripleJoinRow {
                 address,
                 meta_address,
                 name,
                 uri,
                 ..
             }| {
                assert!(address == self.address);

                ListingItem {
                    address: meta_address,
                    name,
                    uri,
                }
            },
        ));
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListingItem {
    #[serde(rename = "metadataAddress")]
    pub address: String,
    pub name: String,
    pub uri: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Storefront {
    pub owner_address: String,
    pub subdomain: String,
    pub title: String,
    pub description: String,
    pub favicon_url: String,
    pub logo_url: String,
    pub updated_at: Option<Timestamp>,
}
