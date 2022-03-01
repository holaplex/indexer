use serde::Deserialize;

use super::prelude::*;

#[derive(Debug, Clone)]
pub struct Profile {
    pub handle: String,
    pub profile_image_url_lowres: String,
    pub profile_image_url_highres: String,
    pub banner_image_url: String,
}

#[derive(Debug, Deserialize)]
pub struct TwitterProfilePictureResponse {
    pub data: TwitterProfilePicture,
}

#[derive(Debug, Deserialize)]
pub struct TwitterProfilePicture {
    pub profile_image_url: String,
}

#[derive(Debug, Deserialize)]
pub struct TwitterShowResponse {
    pub screen_name: String,
    pub profile_image_url_https: String,
    pub profile_banner_url: String,
}

#[graphql_object(Context = AppContext)]
impl Profile {
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
