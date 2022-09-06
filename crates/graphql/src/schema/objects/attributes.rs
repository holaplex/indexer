use super::prelude::*;

#[derive(Debug, Clone, GraphQLObject, PartialEq, Eq, PartialOrd, Ord)]
pub struct AttributeVariant {
    pub name: String,
    pub count: i32,
}

#[derive(Debug, GraphQLObject, PartialEq, Eq, PartialOrd, Ord)]
pub struct AttributeGroup {
    pub name: String,
    pub variants: Vec<AttributeVariant>,
}
