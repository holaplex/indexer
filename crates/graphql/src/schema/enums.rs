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
pub enum CollectionNFTSort {
    #[graphql(name = "PRICE")]
    Price,
    #[graphql(name = "LISTED_AT")]
    ListedAt,
}

impl From<CollectionNFTSort> for db::custom_types::Sort {
    fn from(other: CollectionNFTSort) -> Self {
        match other {
            CollectionNFTSort::Price => Self::Price,
            CollectionNFTSort::ListedAt => Self::ListedAt,
        }
    }
}
