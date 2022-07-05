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