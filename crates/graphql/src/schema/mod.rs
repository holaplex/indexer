#![allow(clippy::module_name_repetitions)]

use juniper::{EmptyMutation, EmptySubscription, GraphQLObject, RootNode};

mod context;
pub(self) mod dataloaders;
pub(self) mod objects;
mod query_root;
pub(self) mod strings;
mod util;

mod prelude {
    pub use std::{collections::HashMap, sync::Arc};

    pub use indexer_core::{
        db::{models, tables, Pool},
        prelude::*,
    };
    pub use juniper::{
        graphql_object, graphql_value, FieldError, FieldResult, GraphQLInputObject, GraphQLObject,
    };

    pub(super) use super::{context::AppContext, dataloaders, objects, strings};
}

pub use context::AppContext;
pub(self) use util::Lamports;

#[derive(Debug, Clone, GraphQLObject)]
struct NftDetail {
    description: String,
    image: String,
}

pub type Schema = RootNode<
    'static,
    query_root::QueryRoot,
    EmptyMutation<AppContext>,
    EmptySubscription<AppContext>,
>;

pub fn create() -> Schema {
    Schema::new(
        query_root::QueryRoot,
        EmptyMutation::new(),
        EmptySubscription::new(),
    )
}
