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
