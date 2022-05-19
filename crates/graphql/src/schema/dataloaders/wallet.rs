use futures_util::future::join_all;
use objects::profile::TwitterProfile;

use super::prelude::*;

#[async_trait]
impl TryBatchFn<String, Option<TwitterProfile>> for TwitterBatcher {
    async fn load(
        &mut self,
        screen_names: &[String],
    ) -> TryBatchMap<String, Option<TwitterProfile>> {
        let http_client = reqwest::Client::new();

        let twitter_users = screen_names.iter().map(|screen_name| {
            let http_client = &http_client;
            let _ = self.bearer();
            let url = self.proxy_url(screen_name);

            async move {
                http_client
                    .get(url.map_err(Error::model_convert)?)
                    .header("Accept", "application/json")
                    .send()
                    .await
                    .map_err(Error::model_convert)?
                    .json::<TwitterProfile>()
                    .await
                    .map_err(Error::model_convert)
            }
        });

        let twitter_users: Vec<_> = join_all(twitter_users).await;

        Ok(twitter_users
            .into_iter()
            .map(|r| r.map_err(|e| error!("Failed to load Twitter profile: {:?}", e)))
            .filter_map(Result::ok)
            .map(|u| (u.handle.clone(), u))
            .batch(screen_names))
    }
}
