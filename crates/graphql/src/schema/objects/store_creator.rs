use objects::nft::BaseNft;

use super::{prelude::*, profile::TwitterProfile};

#[derive(Debug, Clone)]
pub struct StoreCreator {
    pub store_config_address: String,
    pub creator_address: String,
    pub twitter_handle: Option<String>,
}

#[graphql_object(Context = AppContext)]
impl StoreCreator {
    pub fn store_config_address(&self) -> &str {
        &self.store_config_address
    }

    pub fn creator_address(&self) -> &str {
        &self.creator_address
    }

    pub fn twitter_handle(&self) -> Option<&str> {
        self.twitter_handle.as_deref()
    }

    pub async fn preview(&self, context: &AppContext) -> FieldResult<Vec<BaseNft>> {
        context
            .collection_loader
            .load(self.creator_address.clone().into())
            .await
            .map_err(Into::into)
    }

    pub async fn profile(&self, ctx: &AppContext) -> FieldResult<Option<TwitterProfile>> {
        let twitter_handle = match self.twitter_handle {
            Some(ref t) => t.clone(),
            None => return Ok(None),
        };

        ctx.twitter_profile_loader
            .load(twitter_handle)
            .await
            .map_err(Into::into)
    }

    pub async fn nft_count(&self, context: &AppContext) -> FieldResult<Option<i32>> {
        context
            .collection_count_loader
            .load(self.creator_address.clone().into())
            .await
            .map_err(Into::into)
    }
}

impl<'a> From<(Option<String>, models::StoreCreator<'a>)> for StoreCreator {
    fn from(
        (
            twitter_handle,
            models::StoreCreator {
                store_config_address,
                creator_address,
            },
        ): (Option<String>, models::StoreCreator),
    ) -> Self {
        Self {
            store_config_address: store_config_address.into_owned(),
            creator_address: creator_address.into_owned(),
            twitter_handle,
        }
    }
}
