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
pub enum NftSort {
    #[graphql(name = "PRICE")]
    Price,
    #[graphql(name = "LISTED_AT")]
    ListedAt,
}

impl From<NftSort> for db::custom_types::NftSort {
    fn from(other: NftSort) -> Self {
        match other {
            NftSort::Price => Self::Price,
            NftSort::ListedAt => Self::ListedAt,
        }
    }
}

#[derive(Debug, Clone, Copy, juniper::GraphQLEnum)]
#[graphql(description = "Sorts collection results")]
pub enum CollectionSort {
    #[graphql(name = "VOLUME")]
    Volume,
    #[graphql(name = "FLOOR")]
    Floor,
    #[graphql(name = "NUMBER_LISTED")]
    NumberListed,
}

#[derive(Debug, Clone, Copy, juniper::GraphQLEnum)]
#[graphql(description = "Collection intervals")]
pub enum CollectionInterval {
    #[graphql(name = "ONE_DAY")]
    One,
    #[graphql(name = "SEVEN_DAY")]
    Seven,
    #[graphql(name = "THIRTY_DAY")]
    Thirty,
}

#[derive(Debug, Clone, Copy, juniper::GraphQLEnum)]
#[graphql(description = "Reward center mathematical operands")]
pub enum PayoutOperation {
    #[graphql(name = "MULTIPLE")]
    Multiple,
    #[graphql(name = "DIVIDE")]
    Divide,
}

impl From<db::custom_types::PayoutOperationEnum> for PayoutOperation {
    fn from(operation: db::custom_types::PayoutOperationEnum) -> Self {
        match operation {
            db::custom_types::PayoutOperationEnum::Multiple => Self::Multiple,
            db::custom_types::PayoutOperationEnum::Divide => Self::Divide,
        }
    }
}

#[derive(Debug, Clone, Copy, juniper::GraphQLEnum)]
#[graphql(description = "Sorts results by price or listed at")]
pub enum OfferType {
    #[graphql(name = "OFFER_PLACED")]
    OfferPlaced,
    #[graphql(name = "OFFER_RECEIVED")]
    OfferReceived,
}

impl From<OfferType> for String {
    fn from(other: OfferType) -> Self {
        match other {
            OfferType::OfferPlaced => String::from("OFFER_PLACED"),
            OfferType::OfferReceived => String::from("OFFER_RECEIVED"),
        }
    }
}
