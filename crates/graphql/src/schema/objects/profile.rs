use serde::Deserialize;
use tables::twitter_handle_name_services;

use super::prelude::*;

#[derive(Debug, Clone, Deserialize)]
pub struct TwitterProfile {
    pub handle: String,
    pub profile_image_url_lowres: String,
    pub profile_image_url_highres: String,
    pub banner_image_url: String,
    pub description: String,
}

#[graphql_object(Context = AppContext)]
impl TwitterProfile {
    fn wallet_address(&self, ctx: &AppContext) -> FieldResult<Option<String>> {
        let db_conn = ctx.shared.db.get()?;

        let handle = twitter_handle_name_services::table
            .select(twitter_handle_name_services::all_columns)
            .filter(twitter_handle_name_services::twitter_handle.eq(&self.handle))
            .first::<models::TwitterHandle>(&db_conn)
            .optional()
            .context("Failed to load wallet address")?;

        Ok(handle.map(|h| h.wallet_address.into_owned()))
    }

    fn handle(&self) -> &str {
        &self.handle
    }

    #[graphql(deprecated = "Use profileImageUrlLowres instead.")]
    fn profile_image_url(&self) -> &str {
        &self.profile_image_url_lowres
    }

    fn profile_image_url_lowres(&self) -> &str {
        &self.profile_image_url_lowres
    }

    fn profile_image_url_highres(&self) -> &str {
        &self.profile_image_url_highres
    }

    fn banner_image_url(&self) -> &str {
        &self.banner_image_url
    }

    fn description(&self) -> &str {
        &self.description
    }
}
