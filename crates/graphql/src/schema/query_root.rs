use indexer_core::db::queries;
use objects::{
    auction_house::AuctionHouse,
    bid_receipt::BidReceipt,
    bonding_change::EnrichedBondingChange,
    chart::PriceChart,
    creator::Creator,
    denylist::Denylist,
    feed_event::FeedEvent,
    graph_connection::GraphConnection,
    listing::{Listing, ListingColumns, ListingRow},
    marketplace::Marketplace,
    nft::{Nft, NftActivity, NftCount, NftCreator},
    profile::{Profile, TwitterProfilePictureResponse, TwitterShowResponse},
    storefront::{Storefront, StorefrontColumns},
    wallet::Wallet,
};
use scalars::PublicKey;
use tables::{
    auction_caches, auction_datas, auction_datas_ext, bid_receipts, metadata_jsons, metadatas,
    store_config_jsons, storefronts,
};

use super::prelude::*;
pub struct QueryRoot;

#[derive(GraphQLInputObject, Clone, Debug)]
#[graphql(description = "Filter on NFT attributes")]
struct AttributeFilter {
    trait_type: String,
    values: Vec<String>,
}

impl From<AttributeFilter> for queries::metadatas::AttributeFilter {
    fn from(AttributeFilter { trait_type, values }: AttributeFilter) -> Self {
        Self { trait_type, values }
    }
}

#[graphql_object(Context = AppContext)]
impl QueryRoot {
    #[graphql(
        description = "Query feed events for a wallet. Returns events related to the specified user and events for the wallets the user follows.",
        arguments(
            wallet(description = "A user wallet public key"),
            limit(description = "The query record limit"),
            offset(description = "The query record offset")
        )
    )]
    fn feed_events(
        &self,
        ctx: &AppContext,
        wallet: PublicKey<Wallet>,
        limit: i32,
        offset: i32,
    ) -> FieldResult<Vec<FeedEvent>> {
        let conn = ctx.shared.db.get().context("failed to connect to db")?;

        let feed_events =
            queries::feed_event::list(&conn, wallet, limit.try_into()?, offset.try_into()?)?;

        feed_events
            .into_iter()
            .map(TryInto::try_into)
            .collect::<Result<_, _>>()
            .map_err(Into::into)
    }

    #[graphql(arguments(creators(description = "creators of nfts"),))]
    fn nft_counts(&self, creators: Vec<PublicKey<NftCreator>>) -> FieldResult<NftCount> {
        Ok(NftCount::new(creators))
    }
    #[graphql(arguments(
        auction_housese(description = "List of auction houses"),
        start_date(description = "Start date for which we want to get the average price"),
        end_date(description = "End date for which we want to get the average price")
    ))]
    pub async fn charts(
        &self,
        _context: &AppContext,
        auction_houses: Vec<PublicKey<AuctionHouse>>,
        start_date: DateTime<Utc>,
        end_date: DateTime<Utc>,
    ) -> FieldResult<PriceChart> {
        Ok(PriceChart {
            auction_houses,
            start_date,
            end_date,
        })
    }

    #[graphql(arguments(auction_housese(description = "List of auction houses"),))]
    pub async fn activities(
        &self,
        context: &AppContext,
        auction_houses: Vec<PublicKey<AuctionHouse>>,
    ) -> FieldResult<Vec<NftActivity>> {
        let conn = context.shared.db.get()?;
        let rows = queries::activities::list(&conn, auction_houses)?;

        rows.into_iter()
            .map(TryInto::try_into)
            .collect::<Result<_, _>>()
            .map_err(Into::into)
    }

    async fn profile(
        &self,
        ctx: &AppContext,
        #[graphql(description = "Twitter handle")] handle: String,
    ) -> Option<Profile> {
        let twitter_bearer_token = &ctx.shared.twitter_bearer_token;
        let http_client = reqwest::Client::new();

        let twitter_show_response: TwitterShowResponse = http_client
            .get("https://api.twitter.com/1.1/users/show.json")
            .header("Accept", "application/json")
            .query(&[("screen_name", &handle)])
            .bearer_auth(twitter_bearer_token)
            .send()
            .await
            .ok()?
            .json()
            .await
            .ok()?;

        let twitter_profile_picture_response: TwitterProfilePictureResponse = http_client
            .get(format!(
                "https://api.twitter.com/2/users/by/username/{}",
                handle
            ))
            .header("Accept", "application/json")
            .query(&[("user.fields", "profile_image_url")])
            .bearer_auth(twitter_bearer_token)
            .send()
            .await
            .ok()?
            .json()
            .await
            .ok()?;

        Some(Profile::from((
            twitter_profile_picture_response,
            twitter_show_response,
        )))
    }

    fn enriched_bonding_changes(
        &self,
        context: &AppContext,
        #[graphql(description = "The address of the bonding curve")] address: PublicKey<Wallet>,
        #[graphql(description = "The starting unix timestamp (inclusive)")]
        start_unix_time: NaiveDateTime,
        #[graphql(description = "The stop unix timestamp")] stop_unix_time: NaiveDateTime,
        #[graphql(description = "Query limit")] limit: i32,
        #[graphql(description = "Query offset")] offset: i32,
    ) -> FieldResult<Vec<EnrichedBondingChange>> {
        let conn = context.shared.db.get().context("Failed to connect to db")?;

        let rows = queries::bonding_changes::list(
            &conn,
            address,
            start_unix_time,
            stop_unix_time,
            limit,
            offset,
        )?;

        rows.into_iter()
            .map(TryInto::try_into)
            .collect::<Result<_, _>>()
            .map_err(Into::into)
    }

    fn offer(&self, context: &AppContext, address: String) -> FieldResult<Option<BidReceipt>> {
        let conn = context.shared.db.get().context("failed to connect to db")?;

        let row: Option<models::BidReceipt> = bid_receipts::table
            .select(bid_receipts::all_columns)
            .filter(bid_receipts::canceled_at.is_null())
            .filter(bid_receipts::purchase_receipt.is_null())
            .filter(bid_receipts::metadata.eq(address))
            .first(&conn)
            .optional()
            .context("Failed to load bid_receipts")?;

        row.map(TryInto::try_into).transpose().map_err(Into::into)
    }

    fn connections(
        &self,
        context: &AppContext,
        #[graphql(description = "Connections from a list of wallets")] from: Option<
            Vec<PublicKey<Wallet>>,
        >,
        #[graphql(description = "Connections to a list of wallets")] to: Option<
            Vec<PublicKey<Wallet>>,
        >,
        #[graphql(description = "Query limit")] limit: i32,
        #[graphql(description = "Query offset")] offset: i32,
    ) -> FieldResult<Vec<GraphConnection>> {
        if from.is_none() && to.is_none() {
            return Err(FieldError::new(
                "No filter provided! Please provide at least one of the filters",
                graphql_value!({ "Filters": "from: Vec<PublicKey>, to: Vec<PublicKey>" }),
            ));
        }
        let conn = context.shared.db.get().context("failed to connect to db")?;
        let from: Vec<String> = from
            .unwrap_or_else(Vec::new)
            .into_iter()
            .map(Into::into)
            .collect();
        let to: Vec<String> = to
            .unwrap_or_else(Vec::new)
            .into_iter()
            .map(Into::into)
            .collect();

        let rows = queries::graph_connection::connections(&conn, from, to, limit, offset)?;

        rows.into_iter()
            .map(TryInto::try_into)
            .collect::<Result<_, _>>()
            .map_err(Into::into)
    }

    fn creator(
        &self,
        context: &AppContext,
        #[graphql(description = "Address of creator")] address: String,
    ) -> FieldResult<Creator> {
        let conn = context.shared.db.get().context("failed to connect to db")?;

        let twitter_handle = queries::twitter_handle_name_service::get(&conn, &address)?;

        Ok(Creator {
            address,
            twitter_handle,
        })
    }

    fn nfts(
        &self,
        context: &AppContext,
        #[graphql(description = "Filter on owner address")] owners: Option<Vec<PublicKey<Wallet>>>,
        #[graphql(description = "Filter on creator address")] creators: Option<
            Vec<PublicKey<Wallet>>,
        >,
        #[graphql(description = "Filter on offerers address")] offerers: Option<
            Vec<PublicKey<Wallet>>,
        >,
        #[graphql(description = "Filter on attributes")] attributes: Option<Vec<AttributeFilter>>,
        #[graphql(description = "Filter on listed")] listed: Option<Vec<PublicKey<AuctionHouse>>>,
        #[graphql(description = "Filter on a collection")] collection: Option<PublicKey<Nft>>,
        #[graphql(description = "Limit for query")] limit: i32,
        #[graphql(description = "Offset for query")] offset: i32,
    ) -> FieldResult<Vec<Nft>> {
        if collection.is_none()
            && owners.is_none()
            && creators.is_none()
            && listed.is_none()
            && offerers.is_none()
        {
            return Err(FieldError::new(
                "No filter provided! Please provide at least one of the filters",
                graphql_value!({ "Filters": "owners: Vec<PublicKey>, creators: Vec<PublicKey>, offerers: Vec<PublicKey>, listed: Vec<PublicKey>" }),
            ));
        }

        let conn = context.shared.db.get().context("failed to connect to db")?;

        let query_options = queries::metadatas::ListQueryOptions {
            owners: owners.map(|a| a.into_iter().map(Into::into).collect()),
            creators: creators.map(|a| a.into_iter().map(Into::into).collect()),
            offerers: offerers.map(|a| a.into_iter().map(Into::into).collect()),
            attributes: attributes.map(|a| a.into_iter().map(Into::into).collect()),
            listed: listed.map(|a| a.into_iter().map(Into::into).collect()),
            collection: collection.map(|a| a.into()),
            limit: limit.into(),
            offset: offset.into(),
        };
        let nfts = queries::metadatas::list(&conn, query_options)?;

        Ok(nfts.into_iter().map(Into::into).collect())
    }

    fn wallet(
        &self,
        context: &AppContext,
        #[graphql(description = "Address of the wallet")] address: PublicKey<Wallet>,
    ) -> FieldResult<Wallet> {
        let conn = context.shared.db.get()?;

        let twitter_handle = queries::twitter_handle_name_service::get(&conn, &address)?;

        Ok(Wallet::new(address, twitter_handle))
    }

    fn listings(&self, context: &AppContext) -> FieldResult<Vec<Listing>> {
        let now = Local::now().naive_utc();
        let conn = context.shared.db.get()?;

        let rows: Vec<ListingRow> = auction_caches::table
            .inner_join(
                auction_datas::table.on(auction_caches::auction_data.eq(auction_datas::address)),
            )
            .inner_join(
                auction_datas_ext::table
                    .on(auction_caches::auction_ext.eq(auction_datas_ext::address)),
            )
            .inner_join(
                storefronts::table.on(storefronts::address.eq(auction_caches::store_address)),
            )
            .filter(
                queries::store_denylist::owner_address_ok(storefronts::owner_address).and(
                    queries::listing_denylist::listing_address_ok(auction_datas::address),
                ),
            )
            .select(ListingColumns::default())
            .load(&conn)
            .context("Failed to load listings")?;

        rows.into_iter()
            .map(|l| Listing::new(l, now))
            .collect::<Result<_, _>>()
            .map_err(Into::into)
    }

    fn nft(
        &self,
        context: &AppContext,
        #[graphql(description = "Address of NFT")] address: String,
    ) -> FieldResult<Option<Nft>> {
        let conn = context.shared.db.get()?;
        let mut rows: Vec<models::Nft> = metadatas::table
            .inner_join(
                metadata_jsons::table.on(metadatas::address.eq(metadata_jsons::metadata_address)),
            )
            .filter(metadatas::address.eq(address))
            .select((
                metadatas::address,
                metadatas::name,
                metadatas::seller_fee_basis_points,
                metadatas::mint_address,
                metadatas::primary_sale_happened,
                metadatas::uri,
                metadata_jsons::description,
                metadata_jsons::image,
                metadata_jsons::category,
            ))
            .limit(1)
            .load(&conn)
            .context("Failed to load metadata")?;

        Ok(rows.pop().map(Into::into))
    }

    fn storefronts(&self, context: &AppContext) -> FieldResult<Vec<Storefront>> {
        let conn = context.shared.db.get()?;
        let rows: Vec<models::Storefront> = storefronts::table
            .filter(queries::store_denylist::owner_address_ok(
                storefronts::owner_address,
            ))
            .select(StorefrontColumns::default())
            .load(&conn)
            .context("Failed to load storefront")?;

        Ok(rows.into_iter().map(Into::into).collect())
    }

    #[graphql(description = "A storefront")]
    fn storefront(
        &self,
        context: &AppContext,
        subdomain: String,
    ) -> FieldResult<Option<Storefront>> {
        let conn = context.shared.db.get()?;
        let mut rows: Vec<models::Storefront> = storefronts::table
            .filter(storefronts::subdomain.eq(subdomain))
            .select(StorefrontColumns::default())
            .limit(1)
            .load(&conn)
            .context("Failed to load storefront")?;

        Ok(rows.pop().map(Into::into))
    }

    #[graphql(description = "A marketplace")]
    fn marketplace(
        &self,
        context: &AppContext,
        subdomain: String,
    ) -> FieldResult<Option<Marketplace>> {
        let conn = context.shared.db.get()?;
        let mut rows: Vec<models::StoreConfigJson> = store_config_jsons::table
            .filter(store_config_jsons::subdomain.eq(subdomain))
            .select(store_config_jsons::all_columns)
            .limit(1)
            .load(&conn)
            .context("Failed to load store config JSON")?;

        Ok(rows.pop().map(Into::into))
    }

    fn denylist() -> Denylist {
        Denylist
    }
}
