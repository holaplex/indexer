use indexer_core::db;

#[derive(Debug, Clone, Copy, juniper::GraphQLEnum)]
#[graphql(description = "Sorts results ascending or descending")]
pub enum OrderDirection {
    #[graphql(name = "DESC")]
    Desc,
    #[graphql(name = "ASC")]
    Asc,
}

impl From<OrderDirection> for db::custom_types::OrderDirection {
    fn from(other: OrderDirection) -> Self {
        match other {
            OrderDirection::Desc => Self::Desc,
            OrderDirection::Asc => Self::Asc,
        }
    }
}

impl From<OrderDirection> for db::Order {
    fn from(other: OrderDirection) -> Self {
        match other {
            OrderDirection::Desc => Self::Desc,
            OrderDirection::Asc => Self::Asc,
        }
    }
}

#[derive(Debug, Clone, Copy, juniper::GraphQLEnum)]
#[graphql(description = "Sorts results by price or listed at")]
pub enum WalletNftSort {
    #[graphql(name = "PRICE")]
    Price,
    #[graphql(name = "LISTED_AT")]
    ListedAt,
}

impl From<WalletNftSort> for db::custom_types::Sort {
    fn from(other: WalletNftSort) -> Self {
        match other {
            WalletNftSort::Price => Self::Price,
            WalletNftSort::ListedAt => Self::ListedAt,
        }
    }
}

#[derive(Debug, Clone, Copy, juniper::GraphQLEnum)]
#[graphql(description = "Sorts collection results")]
pub enum CollectionSort {
    #[graphql(name = "FLOOR_PRICE")]
    FloorPrice,
    #[graphql(name = "_1D_VOLUME")]
    _1dVolume,
    #[graphql(name = "_7D_VOLUME")]
    _7dVolume,
    #[graphql(name = "_30D_VOLUME")]
    _30dVolume,
    #[graphql(name = "_1D_SALES_COUNT")]
    _1dSalesCount,
    #[graphql(name = "_7D_SALES_COUNT")]
    _7dSalesCount,
    #[graphql(name = "_30D_SALES_COUNT")]
    _30dSalesCount,
}

impl From<CollectionSort> for db::custom_types::CollectionSort {
    fn from(other: CollectionSort) -> Self {
        match other {
            CollectionSort::FloorPrice => Self::FloorPrice,
            CollectionSort::_1dVolume => Self::_1dVolume,
            CollectionSort::_7dVolume => Self::_7dVolume,
            CollectionSort::_30dVolume => Self::_30dVolume,
            CollectionSort::_1dSalesCount => Self::_1dSalesCount,
            CollectionSort::_7dSalesCount => Self::_7dSalesCount,
            CollectionSort::_30dSalesCount => Self::_30dSalesCount,
        }
    }
}
