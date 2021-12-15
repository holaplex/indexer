use indexer_core::db::models::ListingsTripleJoinRow;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Listing {
    #[serde(rename = "listingAddress")]
    pub address: String,
    pub ends_at: Option<String>,
    pub created_at: String,
    pub ended: bool,
    pub highest_bid: Option<i64>,
    pub last_bid_time: Option<String>,
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
            ends_at: ends_at.map(|e| e.to_string()),
            created_at: created_at.to_string(),
            ended,
            highest_bid,
            last_bid_time: last_bid_time.map(|t| t.to_string()),
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
}
