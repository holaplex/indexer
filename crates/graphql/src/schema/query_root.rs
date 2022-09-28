use indexer_core::db::{
    self,
    expression::dsl::all,
    queries::{self, collections::TrendingQueryOptions, feed_event::EventType},
};
use objects::{
    ah_listing::AhListing,
    auction_house::AuctionHouse,
    bid_receipt::BidReceipt,
    bonding_change::EnrichedBondingChange,
    candy_machine::CandyMachine,
    chart::PriceChart,
    creator::Creator,
    denylist::Denylist,
    feed_event::FeedEvent,
    genopets::{GenoHabitat, GenoHabitatList, GenoHabitatsParams},
    graph_connection::GraphConnection,
    listing::{Listing, ListingColumns, ListingRow},
    marketplace::Marketplace,
    nft::{Collection, MetadataJson, Nft, NftActivity, NftCount, NftCreator, NftsStats},
    profile::{ProfilesStats, TwitterProfile},
    spl_governance::{
        Governance, Proposal, ProposalV2, Realm, SignatoryRecord, TokenOwnerRecord, VoteRecord,
    },
    storefront::{Storefront, StorefrontColumns},
    wallet::Wallet,
};
use scalars::{markers::TokenMint, PublicKey};
use serde_json::Value;
use tables::{
    auction_caches, auction_datas, auction_datas_ext, auction_houses, bid_receipts,
    candy_machine_datas, candy_machines, current_metadata_owners, geno_habitat_datas, governances,
    graph_connections, metadata_jsons, metadatas, realms, signatory_records, store_config_jsons,
    storefronts, token_owner_records, twitter_handle_name_services, wallet_totals,
};

use super::{
    enums::{CollectionInterval, CollectionSort, OrderDirection},
    objects::nft::CollectionTrend,
    prelude::*,
};
pub struct QueryRoot;

#[derive(GraphQLInputObject, Clone, Debug)]
#[graphql(description = "Filter on NFT attributes")]
pub struct AttributeFilter {
    trait_type: String,
    values: Vec<String>,
}

impl From<AttributeFilter> for queries::metadatas::AttributeFilter {
    fn from(AttributeFilter { trait_type, values }: AttributeFilter) -> Self {
        Self { trait_type, values }
    }
}

impl QueryRoot {
    fn candy_machine(context: &AppContext, address: String) -> FieldResult<Option<CandyMachine>> {
        let conn = context.shared.db.get()?;

        candy_machines::table
            .inner_join(
                candy_machine_datas::table
                    .on(candy_machines::address.eq(candy_machine_datas::candy_machine_address)),
            )
            .filter(candy_machines::address.eq(address))
            .select((
                candy_machines::all_columns,
                candy_machine_datas::all_columns,
            ))
            .first::<(models::CandyMachine, models::CandyMachineData)>(&conn)
            .optional()
            .context("Failed to load candy machine by address.")?
            .map(TryInto::try_into)
            .transpose()
            .map_err(Into::into)
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

    #[graphql(arguments(creators(description = "creators of nfts")))]
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
        #[graphql(description = "Filter only listed NFTs")] listed: Option<bool>,
        #[graphql(description = "Allow unverified NFTs")] allow_unverified: Option<bool>,
        #[graphql(
            description = "Filter only NFTs with active offers; rejected if flag is 'false'"
        )]
        with_offers: Option<bool>,
        #[graphql(description = "Filter NFTs associated to the list of auction houses")]
        auction_houses: Option<Vec<PublicKey<AuctionHouse>>>,
        #[deprecated = "Deprecated in favor of the collections argument"] collection: Option<
            PublicKey<Nft>,
        >,
        #[graphql(description = "Filter on one or more collections")] collections: Option<
            Vec<PublicKey<Nft>>,
        >,
        #[graphql(
            description = "Return NFTs whose metadata contain this search term (case-insensitive)"
        )]
        term: Option<String>,
        #[graphql(description = "Limit for query")] limit: i32,
        #[graphql(description = "Offset for query")] offset: i32,
    ) -> FieldResult<Vec<Nft>> {
        let collections = match (collections, collection) {
            (c, None) => c,
            (None, Some(c)) => Some(vec![c]),
            (Some(_), Some(_)) => {
                return Err(FieldError::new(
                    "The collection argument is deprecated and cannot be combined with the \
                    collections argument",
                    graphql_value!(None),
                ));
            },
        };

        if collections.is_none()
            && owners.is_none()
            && creators.is_none()
            && auction_houses.is_none()
            && offerers.is_none()
            && term.is_none()
            && update_authorities.is_none()
        {
            return Err(FieldError::new(
                "No filter provided! Please provide at least one of the following arguments",
                graphql_value!([
                    "collections",
                    "owners",
                    "creators",
                    "auction_houses",
                    "offerers",
                    "term",
                    "update_authorities"
                ]),
            ));
        }

        if let Some(false) = with_offers {
            return Err(FieldError::new(
                "with_offers == false is not currently supported",
                graphql_value!({ "invalid_parameter": "with_offers" }),
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
                    .with_limit(context.shared.pre_query_search_limit)
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
            owners: owners.map(|o| o.into_iter().map(Into::into).collect()),
            creators: creators.map(|c| c.into_iter().map(Into::into).collect()),
            update_authorities: update_authorities.map(|a| a.into_iter().map(Into::into).collect()),
            offerers: offerers.map(|o| o.into_iter().map(Into::into).collect()),
            attributes: attributes.map(|a| a.into_iter().map(Into::into).collect()),
            listed,
            allow_unverified,
            with_offers,
            auction_houses: auction_houses.map(|h| h.into_iter().map(Into::into).collect()),
            collections: collections.map(|c| c.into_iter().map(Into::into).collect()),
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
    ) -> FieldResult<Vec<AhListing>> {
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

    async fn wallets(
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

        futures_util::future::try_join_all(addresses.into_iter().map(|a| context.wallet(a)))
            .await
            .map_err(Into::into)
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
            .inner_join(
                current_metadata_owners::table
                    .on(current_metadata_owners::mint_address.eq(metadatas::mint_address)),
            )
            .filter(metadatas::address.eq(address))
            .select(queries::metadatas::NFT_COLUMNS)
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
            .inner_join(
                current_metadata_owners::table
                    .on(current_metadata_owners::mint_address.eq(metadatas::mint_address)),
            )
            .filter(metadatas::mint_address.eq(address))
            .select(queries::metadatas::NFT_COLUMNS)
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
            .inner_join(
                current_metadata_owners::table
                    .on(current_metadata_owners::mint_address.eq(metadatas::mint_address)),
            )
            .filter(metadatas::mint_address.eq(any(addresses)))
            .select(queries::metadatas::NFT_COLUMNS)
            .load(&conn)
            .context("Failed to load NFTs")?;

        rows.into_iter()
            .map(Nft::try_from)
            .collect::<Result<_, _>>()
            .map_err(Into::into)
    }

    #[graphql(deprecated = "Deprecated alias for candyMachine")]
    fn candymachine(&self, ctx: &AppContext, addr: String) -> FieldResult<Option<CandyMachine>> {
        Self::candy_machine(ctx, addr)
    }

    #[graphql(description = "Get a candy machine by the candy machine config address")]
    fn candy_machine(
        &self,
        context: &AppContext,
        #[graphql(description = "address of the candy machine config")] address: String,
    ) -> FieldResult<Option<CandyMachine>> {
        Self::candy_machine(context, address)
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

    #[graphql(
        description = "Returns collection data along with collection activities",
        arguments(address(description = "Collection address"))
    )]
    async fn collection(
        &self,
        context: &AppContext,
        address: String,
    ) -> FieldResult<Option<Collection>> {
        context.generic_collection_loader
            .load(address)
            .await
            .map_err(Into::into)
    }

    #[graphql(
        description = "Returns featured collection NFTs ordered by market cap (floor price * number of NFTs in collection)",
        arguments(
            sort_by(description = "Choose sort for trending collections"),
            time_frame(description = "The desired timeframe to evaluate the trending collection"),
            order_direction(
                description = "Arrange result in ascending or descending order by selected sort_by"
            ),
            limit(description = "Return at most this many results"),
            offset(description = "Return results starting from this index"),
        )
    )]
    async fn collection_trends(
        &self,
        context: &AppContext,
        sort_by: CollectionSort,
        time_frame: CollectionInterval,
        order_direction: Option<OrderDirection>,
        limit: i32,
        offset: i32,
    ) -> FieldResult<Vec<CollectionTrend>> {
        let conn = context.shared.db.get().context("failed to connect to db")?;

        let sort = match (time_frame, sort_by) {
            (CollectionInterval::One, CollectionSort::Volume) => {
                db::custom_types::CollectionSort::OneDayVolume
            },
            (CollectionInterval::Seven, CollectionSort::Volume) => {
                db::custom_types::CollectionSort::SevenDayVolume
            },
            (CollectionInterval::Thirty, CollectionSort::Volume) => {
                db::custom_types::CollectionSort::ThirtyDayVolume
            },
            (CollectionInterval::One, CollectionSort::NumberSales) => {
                db::custom_types::CollectionSort::OneDaySalesCount
            },
            (CollectionInterval::Seven, CollectionSort::NumberSales) => {
                db::custom_types::CollectionSort::SevenDaySalesCount
            },
            (CollectionInterval::Thirty, CollectionSort::NumberSales) => {
                db::custom_types::CollectionSort::ThirtyDaySalesCount
            },
            (CollectionInterval::One, CollectionSort::Marketcap) => {
                db::custom_types::CollectionSort::OneDayMarketcap
            },
            (CollectionInterval::Seven, CollectionSort::Marketcap) => {
                db::custom_types::CollectionSort::SevenDayMarketcap
            },
            (CollectionInterval::Thirty, CollectionSort::Marketcap) => {
                db::custom_types::CollectionSort::ThirtyDayMarketcap
            },
            (
                CollectionInterval::One | CollectionInterval::Seven | CollectionInterval::Thirty,
                CollectionSort::Floor,
            ) => db::custom_types::CollectionSort::FloorPrice,
        };

        let collections = queries::collections::trends(&conn, TrendingQueryOptions {
            sort_by: sort,
            order: order_direction.map(Into::into),
            limit: limit.try_into()?,
            offset: offset.try_into()?,
        })?;

        collections
            .into_iter()
            .map(TryInto::try_into)
            .collect::<Result<_, _>>()
            .map_err(Into::into)
    }

    #[graphql(
        description = "Returns featured collection NFTs ordered by market cap (floor price * number of NFTs in collection)",
        arguments(
            term(
                description = "Return collections whose metadata match this term (case insensitive); sorting occurs among limited search results (rather than searching after sorting)"
            ),
            order_direction(
                description = "Choose (and sort) ascending or descending by market cap"
            ),
            start_date(
                description = "Compute market cap over NFTs listed later than this date (ISO 8601 format like 2022-07-04T17:06:10Z)"
            ),
            end_date(
                description = "Compute market cap over NFTs listed earlier than this date (ISO 8601 format like 2022-07-04T17:06:10Z)"
            ),
            limit(description = "Return at most this many results"),
            offset(description = "Return results starting from this index"),
        )
    )]
    async fn collections_featured_by_market_cap(
        &self,
        context: &AppContext,
        term: Option<String>,
        order_direction: OrderDirection,
        start_date: DateTime<Utc>,
        end_date: DateTime<Utc>,
        limit: i32,
        offset: i32,
    ) -> FieldResult<Vec<Collection>> {
        let conn = context.shared.db.get().context("failed to connect to db")?;

        let addresses: Option<Vec<String>> = match term {
            Some(term) => {
                let search = &context.shared.search;
                let search_result = search
                    .index("collections")
                    .search()
                    .with_query(&term)
                    .with_limit(context.shared.pre_query_search_limit)
                    .execute::<Value>()
                    .await
                    .context("failed to load search result for collections")?
                    .hits;

                Some(
                    search_result
                        .into_iter()
                        .map(|r| MetadataJson::from(r.result).mint_address)
                        .collect(),
                )
            },
            None => None,
        };

        let collections = queries::collections::by_market_cap(
            &conn,
            addresses,
            order_direction.into(),
            start_date,
            end_date,
            limit,
            offset,
        )?;

        collections
            .into_iter()
            .map(TryInto::try_into)
            .collect::<Result<_, _>>()
            .map_err(Into::into)
    }

    #[graphql(
        description = "Returns featured collection NFTs ordered by volume (sum of purchase prices)",
        arguments(
            term(
                description = "Return collections whose metadata match this term (case insensitive); sorting occurs among limited search results (rather than searching after sorting)"
            ),
            order_direction(description = "Choose (and sort) ascending or descending by volume"),
            start_date(
                description = "Compute volume over sales starting from this date (ISO 8601 format like 2022-07-04T17:06:10Z)"
            ),
            end_date(
                description = "Compute volume over sales ending at this date (ISO 8601 format like 2022-07-04T17:06:10Z)"
            ),
            limit(description = "Return at most this many results"),
            offset(description = "Return results starting from this index"),
        )
    )]
    async fn collections_featured_by_volume(
        &self,
        context: &AppContext,
        term: Option<String>,
        order_direction: OrderDirection,
        start_date: DateTime<Utc>,
        end_date: DateTime<Utc>,
        limit: i32,
        offset: i32,
    ) -> FieldResult<Vec<Collection>> {
        let conn = context.shared.db.get().context("failed to connect to db")?;

        let addresses: Option<Vec<String>> = match term {
            Some(term) => {
                let search = &context.shared.search;
                let search_result = search
                    .index("collections")
                    .search()
                    .with_query(&term)
                    .with_limit(context.shared.pre_query_search_limit)
                    .execute::<Value>()
                    .await
                    .context("failed to load search result for collections")?
                    .hits;

                Some(
                    search_result
                        .into_iter()
                        .map(|r| MetadataJson::from(r.result).mint_address)
                        .collect(),
                )
            },
            None => None,
        };

        let collections = queries::collections::by_volume(
            &conn,
            addresses,
            order_direction.into(),
            start_date,
            end_date,
            limit,
            offset,
        )?;

        collections
            .into_iter()
            .map(TryInto::try_into)
            .collect::<Result<_, _>>()
            .map_err(Into::into)
    }

    #[graphql(description = "returns all the collections matching the search term")]
    async fn search_collections(
        &self,
        context: &AppContext,
        #[graphql(description = "Search term")] term: String,
        #[graphql(description = "Query limit")] limit: i32,
        #[graphql(description = "Query offset")] offset: i32,
    ) -> FieldResult<Vec<MetadataJson>> {
        let search = &context.shared.search;

        let query_result = search
            .index("collections")
            .search()
            .with_query(&term)
            .with_offset(offset.try_into()?)
            .with_limit(limit.try_into()?)
            .execute::<Value>()
            .await
            .context("failed to load search result for collections")?
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

    fn auction_house(
        &self,
        context: &AppContext,
        #[graphql(description = "AuctionHouse Address")] address: String,
    ) -> FieldResult<Option<AuctionHouse>> {
        let conn = context.shared.db.get()?;
        auction_houses::table
            .filter(auction_houses::address.eq(address))
            .first::<models::AuctionHouse>(&conn)
            .optional()
            .context("Failed to load AuctionHouse by address.")?
            .map(TryInto::try_into)
            .transpose()
            .map_err(Into::into)
    }

    #[graphql(
        description = "Query up to one Genopets habitat by the public key of its on-chain data",
        arguments(
            address(description = "Select a habitat by its data account address"),
            mint(description = "Select a habitat by its by mint address"),
        )
    )]
    fn geno_habitat(
        &self,
        context: &AppContext,
        address: Option<PublicKey<GenoHabitat>>,
        mint: Option<PublicKey<TokenMint>>,
    ) -> FieldResult<Option<GenoHabitat>> {
        let conn = context.shared.db.get()?;

        let row = match (address, mint) {
            (None, None) | (Some(_), Some(_)) => {
                return Err(FieldError::new(
                    "Exactly one parameter must be specified",
                    graphql_value!(["address", "mint"]),
                ));
            },
            (Some(address), None) => geno_habitat_datas::table
                .filter(geno_habitat_datas::address.eq(address))
                .first::<models::GenoHabitatData>(&conn),
            (None, Some(mint)) => geno_habitat_datas::table
                .filter(geno_habitat_datas::habitat_mint.eq(mint))
                .first::<models::GenoHabitatData>(&conn),
        }
        .optional()
        .context("Failed to load Genopets habitat")?;

        Ok(row.map(Into::into))
    }

    #[graphql(deprecated = "Use genoHabitatsCounted instead")]
    async fn geno_habitats(
        &self,
        ctx: &AppContext,
        mints: Option<Vec<PublicKey<TokenMint>>>,
        owners: Option<Vec<PublicKey<Wallet>>>,
        renters: Option<Vec<PublicKey<Wallet>>>,
        harvesters: Option<Vec<String>>,
        genesis: Option<bool>,
        elements: Option<Vec<i32>>,
        min_level: Option<i32>,
        max_level: Option<i32>,
        min_sequence: Option<i32>,
        max_sequence: Option<i32>,
        guilds: Option<Vec<i32>>,
        min_durability: Option<i32>,
        max_durability: Option<i32>,
        min_expiry: Option<DateTime<Utc>>,
        max_expiry: Option<DateTime<Utc>>,
        harvester_open_market: Option<bool>,
        rental_open_market: Option<bool>,
        has_alchemist: Option<bool>,
        has_harvester: Option<bool>,
        has_max_ki: Option<bool>,
        is_activated: Option<bool>,
        term: Option<String>,
        limit: i32,
        offset: i32,
    ) -> FieldResult<Vec<GenoHabitat>> {
        let conn = ctx.shared.db.get().context("Failed to connect to the DB")?;

        let opts = GenoHabitatsParams {
            mints,
            owners,
            renters,
            harvesters,
            genesis,
            elements,
            min_level,
            max_level,
            min_sequence,
            max_sequence,
            guilds,
            min_durability,
            max_durability,
            min_expiry,
            max_expiry,
            harvester_open_market,
            rental_open_market,
            has_alchemist,
            has_harvester,
            has_max_ki,
            is_activated,
            term,
            sort_field: None,
            sort_desc: None,
            limit,
            offset,
        }
        .into_db_opts(ctx)
        .await?;

        queries::genopets::list_habitats(&conn, opts)
            .map(|(l, _)| l.into_iter().map(Into::into).collect())
            .map_err(Into::into)
    }

    #[graphql(description = "Query zero or more Genopets habitats")]
    async fn geno_habitats_counted(
        &self,
        ctx: &AppContext,
        params: GenoHabitatsParams,
    ) -> FieldResult<GenoHabitatList> {
        let conn = ctx.shared.db.get().context("Failed to connect to the DB")?;
        let opts = params.into_db_opts(ctx).await?;

        queries::genopets::list_habitats(&conn, opts)
            .map(Into::into)
            .map_err(Into::into)
    }

    fn token_owner_records(
        &self,
        context: &AppContext,
        #[graphql(description = "Filter on Realms TokenOwnerRecords")] addresses: Option<
            Vec<PublicKey<TokenOwnerRecord>>,
        >,
        #[graphql(description = "Filter on Realms")] realms: Option<Vec<PublicKey<Realm>>>,
        #[graphql(description = "Filter on Governing Token mints")] governing_token_mints: Option<
            Vec<PublicKey<TokenMint>>,
        >,
    ) -> FieldResult<Vec<TokenOwnerRecord>> {
        if addresses.is_none() && realms.is_none() && governing_token_mints.is_none() {
            return Err(FieldError::new(
                "You must supply atleast one filter",
                graphql_value!({ "Filters": "addresses: Vec<PublicKey<TokenOwnerRecord>>, realms: Vec<PublicKey<Realm>>, governing_token_mints: Vec<PublicKey<TokenMint>>" }),
            ));
        }

        let conn = context.shared.db.get()?;
        let mut query = token_owner_records::table
            .select(token_owner_records::all_columns)
            .into_boxed();

        if let Some(addresses) = addresses {
            query = query.filter(token_owner_records::address.eq(any(addresses)));
        }

        if let Some(realms) = realms {
            query = query.filter(token_owner_records::realm.eq(any(realms)));
        }

        if let Some(mints) = governing_token_mints {
            query = query.filter(token_owner_records::governing_token_mint.eq(any(mints)));
        }

        query
            .load::<models::TokenOwnerRecord>(&conn)
            .context("Failed to load spl token owner records.")?
            .into_iter()
            .map(TokenOwnerRecord::try_from)
            .collect::<Result<_, _>>()
            .map_err(Into::into)
    }

    fn governances(
        &self,
        context: &AppContext,
        #[graphql(description = "Filter on SPL Governances")] addresses: Option<
            Vec<PublicKey<Governance>>,
        >,
        #[graphql(description = "Filter on Realms")] realms: Option<Vec<PublicKey<Realm>>>,
    ) -> FieldResult<Vec<Governance>> {
        if addresses.is_none() && realms.is_none() {
            return Err(FieldError::new(
                "You must supply atleast one filter",
                graphql_value!({ "Filters": "addresses: Vec<PublicKey<Governance>>, realms: Vec<PublicKey<Realm>>" }),
            ));
        }

        let conn = context.shared.db.get()?;
        let mut query = governances::table
            .select(governances::all_columns)
            .into_boxed();

        if let Some(addresses) = addresses {
            query = query.filter(governances::address.eq(any(addresses)));
        }
        if let Some(realms) = realms {
            query = query.filter(governances::realm.eq(any(realms)));
        }

        query
            .load::<models::Governance>(&conn)
            .context("Failed to load spl governances.")?
            .into_iter()
            .map(Governance::try_from)
            .collect::<Result<_, _>>()
            .map_err(Into::into)
    }

    fn proposals(
        &self,
        context: &AppContext,
        #[graphql(description = "Filter on SPL Governance proposals")] addresses: Option<
            Vec<PublicKey<Proposal>>,
        >,
        #[graphql(description = "Filter on spl governance")] governances: Option<
            Vec<PublicKey<Governance>>,
        >,
    ) -> FieldResult<Vec<Proposal>> {
        if addresses.is_none() && governances.is_none() {
            return Err(FieldError::new(
                "You must supply atleast one filter",
                graphql_value!({ "Filters": "addresses: Vec<PublicKey<Proposal>>, governances: Vec<PublicKey<Governance>>" }),
            ));
        }

        let conn = context.shared.db.get()?;

        let proposals: Vec<models::SplGovernanceProposal> =
            queries::spl_governance::proposals(&conn, addresses, governances)?;

        proposals
            .into_iter()
            .map(Proposal::try_from)
            .collect::<Result<_, _>>()
            .map_err(Into::into)
    }

    fn vote_records(
        &self,
        context: &AppContext,
        #[graphql(description = "Filter on SPL VoteRecordV2 pubkeys")] addresses: Option<
            Vec<PublicKey<VoteRecord>>,
        >,
        #[graphql(description = "Filter on Proposals")] proposals: Option<Vec<PublicKey<Proposal>>>,
        #[graphql(description = "Filter on GoverningTokenOwners")] governing_token_owners: Option<
            Vec<PublicKey<Wallet>>,
        >,
        #[graphql(description = "Filter on is_relinquished")] is_relinquished: Option<bool>,
    ) -> FieldResult<Vec<VoteRecord>> {
        if addresses.is_none()
            && proposals.is_none()
            && governing_token_owners.is_none()
            && is_relinquished.is_none()
        {
            return Err(FieldError::new(
                "You must supply atleast one filter",
                graphql_value!({ "Filters": "addresses: Vec<PublicKey<VoteRecordV2>>, proposals: Vec<PublicKey<Proposal>>, governing_token_owners: Vec<PublicKey<Wallet>>, is_relinquished: bool" }),
            ));
        }

        let conn = context.shared.db.get()?;

        let vote_records: Vec<models::VoteRecord> = queries::spl_governance::vote_records(
            &conn,
            addresses,
            proposals,
            governing_token_owners,
            is_relinquished,
        )?;

        vote_records
            .into_iter()
            .map(VoteRecord::try_from)
            .collect::<Result<_, _>>()
            .map_err(Into::into)
    }

    fn signatory_records(
        &self,
        context: &AppContext,
        #[graphql(description = "Filter on SPL SignatoryRecord pubkeys")] addresses: Option<
            Vec<PublicKey<SignatoryRecord>>,
        >,
        #[graphql(description = "Filter on Proposals")] proposals: Option<
            Vec<PublicKey<ProposalV2>>,
        >,
    ) -> FieldResult<Vec<SignatoryRecord>> {
        if addresses.is_none() && proposals.is_none() {
            return Err(FieldError::new(
                "You must supply atleast one filter",
                graphql_value!({ "Filters": "addresses: Vec<PublicKey<SignatoryRecord>>, proposals: Vec<PublicKey<Proposal>>" }),
            ));
        }

        let conn = context.shared.db.get()?;

        let mut query = signatory_records::table
            .select(signatory_records::all_columns)
            .into_boxed();

        if let Some(addresses) = addresses {
            query = query.filter(signatory_records::address.eq(any(addresses)));
        }

        if let Some(proposals) = proposals {
            query = query.filter(signatory_records::proposal.eq(any(proposals)));
        }

        query
            .load::<models::SignatoryRecord>(&conn)
            .context("Failed to load spl governance signatory records.")?
            .into_iter()
            .map(SignatoryRecord::try_from)
            .collect::<Result<_, _>>()
            .map_err(Into::into)
    }

    fn realms(
        &self,
        context: &AppContext,
        #[graphql(description = "Filter on SPL Realm pubkeys")] addresses: Option<
            Vec<PublicKey<Realm>>,
        >,
        #[graphql(description = "Filter on Community mints")] community_mints: Option<
            Vec<PublicKey<TokenMint>>,
        >,
    ) -> FieldResult<Vec<Realm>> {
        if addresses.is_none() && community_mints.is_none() {
            return Err(FieldError::new(
                "You must supply atleast one filter",
                graphql_value!({ "Filters": "addresses: Vec<PublicKey<Realm>>, communityMints: Vec<PublicKey<TokenMint>>" }),
            ));
        }

        let conn = context.shared.db.get()?;

        let mut query = realms::table.select(realms::all_columns).into_boxed();

        if let Some(addresses) = addresses {
            query = query.filter(realms::address.eq(any(addresses)));
        }

        if let Some(community_mints) = community_mints {
            query = query.filter(realms::community_mint.eq(any(community_mints)));
        }

        query
            .load::<models::Realm>(&conn)
            .context("Failed to load spl governance realms.")?
            .into_iter()
            .map(Realm::try_from)
            .collect::<Result<_, _>>()
            .map_err(Into::into)
    }

    fn denylist() -> Denylist {
        Denylist
    }
}
