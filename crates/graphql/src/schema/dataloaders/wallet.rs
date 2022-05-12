use futures_util::future::join_all;
use objects::profile::{TwitterProfile, TwitterUserProfileResponse};

use crate::schema::objects::profile::TwitterProfilePictureResponse;

use super::prelude::*;

const TWITTER_SCREEN_NAME_CHUNKS: usize = 100;

#[async_trait]
impl TryBatchFn<String, Option<TwitterProfile>> for TwitterBatcher {
    async fn load(
        &mut self,
        screen_names: &[String],
    ) -> TryBatchMap<String, Option<TwitterProfile>> {
        let http_client = reqwest::Client::new();
        let twitter_bearer_token = self.bearer();

        let chunked_screen_names = screen_names.chunks(TWITTER_SCREEN_NAME_CHUNKS);

        let twitter_users = chunked_screen_names
            .clone()
            .into_iter()
            .map(|screen_names| {
                let http_client = &http_client;

                async move {
                    http_client
                        .post("https://api.twitter.com/1.1/users/lookup.json")
                        .header("Accept", "application/json")
                        .form(&[("screen_name", &screen_names.join(", "))])
                        .bearer_auth(twitter_bearer_token)
                        .send()
                        .await
                        .map_err(Error::model_convert)?
                        .json::<Vec<TwitterUserProfileResponse>>()
                        .await
                        .map_err(Error::model_convert)
                }
            })
            .collect::<Vec<_>>();

        let twitter_users_pfps = chunked_screen_names
            .clone()
            .into_iter()
            .map(|screen_names| {
                let http_client = &http_client;

                screen_names.into_iter().map(|screen_name| async move {
                    http_client
                        .get(format!(
                            "https://api.twitter.com/2/users/by/username/{}",
                            screen_name
                        ))
                        .header("Accept", "application/json")
                        .query(&[("user.fields", "profile_image_url")])
                        .bearer_auth(twitter_bearer_token)
                        .send()
                        .await
                        .map_err(Error::model_convert)?
                        .json::<TwitterProfilePictureResponse>()
                        .await
                        .map_err(Error::model_convert)
                }).collect::<Vec<_>>()
            })
            .collect::<Vec<_>>();

        // TODO: twitter_users_pfps is a vec of vecs of futures... need to flatten, then join_all with twitter_users

        let twitter_users: Vec<_> = join_all(twitter_users).await;

        // TODO: iterate over both twitter_users_pfps and twitter_users, adding to a map of 
        //  Map<String (handle), (TwitterProfilePictureResponse, TwitterUserProfileResponse)>
        //  and then extracting the vector of tuples of values to pass into the converter


        Ok(twitter_users
            .into_iter()
            .zip(chunked_screen_names)
            .filter_map(|(result, _)| result.ok())
            .flatten()
            .map(|u| (u.screen_name.clone(), u.try_into()))
            .batch(screen_names))
    }
}
