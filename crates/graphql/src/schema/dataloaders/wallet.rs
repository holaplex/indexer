use futures_util::future::join_all;
use objects::profile::{TwitterProfile, TwitterUserProfileResponse};

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

        let twitter_users: Vec<_> = join_all(twitter_users).await;

        Ok(twitter_users
            .into_iter()
            .zip(chunked_screen_names)
            .filter_map(|(result, _)| result.ok())
            .flatten()
            .map(|u| (u.screen_name.clone(), u.try_into()))
            .batch(screen_names))
    }
}
