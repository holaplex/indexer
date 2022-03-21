use super::{nft::Nft, prelude::*};

#[derive(Debug, Clone)]
pub struct StoreCreator {
    pub store_config_address: String,
    pub creator_address: String,
}

#[graphql_object(Context = AppContext)]
impl StoreCreator {
    pub fn store_config_address(&self) -> &str {
        &self.store_config_address
    }

    pub fn creator_address(&self) -> &str {
        &self.creator_address
    }

    pub async fn preview(&self, context: &AppContext) -> FieldResult<Vec<Nft>> {
        context
            .collection_loader
            .load(self.creator_address.clone().into())
            .await
            .map_err(Into::into)
    }
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
