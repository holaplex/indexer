use super::prelude::*;

#[derive(Debug, Clone, GraphQLObject)]
pub struct StoreCreator {
    pub store_config_address: String,
    pub creator_address: String,
}

impl<'a> From<models::StoreCreator<'a>> for StoreCreator {
    fn from(
        models::StoreCreator {
            store_config_address,
            creator_address,
        }: models::StoreCreator,
    ) -> Self {
        Self {
            store_config_address: store_config_address.into_owned(),
            creator_address: creator_address.into_owned(),
        }
    }
}
