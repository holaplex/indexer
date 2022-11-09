use enums::{NftSort, OfferType, OrderDirection};
use indexer_core::{
    bigdecimal::{BigDecimal, ToPrimitive, Zero},
    db::queries::{self, metadatas::WalletNftOptions},
    pubkeys,
    uuid::Uuid,
};
use objects::{
    auction_house::AuctionHouse,
    collection::Collection,
    listing::Bid,
    nft::{Nft, NftCreator},
    profile::TwitterProfile,
};
use scalars::{markers::TokenMint, PublicKey, U64};
use tables::{associated_token_accounts, bids, graph_connections, wallet_total_rewards};

use super::{ah_offer::Offer, prelude::*, reward_center::RewardCenter};

#[derive(Debug, Clone)]
pub struct Wallet {
    pub address: PublicKey<Wallet>,
    pub twitter_handle: Option<String>,
}

impl From<(models::WalletTotal, Option<String>)> for Wallet {
    fn from(
        (models::WalletTotal { address, .. }, twitter_handle): (
            models::WalletTotal,
            Option<String>,
        ),
    ) -> Self {
        Self {
            address: address.into(),
            twitter_handle,
        }
    }
}

impl Wallet {
    pub fn new(address: PublicKey<Wallet>, twitter_handle: Option<String>) -> Self {
        Self {
            address,
            twitter_handle,
        }
    }
}

impl From<serde_json::Value> for Wallet {
    fn from(value: serde_json::Value) -> Self {
        Self {
            address: value
                .get("owner")
                .and_then(serde_json::Value::as_str)
                .map_or_else(|| String::new().into(), |s| s.to_string().into()),
            twitter_handle: value
                .get("handle")
                .and_then(serde_json::Value::as_str)
                .map(Into::into),
        }
    }
}

#[derive(Debug, Clone)]
pub struct WalletNftCount {
    wallet: PublicKey<Wallet>,
    creators: Option<Vec<PublicKey<NftCreator>>>,
}

impl WalletNftCount {
    #[must_use]
    pub fn new(wallet: PublicKey<Wallet>, creators: Option<Vec<PublicKey<NftCreator>>>) -> Self {
        Self { wallet, creators }
    }
}

#[graphql_object(Context = AppContext)]
impl WalletNftCount {
    fn owned(&self, context: &AppContext) -> FieldResult<i32> {
        let conn = context.shared.db.get()?;

        let count = queries::nft_count::owned(&conn, &self.wallet, self.creators.as_deref())?;

        Ok(count.try_into()?)
    }

    #[graphql(arguments(auction_houses(description = "auction houses to scope wallet counts")))]
    fn offered(
        &self,
        context: &AppContext,
        auction_houses: Option<Vec<PublicKey<AuctionHouse>>>,
    ) -> FieldResult<i32> {
        let conn = context.shared.db.get()?;

        let count = queries::nft_count::offered(
            &conn,
            &self.wallet,
            self.creators.as_deref(),
            auction_houses.as_deref(),
            pubkeys::OPENSEA_AUCTION_HOUSE.to_string(),
        )?;

        Ok(count.try_into()?)
    }

    #[graphql(arguments(auction_houses(description = "auction houses to scope wallet counts")))]
    fn listed(
        &self,
        context: &AppContext,
        auction_houses: Option<Vec<PublicKey<AuctionHouse>>>,
    ) -> FieldResult<i32> {
        let conn = context.shared.db.get()?;

        let count = queries::nft_count::wallet_listed(
            &conn,
            &self.wallet,
            self.creators.as_deref(),
            auction_houses.as_deref(),
        )?;

        Ok(count.try_into()?)
    }

    fn created(&self, context: &AppContext) -> FieldResult<i32> {
        let conn = context.shared.db.get()?;

        let count = queries::nft_count::created(&conn, &self.wallet)?;

        Ok(count.try_into()?)
    }
}

#[derive(Debug, Clone)]
pub struct CollectedCollection {
    collection_id: String,
    nfts_owned: i32,
    estimated_value: U64,
}

impl TryFrom<models::CollectedCollection> for CollectedCollection {
    type Error = std::num::TryFromIntError;

    fn try_from(
        models::CollectedCollection {
            collection_id,
            nfts_owned,
            estimated_value,
        }: models::CollectedCollection,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            collection_id,
            nfts_owned: nfts_owned.try_into()?,
            estimated_value: estimated_value.try_into()?,
        })
    }
}

#[graphql_object(Context = AppContext)]
impl CollectedCollection {
    async fn collection(&self, ctx: &AppContext) -> FieldResult<Option<Collection>> {
        ctx.generic_collection_loader
            .load(self.collection_id.clone())
            .await
            .map_err(Into::into)
    }

    fn nfts_owned(&self) -> i32 {
        self.nfts_owned
    }

    fn estimated_value(&self) -> U64 {
        self.estimated_value
    }
}

#[derive(Debug, Clone)]
pub struct WalletActivity {
    pub id: Uuid,
    pub metadata: PublicKey<Nft>,
    pub auction_house: PublicKey<AuctionHouse>,
    pub marketplace_program_address: String,
    pub price: U64,
    pub created_at: DateTime<Utc>,
    pub wallets: Vec<Wallet>,
    pub activity_type: String,
}

impl TryFrom<models::WalletActivity> for WalletActivity {
    type Error = std::num::TryFromIntError;

    fn try_from(
        models::WalletActivity {
            id,
            metadata,
            auction_house,
            marketplace_program,
            price,
            created_at,
            wallets,
            wallet_twitter_handles,
            activity_type,
        }: models::WalletActivity,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            id,
            metadata: metadata.into(),
            auction_house: auction_house.into(),
            marketplace_program_address: marketplace_program,
            price: price.try_into()?,
            created_at: DateTime::from_utc(created_at, Utc),
            wallets: wallets
                .into_iter()
                .zip(wallet_twitter_handles.into_iter())
                .map(|(address, twitter_handle)| Wallet::new(address.into(), twitter_handle))
                .collect(),
            activity_type,
        })
    }
}

#[graphql_object(Context = AppContext)]
impl WalletActivity {
    fn id(&self) -> &Uuid {
        &self.id
    }

    fn metadata(&self) -> &PublicKey<Nft> {
        &self.metadata
    }

    fn price(&self) -> U64 {
        self.price
    }

    fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }

    fn wallets(&self) -> &Vec<Wallet> {
        &self.wallets
    }

    fn activity_type(&self) -> &str {
        &self.activity_type
    }

    fn marketplace_program_address(&self) -> &str {
        &self.marketplace_program_address
    }

    pub async fn nft(&self, ctx: &AppContext) -> FieldResult<Option<Nft>> {
        ctx.nft_loader
            .load(self.metadata.clone())
            .await
            .map_err(Into::into)
    }

    pub async fn auction_house(&self, context: &AppContext) -> FieldResult<Option<AuctionHouse>> {
        context
            .auction_house_loader
            .load(self.auction_house.clone())
            .await
            .map_err(Into::into)
    }
}

#[graphql_object(Context = AppContext)]
impl Wallet {
    pub fn address(&self) -> &PublicKey<Wallet> {
        &self.address
    }

    pub fn twitter_handle(&self) -> Option<&str> {
        self.twitter_handle.as_deref()
    }

    pub async fn nfts(
        &self,
        ctx: &AppContext,
        auction_house: Option<String>,
        marketplace_program: Option<String>,
        collections: Option<Vec<String>>,
        sort_by: Option<NftSort>,
        order_by: Option<OrderDirection>,
        limit: i32,
        offset: i32,
    ) -> FieldResult<Vec<Nft>> {
        let conn = ctx.shared.db.get()?;

        let nfts = queries::metadatas::wallet_nfts(
            &conn,
            WalletNftOptions {
                wallet: self.address.clone().into(),
                auction_house,
                marketplace_program,
                collections: collections.map(|c| c.into_iter().map(Into::into).collect()),
                sort_by: sort_by.map(Into::into),
                order: order_by.map(Into::into),
                limit: limit.try_into()?,
                offset: offset.try_into()?,
            },
            pubkeys::OPENSEA_AUCTION_HOUSE.to_string(),
        )?;

        nfts.into_iter()
            .map(TryInto::try_into)
            .collect::<Result<_, _>>()
            .map_err(Into::into)
    }

    pub fn associated_token_accounts(
        &self,
        ctx: &AppContext,
        mint_address: Option<PublicKey<TokenMint>>,
    ) -> FieldResult<Vec<AssociatedTokenAccount>> {
        let conn = ctx.shared.db.get()?;

        let mut query = associated_token_accounts::table.into_boxed();
        if let Some(mint_address) = mint_address {
            query = query.filter(associated_token_accounts::mint.eq(mint_address));
        }
        let accts: Vec<models::AssociatedTokenAccount> = query
            .select(associated_token_accounts::all_columns)
            .filter(associated_token_accounts::owner.eq(&self.address))
            .load(&conn)
            .context("Failed to load token accounts")?;

        accts
            .into_iter()
            .map(TryInto::try_into)
            .collect::<Result<_, _>>()
            .map_err(Into::into)
    }

    pub fn collected_collections(&self, ctx: &AppContext) -> FieldResult<Vec<CollectedCollection>> {
        let conn = ctx.shared.db.get()?;

        let collections = queries::wallet::collected_collections(&conn, &self.address)?;
        collections
            .into_iter()
            .map(TryInto::try_into)
            .collect::<Result<_, _>>()
            .map_err(Into::into)
    }

    pub fn activities(
        &self,
        ctx: &AppContext,
        event_types: Option<Vec<String>>,
        limit: i32,
        offset: i32,
    ) -> FieldResult<Vec<WalletActivity>> {
        let conn = ctx.shared.db.get()?;

        let activities =
            queries::wallet::activities(&conn, &self.address, event_types, limit, offset)?;

        activities
            .into_iter()
            .map(TryInto::try_into)
            .collect::<Result<_, _>>()
            .map_err(Into::into)
    }

    pub fn offers(
        &self,
        ctx: &AppContext,
        offer_type: Option<OfferType>,
        limit: i32,
        offset: i32,
    ) -> FieldResult<Vec<Offer>> {
        let conn = ctx.shared.db.get()?;
        let offer_type: Option<String> = offer_type.map(Into::into);
        let offers = queries::wallet::offers(&conn, &self.address, offer_type, limit, offset)?;

        offers
            .into_iter()
            .map(TryInto::try_into)
            .collect::<Result<_, _>>()
            .map_err(Into::into)
    }

    pub fn bids(&self, ctx: &AppContext) -> FieldResult<Vec<Bid>> {
        let db_conn = ctx.shared.db.get()?;

        let rows: Vec<models::Bid> = bids::table
            .select(bids::all_columns)
            .filter(bids::bidder_address.eq(&self.address))
            .order_by(bids::last_bid_time.desc())
            .load(&db_conn)
            .context("Failed to load wallet bids")?;

        rows.into_iter()
            .map(TryInto::try_into)
            .collect::<Result<_, _>>()
            .map_err(Into::into)
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

    pub fn connection_counts(&self) -> FieldResult<ConnectionCounts> {
        Ok(ConnectionCounts {
            address: self.address.clone(),
        })
    }

    #[graphql(arguments(creators(description = "a list of auction house public keys")))]
    pub fn nft_counts(
        &self,
        _ctx: &AppContext,
        creators: Option<Vec<PublicKey<NftCreator>>>,
    ) -> WalletNftCount {
        WalletNftCount::new(self.address.clone(), creators)
    }

    pub fn total_rewards(
        &self,
        ctx: &AppContext,
        reward_center: PublicKey<RewardCenter>,
    ) -> FieldResult<U64> {
        let db_conn = ctx.shared.db.get()?;

        let wallet_total_reward = wallet_total_rewards::table
            .select(wallet_total_rewards::total_reward)
            .filter(
                wallet_total_rewards::reward_center_address
                    .eq(&reward_center)
                    .and(wallet_total_rewards::wallet_address.eq(&self.address)),
            )
            .first::<BigDecimal>(&db_conn)
            .optional()
            .transpose()
            .unwrap_or_else(|| Ok(BigDecimal::zero()))
            .unwrap_or_else(|_| BigDecimal::zero());

        Ok(wallet_total_reward.to_u64().unwrap_or_default().into())
    }
}

pub struct ConnectionCounts {
    pub address: PublicKey<Wallet>,
}

#[graphql_object(Context = AppContext)]
impl ConnectionCounts {
    pub fn from_count(&self, ctx: &AppContext) -> FieldResult<i32> {
        let db_conn = ctx.shared.db.get()?;

        let count: i64 = graph_connections::table
            .filter(graph_connections::from_account.eq(&self.address))
            .count()
            .get_result(&db_conn)
            .context("Failed to count from_connections")?;

        Ok(count.try_into()?)
    }

    pub fn to_count(&self, ctx: &AppContext) -> FieldResult<i32> {
        let db_conn = ctx.shared.db.get()?;

        let count: i64 = graph_connections::table
            .filter(graph_connections::to_account.eq(&self.address))
            .count()
            .get_result(&db_conn)
            .context("Failed to count to_connections")?;

        Ok(count.try_into()?)
    }
}

#[derive(Debug, Clone, GraphQLObject)]
pub struct AssociatedTokenAccount {
    pub address: PublicKey<AssociatedTokenAccount>,
    pub mint: PublicKey<TokenMint>,
    pub owner: PublicKey<Wallet>,
    pub amount: U64,
}

impl<'a> TryFrom<models::AssociatedTokenAccount<'a>> for AssociatedTokenAccount {
    type Error = std::num::TryFromIntError;
    fn try_from(
        models::AssociatedTokenAccount {
            address,
            mint,
            owner,
            amount,
            ..
        }: models::AssociatedTokenAccount,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            address: address.into(),
            mint: mint.into(),
            owner: owner.into(),
            amount: amount.try_into().unwrap_or_default(),
        })
    }
}
