use indexer_core::db::{
    expression::dsl::all,
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
    nft::{MetadataJson, Nft, NftActivity, NftCount, NftCreator, NftsStats},
    profile::{ProfilesStats, TwitterProfile},
    storefront::{Storefront, StorefrontColumns},
    wallet::Wallet,
};
use scalars::PublicKey;
use serde_json::Value;
use tables::{
    auction_caches, auction_datas, auction_datas_ext, bid_receipts, graph_connections,
    metadata_jsons, metadatas, store_config_jsons, storefronts, wallet_totals,
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

        let mut query = wallet_totals::table
            .left_join(
                twitter_handle_name_services::table
                    .on(wallet_totals::address.eq(twitter_handle_name_services::wallet_address)),
            )
            .select((
                (wallet_totals::all_columns),
                twitter_handle_name_services::twitter_handle.nullable(),
            ))
            .filter(wallet_totals::address.ne(all(&ctx.shared.follow_wallets_exclusions)))
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
                .map(|v| v.parse::<EventType>())
                .filter_map(Result::ok)
                .collect()
        });

        let feed_events = queries::feed_event::list(
            &conn,
            limit.try_into()?,
            offset.try_into()?,
            Some(wallet.to_string()),
            exclude_types_parsed,
        )?;

        feed_events
            .into_iter()
            .map(TryInto::try_into)
            .collect::<Result<_, _>>()
            .map_err(Into::into)
    }

    #[graphql(
        description = "Returns the latest on chain events using the graph_program.",
        arguments(
            limit(description = "The query record limit"),
            is_forward(description = "Data record needed forward or backward"),
            cursor(description = "The query record offset")
        )
    )]
    fn latest_feed_events(
        &self,
        ctx: &AppContext,
        limit: i32,
        is_forward: bool,
        cursor: String,
        include_types: Option<Vec<String>>,
    ) -> FieldResult<Vec<FeedEvent>> {
        let conn = ctx.shared.db.get().context("failed to connect to db")?;

        let include_types_parsed: Option<Vec<EventType>> = include_types.map(|v_types| {
            v_types
                .iter()
                .map(|v| v.parse::<EventType>())
                .filter_map(Result::ok)
                .collect()
        });

        let feed_events = queries::feed_event::list_relay(
            &conn,
            limit.try_into()?,
            is_forward,
            cursor,
            None,
            include_types_parsed,
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

    async fn nfts(
        &self,
        context: &AppContext,
        #[graphql(description = "Filter on owner address")] owners: Option<Vec<PublicKey<Wallet>>>,
        #[graphql(description = "Filter on creator address")] creators: Option<
            Vec<PublicKey<Wallet>>,
        >,
        #[graphql(description = "Filter on update authorities")] update_authorities: Option<
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
        #[graphql(
            description = "Return NFTs whose metadata contain this search term (case-insensitive)"
        )]
        term: Option<String>,
        #[graphql(description = "Limit for query")] limit: i32,
        #[graphql(description = "Offset for query")] offset: i32,
    ) -> FieldResult<Vec<Nft>> {
        if collection.is_none()
            && owners.is_none()
            && creators.is_none()
            && auction_houses.is_none()
            && offerers.is_none()
            && term.is_none()
        {
            return Err(FieldError::new(
                "No filter provided! Please provide at least one of the filters",
                graphql_value!({ "Filters": "owners: Vec<PublicKey>, creators: Vec<PublicKey>, offerers: Vec<PublicKey>, auction_houses: Vec<PublicKey>, term: String" }),
            ));
        }

        let conn = context.shared.db.get().context("failed to connect to db")?;

        let addresses = match term {
            Some(term) => {
                let search = &context.shared.search;
                let search_result = search
                    .index("metadatas")
                    .search()
                    .with_query(&term)
                    .with_offset(offset.try_into()?)
                    .with_limit(limit.try_into()?)
                    .execute::<Value>()
                    .await
                    .context("failed to load search result for metadata json")?
                    .hits;

                Some(
                    search_result
                        .into_iter()
                        .map(|r| MetadataJson::from(r.result).address)
                        .collect(),
                )
            },
            None => None,
        };

        let query_options = queries::metadatas::ListQueryOptions {
            addresses,
            owners: owners.map(|a| a.into_iter().map(Into::into).collect()),
            creators: creators.map(|a| a.into_iter().map(Into::into).collect()),
            update_authorities: update_authorities.map(|a| a.into_iter().map(Into::into).collect()),
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

    #[graphql(description = "Stats aggregated across all indexed NFTs")]
    fn nfts_stats(&self) -> NftsStats {
        NftsStats
    }

    fn featured_listings(
        &self,
        context: &AppContext,
        #[graphql(description = "Return listings only from these auction houses")]
        auction_houses: Option<Vec<PublicKey<AuctionHouse>>>,
        #[graphql(description = "Return listings not from these sellers")]
        seller_exclusions: Option<Vec<PublicKey<Wallet>>>,
        #[graphql(description = "Return at most this many listings per seller")]
        limit_per_seller: Option<i32>,
        #[graphql(description = "Limit for query")] limit: i32,
        #[graphql(description = "Offset for query")] offset: Option<i32>,
    ) -> FieldResult<Vec<ListingReceipt>> {
        let conn = context.shared.db.get().context("Failed to connect to DB")?;

        let auction_houses = auction_houses.unwrap_or_else(|| {
            context
                .shared
                .featured_listings_auction_houses
                .iter()
                .map(|a| PublicKey::from(a.clone()))
                .collect()
        });
        let seller_exclusions = seller_exclusions.unwrap_or_else(|| {
            context
                .shared
                .featured_listings_seller_exclusions
                .iter()
                .map(|s| PublicKey::from(s.clone()))
                .collect()
        });
        let limit_per_seller = limit_per_seller.unwrap_or(5);
        let offset = offset.unwrap_or(0);

        // choose listings whose NFT's creators have a lot of followers
        let listings = queries::featured_listings::list(
            &conn,
            auction_houses,
            seller_exclusions,
            limit_per_seller,
            limit,
            offset,
        )?;

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

    fn wallets(
        &self,
        context: &AppContext,
        #[graphql(description = "Addresses of the wallets")] addresses: Vec<PublicKey<Wallet>>,
    ) -> FieldResult<Vec<Wallet>> {
        if addresses.is_empty() {
            return Err(FieldError::new(
                "You must supply at least one address to query.",
                graphql_value!({ "addresses": "Vec<String>"}),
            ));
        }

        let conn = context.shared.db.get()?;

        let twitter_handles = queries::twitter_handle_name_service::get_multiple(
            &conn,
            addresses.iter().map(ToString::to_string).collect(),
        )?;

        let wallets = twitter_handles.into_iter().fold(
            addresses
                .into_iter()
                .map(|a| (a, None))
                .collect::<HashMap<_, _>>(),
            |mut h,
             models::TwitterHandle {
                 wallet_address,
                 twitter_handle,
                 ..
             }| {
                *h.entry(wallet_address.into_owned().into()).or_insert(None) =
                    Some(twitter_handle.into_owned());

                h
            },
        );

        Ok(wallets
            .into_iter()
            .map(|(k, v)| Wallet::new(k, v))
            .collect())
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

    #[graphql(description = "Get an NFT by metadata address.")]
    fn nft(
        &self,
        context: &AppContext,
        #[graphql(description = "Metadata address of NFT")] address: String,
    ) -> FieldResult<Option<Nft>> {
        let conn = context.shared.db.get()?;
        metadatas::table
            .inner_join(
                metadata_jsons::table.on(metadatas::address.eq(metadata_jsons::metadata_address)),
            )
            .filter(metadatas::address.eq(address))
            .select(queries::metadatas::NftColumns::default())
            .first::<models::Nft>(&conn)
            .optional()
            .context("Failed to load NFT by metadata address.")?
            .map(TryInto::try_into)
            .transpose()
            .map_err(Into::into)
    }

    #[graphql(description = "Get an NFT by mint address.")]
    fn nft_by_mint_address(
        &self,
        context: &AppContext,
        #[graphql(description = "Mint address of NFT")] address: String,
    ) -> FieldResult<Option<Nft>> {
        let conn = context.shared.db.get()?;
        metadatas::table
            .inner_join(
                metadata_jsons::table.on(metadatas::address.eq(metadata_jsons::metadata_address)),
            )
            .filter(metadatas::mint_address.eq(address))
            .select(queries::metadatas::NftColumns::default())
            .first::<models::Nft>(&conn)
            .optional()
            .context("Failed to load NFT by mint address.")?
            .map(TryInto::try_into)
            .transpose()
            .map_err(Into::into)
    }

    #[graphql(description = "Get a list of NFTs by mint address.")]
    fn nfts_by_mint_address(
        &self,
        context: &AppContext,
        #[graphql(description = "Mint addresses of NFTs")] addresses: Vec<PublicKey<Nft>>,
    ) -> FieldResult<Vec<Nft>> {
        let conn = context.shared.db.get()?;
        let rows: Vec<models::Nft> = metadatas::table
            .inner_join(
                metadata_jsons::table.on(metadatas::address.eq(metadata_jsons::metadata_address)),
            )
            .filter(metadatas::mint_address.eq(any(addresses)))
            .select(queries::metadatas::NftColumns::default())
            .load(&conn)
            .context("Failed to load NFTs")?;

        rows.into_iter()
            .map(Nft::try_from)
            .collect::<Result<_, _>>()
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
            .filter(
                store_config_jsons::store_address
                    .ne(all(&context.shared.marketplaces_store_address_exclusions)),
            )
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

    #[graphql(description = "returns stats about profiles")]
    async fn profiles_stats(&self) -> ProfilesStats {
        ProfilesStats
    }

    #[graphql(
        description = "Get multiple marketplaces; results will be in alphabetical order by subdomain"
    )]
    fn marketplaces(
        &self,
        context: &AppContext,
        #[graphql(
            description = "Return these marketplaces; results will be in alphabetical order by subdomain."
        )]
        subdomains: Option<Vec<String>>,
        #[graphql(description = "Limit for query")] limit: Option<i32>,
        #[graphql(description = "Offset for query")] offset: Option<i32>,
    ) -> FieldResult<Vec<Marketplace>> {
        let too_many_filters = subdomains.is_some() && (limit.is_some() || offset.is_some());
        let not_enough_filters = subdomains.is_none() && limit.is_none();
        if too_many_filters || not_enough_filters {
            return Err(FieldError::new(
                "You must supply either a limit (and optionally offset) or subdomains",
                graphql_value!({ "Filters": "subdomains: Vec<String>, limit: i32, offset: i32" }),
            ));
        }

        let conn = context.shared.db.get()?;
        let mut query = store_config_jsons::table
            .select(store_config_jsons::all_columns)
            .filter(
                store_config_jsons::store_address
                    .ne(all(&context.shared.marketplaces_store_address_exclusions)),
            )
            .order(store_config_jsons::name.asc())
            .into_boxed();

        if let Some(subdomains) = subdomains {
            query = query.filter(store_config_jsons::subdomain.eq(any(subdomains)));
        } else {
            query = query
                .limit(limit.unwrap_or_else(|| unreachable!()).into())
                .offset(offset.unwrap_or(0).into());
        }

        let rows: Vec<models::StoreConfigJson> = query
            .load(&conn)
            .context("Failed to load store config JSON")?;

        Ok(rows.into_iter().map(Into::into).collect())
    }

    fn denylist() -> Denylist {
        Denylist
    }
}
