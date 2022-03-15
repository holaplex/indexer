use indexer_core::db::queries;
use objects::{
    auction_house::AuctionHouse,
    creator::Creator,
    denylist::Denylist,
    listing::{Listing, ListingColumns, ListingRow},
    marketplace::Marketplace,
    nft::Nft,
    profile::{Profile, TwitterProfilePictureResponse, TwitterShowResponse},
    storefront::{Storefront, StorefrontColumns},
    wallet::Wallet,
};
use scalars::PublicKey;
use tables::{
    auction_caches, auction_datas, auction_datas_ext, metadata_jsons, metadatas,
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
    async fn profile(
        &self,
        ctx: &AppContext,
        #[graphql(description = "Twitter handle")] handle: String,
    ) -> Option<Profile> {
        let twitter_bearer_token = &ctx.twitter_bearer_token;
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

    fn creator(
        &self,
        _context: &AppContext,
        #[graphql(description = "Address of creator")] address: String,
    ) -> Creator {
        Creator { address }
    }

    fn nfts(
        &self,
        context: &AppContext,
        #[graphql(description = "Filter on owner address")] owners: Option<Vec<PublicKey<Wallet>>>,
        #[graphql(description = "Filter on creator address")] creators: Option<
            Vec<PublicKey<Wallet>>,
        >,
        #[graphql(description = "Filter on attributes")] attributes: Option<Vec<AttributeFilter>>,
        #[graphql(description = "Filter on listed")] listed: Option<Vec<PublicKey<AuctionHouse>>>,
        #[graphql(description = "Limit for query")] limit: i32,
        #[graphql(description = "Offset for query")] offset: i32,
    ) -> FieldResult<Vec<Nft>> {
        if owners.is_none() && creators.is_none() && listed.is_none() {
            return Err(FieldError::new(
                "No filter provided! Please provide at least one of the filters",
                graphql_value!({ "Filters": "owners: Vec<PublicKey>, creators: Vec<PublicKey>, listed: Vec<PublicKey>" }),
            ));
        }

        let conn = context.db_pool.get().context("failed to connect to db")?;

        let query_options = queries::metadatas::ListQueryOptions {
            owners: owners.map(|a| a.into_iter().map(Into::into).collect()),
            creators: creators.map(|a| a.into_iter().map(Into::into).collect()),
            attributes: attributes.map(|a| a.into_iter().map(Into::into).collect()),
            listed: listed.map(|a| a.into_iter().map(Into::into).collect()),
            limit: limit.into(),
            offset: offset.into(),
        };
        let nfts = queries::metadatas::list(&conn, query_options)?;

        Ok(nfts.into_iter().map(Into::into).collect())
    }

    fn wallet(
        &self,
        _context: &AppContext,
        #[graphql(description = "Address of NFT")] address: String,
    ) -> Option<Wallet> {
        Some(Wallet { address })
    }

    fn listings(&self, context: &AppContext) -> FieldResult<Vec<Listing>> {
        let now = Local::now().naive_utc();
        let conn = context.db_pool.get()?;

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
        let conn = context.db_pool.get()?;
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
                metadata_jsons::description,
                metadata_jsons::image,
            ))
            .limit(1)
            .load(&conn)
            .context("Failed to load metadata")?;

        Ok(rows.pop().map(Into::into))
    }

    fn storefronts(&self, context: &AppContext) -> FieldResult<Vec<Storefront>> {
        let conn = context.db_pool.get()?;
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
        let conn = context.db_pool.get()?;
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
        let conn = context.db_pool.get()?;
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
