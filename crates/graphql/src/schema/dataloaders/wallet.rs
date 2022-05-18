use futures_util::future::join_all;
use objects::profile::{TwitterProfile, TwitterUserProfileResponse};

use super::prelude::*;

#[async_trait]
impl TryBatchFn<String, Option<TwitterProfile>> for TwitterBatcher {
    async fn load(
        &mut self,
        screen_names: &[String],
    ) -> TryBatchMap<String, Option<TwitterProfile>> {
        let http_client = reqwest::Client::new();
        let endpoint = self.endpoint();

        let twitter_users = screen_names
            .into_iter()
            .map(|screen_name| {
                let http_client = &http_client;
                async move {
                    http_client
                        .get(format!("{}/twitter/{}", endpoint, screen_name))
                        .header("Accept", "application/json")
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
            //.zip(chunked_screen_names)
            .filter_map(|result| result.ok())
            .flatten()
            .map(|u| (u.screen_name.clone(), u.try_into()))
            .batch(screen_names))
    }
}
