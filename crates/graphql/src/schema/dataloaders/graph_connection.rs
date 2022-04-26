use indexer_core::db::queries;
use objects::graph_connection::GraphConnection;
use scalars::PublicKey;

use super::prelude::*;

#[async_trait]
impl TryBatchFn<PublicKey<GraphConnection>, Option<GraphConnection>> for Batcher {
    async fn load(
        &mut self,
        addresses: &[PublicKey<GraphConnection>],
    ) -> TryBatchMap<PublicKey<GraphConnection>, Option<GraphConnection>> {
        let conn = self.db()?;

        let rows = queries::graph_connection::list(&conn, addresses)?;

        Ok(rows
            .into_iter()
            .map(|gc| (gc.connection_address.clone(), gc.try_into()))
            .batch(addresses))
    }
}
