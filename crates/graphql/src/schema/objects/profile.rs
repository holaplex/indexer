use serde::Deserialize;
use tables::twitter_handle_name_services;

use super::prelude::*;

#[derive(Debug, Clone, GraphQLObject)]
pub struct TwitterProfile {
    pub handle: String,
    pub profile_image_url: String,
    pub profile_image_url_highres: String,
    pub banner_image_url: String,
    pub description: String,
}

#[derive(Debug, Clone)]
pub struct Profile {
    pub handle: String,
    pub profile_image_url_lowres: String,
    pub profile_image_url_highres: String,
    pub banner_image_url: String,
}

impl From<(TwitterProfilePictureResponse, TwitterUserProfileResponse)> for TwitterProfile {
    fn from(
        (twitter_profile_picture_response, twitter_user_profile_response): (
            TwitterProfilePictureResponse,
            TwitterUserProfileResponse,
        ),
    ) -> Self {
        Self {
            handle: twitter_user_profile_response.screen_name,
            profile_image_url: twitter_user_profile_response.profile_image_url_https,
            profile_image_url_highres: twitter_profile_picture_response.data.profile_image_url,
            banner_image_url: twitter_user_profile_response.profile_banner_url,
            description: twitter_user_profile_response.description,
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct TwitterProfilePictureResponse {
    pub data: TwitterProfilePicture,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TwitterProfilePicture {
    pub profile_image_url: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TwitterShowResponse {
    pub screen_name: String,
    pub profile_image_url_https: String,
    pub profile_banner_url: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TwitterUserProfileResponse {
    pub screen_name: String,
    pub description: String,
    pub profile_image_url_https: String,
    pub profile_banner_url: String,
}

#[graphql_object(Context = AppContext)]
impl Profile {
    fn wallet_address(&self, ctx: &AppContext) -> FieldResult<Option<String>> {
        let db_conn = ctx.shared.db.get()?;
        let result: Vec<models::TwitterHandle> = twitter_handle_name_services::table
            .select(twitter_handle_name_services::all_columns)
            .limit(1)
            .filter(twitter_handle_name_services::twitter_handle.eq(&self.handle))
            .load(&db_conn)
            .context("Failed to load wallet address")?;
        if result.is_empty() {
            return Ok(None);
        }
        let matching_item = result.get(0).unwrap();
        let wallet_address = &matching_item.wallet_address;
        Ok(Some(wallet_address.to_string()))
    }
    fn handle(&self) -> &str {
        &self.handle
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
}

impl From<(TwitterProfilePictureResponse, TwitterShowResponse)> for Profile {
    fn from(
        (profile_picture_response, show_response): (
            TwitterProfilePictureResponse,
            TwitterShowResponse,
        ),
    ) -> Self {
        Self {
            banner_image_url: show_response.profile_banner_url,
            handle: show_response.screen_name,
            profile_image_url_highres: profile_picture_response.data.profile_image_url,
            profile_image_url_lowres: show_response.profile_image_url_https,
        }
    }
}
