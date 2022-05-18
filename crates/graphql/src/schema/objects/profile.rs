use serde::Deserialize;
use tables::twitter_handle_name_services;

use super::prelude::*;

#[derive(Debug, Clone, Deserialize)]
pub struct TwitterUserProfileResponse {
    pub screen_name: String,
    pub description: String,
    pub profile_image_url_https: String,
    pub profile_banner_url: String,
}

#[derive(Debug, Clone)]
pub struct TwitterProfile {
    pub handle: String,
    pub profile_image_url_lowres: String,
    pub profile_image_url_highres: String,
    pub banner_image_url: String,
    pub description: String,
}

impl From<TwitterUserProfileResponse> for TwitterProfile {
    fn from(
        TwitterUserProfileResponse {
            screen_name,
            description,
            profile_image_url_https,
            profile_banner_url,
        }: TwitterUserProfileResponse,
    ) -> Self {
        Self {
            handle: screen_name,
            profile_image_url_lowres: profile_image_url_https.clone(),
            profile_image_url_highres: profile_image_url_https.replace("_normal.", "."),
            banner_image_url: profile_banner_url,
            description,
        }
    }
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
