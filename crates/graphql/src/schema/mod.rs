#![allow(clippy::module_name_repetitions)]

use juniper::{EmptyMutation, EmptySubscription, RootNode};

mod context;
pub(self) mod dataloaders;
pub(self) mod objects;
mod query_root;
pub(self) mod scalars;

pub(self) mod prelude {
    pub use std::{collections::HashMap, sync::Arc};

    pub use indexer_core::{
        db::{models, tables, Pool},
        prelude::*,
    };
    pub use juniper::{
        graphql_object, graphql_value, FieldError, FieldResult, GraphQLInputObject, GraphQLObject,
    };

    pub(super) use super::{context::AppContext, dataloaders, objects, scalars};
    pub(crate) use crate::SharedData;
}

pub use context::AppContext;

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
