use std::str::FromStr;

use indexer_core::db::{
    queries::{self, feed_event::EventType},
    tables::twitter_handle_name_services,
};
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
    listing_receipt::ListingReceipt,
    marketplace::Marketplace,
    nft::{MetadataJson, Nft, NftActivity, NftCount, NftCreator},
    profile::TwitterProfile,
    storefront::{Storefront, StorefrontColumns},
    wallet::Wallet,
};
use scalars::PublicKey;
use serde_json::Value;
use tables::{
    auction_caches, auction_datas, auction_datas_ext, bid_receipts, graph_connections,
    listing_receipts, metadata_jsons, metadatas, store_config_jsons, storefronts, wallet_totals,
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
        description = "Recommend wallets to follow.",
        arguments(
            wallet(description = "A user wallet public key"),
            limit(description = "The query record limit"),
            offset(description = "The query record offset")
        )
    )]
    fn follow_wallets(
        &self,
        ctx: &AppContext,
        wallet: Option<PublicKey<Wallet>>,
        limit: i32,
        offset: i32,
    ) -> FieldResult<Vec<Wallet>> {
        let conn = ctx.shared.db.get().context("failed to connect to db")?;

        let wallets_to_follow = vec![
            "NWswq7QR7E1i1jkdkddHQUFtRPihqBmJ7MfnMCcUf4H",
            "3XzWJgu5WEU3GV3mHkWKDYtMXVybUhGeFt7N6uwkcezF",
            "2BNABAPHhYAxjpWRoKKnTsWT24jELuvadmZALvP6WvY4",
            "DCFWUYqK1iwGzSD3w6SzpwH77GgmbbVX3E2QRtN1qBrj",
            "x31BQteSUcoLcatn57pUPh1ATqDkGSBti4KpnXbqjMq",
            "14kVL6sWSc4oX9rwcJU7aMHMLMjsEfpXcAJf68kmsMeP",
            "3StkrkMEmdiu9nER5qM3fNR1fCMdawUhGsihF2UkXrPP",
            "DgzWqku67fPeuWgmyZbNa7U7yicKRhbjM7nB86Yx2ojQ",
            "Dz3BJenPAMziCBhJFGwUxvu3qhMUuLch8NjoZdfP9xsa",
            "AwiRcagxnT8NLnJeS8ScVcq2Y9f5VeJUfyfR5AXmVFfh",
            "AXXRH6NVXjUNxi6GvVWV4Pp6q7k9xkqeqHhRx8sW41TX",
            "79j2yWfDHnAU3Aq4yfoTcE9KCHCDo2m54aL4te67683Q",
            "Fh2rUc2CrMTp6H7t1CGnG4aXhWn7BzPPQBU2KkgR4jeh",
            "gNEt8EeWqdcSpebQXZ8YVnBC9k5yKp2WGvnA9HR8RzQ",
            "zenom3SnXK6k2UJm73jRQ1n8U7KkLPrTypDatKjGxoL",
            "Er6QJPusC1JsUqevTjFKXtYHbgCtJkyo1DNjEBWevWut",
            "HLSgM1a7wSufVwe1NrPPR22ynY2aPsH8a1XQfqFsQiqY",
            "BjSaYdgdWtBNXGJmy6cuTzxYzLcxvn4Anw6yrWsgKdNm",
            "8t6BxNBe7pM8YwvG4JUQxi1W9PYfuphrYcUiJF99oWsP",
            "4ZjYSCH3Sib9iMSM3QN2sL2kwxNcXG2P4XCemSC2hsyb",
            "GcpdC1iUtfiQ48B6dn7bcM2Ax13R6TDom65jQJiTD18G",
            "xYwSUQv7DX62XGo4XXFAQRSTwtS1NrWz8rifR7Gppeg",
            "A1Fk3zhtamLixGStRFc4eBd3pVodoFrNRbVFCaPaPJBu",
            "B3jtSCpXQpMZR5r5m87854bgMj5veHwz9idjd22eVrP7",
            "GpnKen3QMaLc1CzFsoy8UbcbPwEXRXb5k2qTkTcUa3RX",
            "H4dfSserFhYBswmPMF3FdYEbu4aj1AQt7RSXRyVqVjaS",
            "4Jb3dS76hxcBXKZDkwx3KC4NSMXoTKsyXfwW18apS4vZ",
            "yTM5APEbWb1GBBtgsjzTF6ZYw5pWxqCr7qKykWW7qLS",
            "2TKEfKKLreKYykZMCKiFMYhhkKFxajfSBeKNZ8rFa6qt",
            "CsxZxjL19pJ3DM4yyDZDePgCWj6m7Lm9htwHggBbgP1r",
            "57DsAWRijeENrhb4RdKiLjKzLF1V8f1J2D8mCNvGMSPu",
            "DWPpeotxT2Q1m1BDzuycQktDe6VnVejLiPWsjfwG6Nb7",
        ];

        let mut query = wallet_totals::table
            .left_join(
                twitter_handle_name_services::table
                    .on(wallet_totals::address.eq(twitter_handle_name_services::wallet_address)),
            )
            .select((
                (wallet_totals::all_columns),
                twitter_handle_name_services::twitter_handle.nullable(),
            ))
            .filter(wallet_totals::address.eq_any(wallets_to_follow))
            .order(wallet_totals::followers.desc())
            .limit(limit.try_into()?)
            .offset(offset.try_into()?)
            .into_boxed();

        if let Some(wallet) = wallet {
            let following_query = graph_connections::table
                .select(graph_connections::to_account)
                .filter(graph_connections::from_account.eq(wallet.clone()));

            query = query
                .filter(not(wallet_totals::address.eq(any(following_query))))
                .filter(not(wallet_totals::address.eq(wallet)));
        }

        let rows: Vec<(models::WalletTotal, Option<String>)> =
            query.load(&conn).context("Failed to load wallet totals")?;

        Ok(rows.into_iter().map(Into::into).collect())
    }

    #[graphql(
        description = "Returns events for the wallets the user is following using the graph_program.",
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
        exclude_types: Option<Vec<String>>,
    ) -> FieldResult<Vec<FeedEvent>> {
        let conn = ctx.shared.db.get().context("failed to connect to db")?;

        let exclude_types_parsed: Option<Vec<EventType>> = exclude_types.map(|v_types| {
            v_types
                .iter()
                .map(|v| EventType::from_str(v))
                .filter(|v| !v.is_err())
                .map(Result::unwrap)
                .collect()
        });

        let feed_events = queries::feed_event::list(
            &conn,
            wallet,
            limit.try_into()?,
            offset.try_into()?,
            exclude_types_parsed,
        )?;

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
        auction_houses(description = "List of auction houses"),
        creators(description = "Optional list of creators"),
        start_date(description = "Start date for which we want to get the average price"),
        end_date(description = "End date for which we want to get the average price")
    ))]
    pub async fn charts(
        &self,
        _context: &AppContext,
        auction_houses: Vec<PublicKey<AuctionHouse>>,
        creators: Option<Vec<PublicKey<Creator>>>,
        start_date: DateTime<Utc>,
        end_date: DateTime<Utc>,
    ) -> FieldResult<PriceChart> {
        Ok(PriceChart {
            auction_houses,
            creators,
            start_date,
            end_date,
        })
    }

    #[graphql(arguments(
        auction_housese(description = "List of auction houses"),
        creators(description = "Optional list of creators"),
    ))]
    pub async fn activities(
        &self,
        context: &AppContext,
        auction_houses: Vec<PublicKey<AuctionHouse>>,
        creators: Option<Vec<PublicKey<Creator>>>,
    ) -> FieldResult<Vec<NftActivity>> {
        let conn = context.shared.db.get()?;
        let rows = queries::activities::list(&conn, auction_houses, creators)?;

        rows.into_iter()
            .map(TryInto::try_into)
            .collect::<Result<_, _>>()
            .map_err(Into::into)
    }

    async fn profile(
        &self,
        ctx: &AppContext,
        #[graphql(description = "Twitter handle")] handle: String,
    ) -> FieldResult<Option<TwitterProfile>> {
        ctx.twitter_profile_loader
            .load(handle)
            .await
            .map_err(Into::into)
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
            .filter(bid_receipts::address.eq(address))
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
        #[graphql(description = "Filter only listed nfts")] listed: Option<bool>,
        #[graphql(description = "Filter nfts associated to the list of auction houses")]
        auction_houses: Option<Vec<PublicKey<AuctionHouse>>>,
        #[graphql(description = "Filter on a collection")] collection: Option<PublicKey<Nft>>,
        #[graphql(description = "Limit for query")] limit: i32,
        #[graphql(description = "Offset for query")] offset: i32,
    ) -> FieldResult<Vec<Nft>> {
        if collection.is_none()
            && owners.is_none()
            && creators.is_none()
            && auction_houses.is_none()
            && offerers.is_none()
        {
            return Err(FieldError::new(
                "No filter provided! Please provide at least one of the filters",
                graphql_value!({ "Filters": "owners: Vec<PublicKey>, creators: Vec<PublicKey>, offerers: Vec<PublicKey>, auction_houses: Vec<PublicKey>" }),
            ));
        }

        let conn = context.shared.db.get().context("failed to connect to db")?;

        let query_options = queries::metadatas::ListQueryOptions {
            owners: owners.map(|a| a.into_iter().map(Into::into).collect()),
            creators: creators.map(|a| a.into_iter().map(Into::into).collect()),
            offerers: offerers.map(|a| a.into_iter().map(Into::into).collect()),
            attributes: attributes.map(|a| a.into_iter().map(Into::into).collect()),
            listed,
            auction_houses: auction_houses.map(|a| a.into_iter().map(Into::into).collect()),
            collection: collection.map(Into::into),
            limit: limit.try_into()?,
            offset: offset.try_into()?,
        };
        let nfts = queries::metadatas::list(&conn, query_options)?;

        nfts.into_iter()
            .map(TryInto::try_into)
            .collect::<Result<_, _>>()
            .map_err(Into::into)
    }

    fn featured_listings(
        &self,
        context: &AppContext,
        #[graphql(description = "Limit for query")] limit: i32,
        #[graphql(description = "Offset for query")] offset: i32,
    ) -> FieldResult<Vec<ListingReceipt>> {
        let conn = context.shared.db.get().context("failed to connect to db")?;

        let holaplex_auction_house_address = "9SvsTjqk3YoicaYnC4VW1f8QAN9ku7QCCk6AyfUdzc9t";
        // TODO expand with other wallets to exclude
        let sellers_to_exclude = vec!["7r8oBPs3vNqgqEG8gnyPWUPgWuScxXyUxtmoLd1bg17F"];

        let listings: Vec<models::ListingReceipt> = listing_receipts::table
            .inner_join(
                wallet_totals::table.on(wallet_totals::address.eq(listing_receipts::seller)),
            )
            .select(listing_receipts::all_columns)
            .filter(listing_receipts::canceled_at.is_null())
            .filter(listing_receipts::purchase_receipt.is_null())
            .filter(listing_receipts::auction_house.eq(holaplex_auction_house_address))
            .filter(listing_receipts::seller.ne_all(sellers_to_exclude))
            .filter(listing_receipts::seller.eq(wallet_totals::address))
            .order(wallet_totals::followers.desc())
            .limit(limit.try_into()?)
            .offset(offset.try_into()?)
            .load(&conn)
            .context("Failed to load listings")?;

        listings
            .into_iter()
            .map(TryInto::try_into)
            .collect::<Result<_, _>>()
            .map_err(Into::into)
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
            .select(queries::metadatas::NftColumns::default())
            .limit(1)
            .load(&conn)
            .context("Failed to load metadata")?;

        rows.pop()
            .map(TryInto::try_into)
            .transpose()
            .map_err(Into::into)
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

    #[graphql(description = "returns metadata_jsons matching the term")]
    async fn metadata_jsons(
        &self,
        context: &AppContext,
        #[graphql(description = "Search term")] term: String,
        #[graphql(description = "Query limit")] limit: i32,
        #[graphql(description = "Query offset")] offset: i32,
    ) -> FieldResult<Vec<MetadataJson>> {
        let search = &context.shared.search;

        let query_result = search
            .index("metadatas")
            .search()
            .with_query(&term)
            .with_offset(offset.try_into()?)
            .with_limit(limit.try_into()?)
            .execute::<Value>()
            .await
            .context("failed to load search result for metadata json")?
            .hits;

        Ok(query_result
            .into_iter()
            .map(|r| r.result.into())
            .collect::<Vec<MetadataJson>>())
    }

    #[graphql(description = "returns profiles matching the search term")]
    async fn profiles(
        &self,
        context: &AppContext,
        #[graphql(description = "Search term")] term: String,
        #[graphql(description = "Query limit")] limit: i32,
        #[graphql(description = "Query offset")] offset: i32,
    ) -> FieldResult<Vec<Wallet>> {
        let search = &context.shared.search;

        let query_result = search
            .index("name_service")
            .search()
            .with_query(&term)
            .with_offset(offset.try_into()?)
            .with_limit(limit.try_into()?)
            .execute::<Value>()
            .await
            .context("failed to load search result for twitter handle")?
            .hits;

        Ok(query_result
            .into_iter()
            .map(|r| r.result.into())
            .collect::<Vec<Wallet>>())
    }

    fn denylist() -> Denylist {
        Denylist
    }
}
