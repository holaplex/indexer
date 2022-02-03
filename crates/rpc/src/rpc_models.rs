use indexer_core::{
    chrono::prelude::*,
    db::{
        models,
        models::{Bid, ListingsTripleJoinRow, MetadataCreator},
        queries::metadata_edition::MetadataEdition,
    },
};
use serde::{Deserialize, Serialize};

use crate::prelude::*;

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
pub struct Listing<L = (), I = ()> {
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
    pub logo_url: String,
    pub favicon_url: String,
    pub items: Vec<ListingItem<I>>,
    #[serde(flatten)]
    pub extra: L,
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
            logo_url,
            favicon_url,
            meta_address,
            name,
            uri,
            primary_sale_happened,
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
            logo_url,
            favicon_url,
            items: vec![ListingItem {
                address: meta_address,
                name,
                uri,
                primary_sale_happened,
                extra: (),
            }],
            extra: (),
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
                 primary_sale_happened,
                 ..
             }| {
                assert!(address == self.address);

                ListingItem {
                    address: meta_address,
                    name,
                    uri,
                    primary_sale_happened,
                    extra: (),
                }
            },
        ));
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListingItem<I = ()> {
    #[serde(rename = "metadataAddress")]
    pub address: String,
    pub name: String,
    pub uri: String,
    pub primary_sale_happened: bool,
    #[serde(flatten)]
    pub extra: I,
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
    pub banner_url: String,
}

impl<'a> From<models::Storefront<'a>> for Storefront {
    fn from(
        models::Storefront {
            owner_address,
            subdomain,
            title,
            description,
            favicon_url,
            logo_url,
            updated_at,
            banner_url,
        }: models::Storefront,
    ) -> Self {
        Self {
            owner_address: owner_address.into_owned(),
            subdomain: subdomain.into_owned(),
            title: title.into_owned(),
            description: description.into_owned(),
            favicon_url: favicon_url.into_owned(),
            logo_url: logo_url.into_owned(),
            updated_at: updated_at.map(Timestamp::from_utc),
            banner_url: banner_url.map_or_else(String::new, Cow::into_owned),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Creator {
    pub wallet_address: String,
}

impl<'a> From<MetadataCreator<'a>> for Creator {
    fn from(
        MetadataCreator {
            creator_address, ..
        }: MetadataCreator,
    ) -> Self {
        Self {
            wallet_address: creator_address.into_owned(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged, rename_all = "camelCase")]
pub enum ItemEdition {
    Edition {
        address: String,
        parent_address: String,
        edition: i64,
        supply: i64,
        max_supply: Option<i64>,
    },
    MasterEdition {
        address: String,
        supply: i64,
        max_supply: Option<i64>,
    },
}

impl<'a> From<MetadataEdition<'a>> for ItemEdition {
    fn from(edition: MetadataEdition<'a>) -> Self {
        match edition {
            MetadataEdition::Edition {
                edition:
                    models::Edition {
                        address,
                        parent_address,
                        edition,
                        metadata_address: _,
                    },
                parent:
                    models::MasterEdition {
                        address: _,
                        supply,
                        max_supply,
                        metadata_address: _,
                    },
            } => Self::Edition {
                address: address.into_owned(),
                parent_address: parent_address.into_owned(),
                edition,
                supply,
                max_supply,
            },
            MetadataEdition::MasterEdition(models::MasterEdition {
                address,
                supply,
                max_supply,
                metadata_address: _,
            }) => Self::MasterEdition {
                address: address.into_owned(),
                supply,
                max_supply,
            },
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListingBidder {
    bidder_address: String,
    last_bid_time: Timestamp,
    last_bid_amount: i64,
    cancelled: bool,
}

impl<'a> From<models::Bid<'a>> for ListingBidder {
    fn from(
        Bid {
            bidder_address,
            last_bid_time,
            last_bid_amount,
            cancelled,
            ..
        }: Bid,
    ) -> Self {
        Self {
            bidder_address: bidder_address.into_owned(),
            last_bid_time: Timestamp::from_utc(last_bid_time),
            last_bid_amount,
            cancelled,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListingExtra {
    bidders: Vec<ListingBidder>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ItemExtra {
    edition: Option<ItemEdition>,
    creators: Vec<Creator>,
}

pub type ListingDetails = Listing<ListingExtra, ItemExtra>;

impl ListingDetails {
    pub fn new(
        listing: Listing,
        get_bidders: impl FnOnce(&Listing) -> Result<Vec<ListingBidder>>,
        get_item_data: impl Fn(&ListingItem) -> Result<(Option<ItemEdition>, Vec<Creator>)>,
    ) -> Result<Self> {
        let bidders = get_bidders(&listing).context("Failed to get listing bids")?;

        let Listing {
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
            logo_url,
            favicon_url,
            items,
            extra: (),
        } = listing;

        let items = items
            .into_iter()
            .map(|item| {
                let (edition, creators) = get_item_data(&item)?;

                let ListingItem {
                    address,
                    name,
                    uri,
                    primary_sale_happened,
                    extra: (),
                } = item;

                Ok(ListingItem {
                    address,
                    name,
                    uri,
                    primary_sale_happened,
                    extra: ItemExtra { edition, creators },
                })
            })
            .collect::<Result<_>>()
            .context("Failed to get item data")?;

        Ok(Self {
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
            logo_url,
            favicon_url,
            items,
            extra: ListingExtra { bidders },
        })
    }
}
