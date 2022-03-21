use objects::{auction_house::AuctionHouse, mint_stats::MintStats};
use scalars::PublicKey;

use super::prelude::*;

#[async_trait]
impl TryBatchFn<PublicKey<AuctionHouse>, Vec<MintStats>> for Batcher {
    async fn load(
        &mut self,
        addresses: &[PublicKey<AuctionHouse>],
    ) -> TryBatchMap<PublicKey<AuctionHouse>, Vec<MintStats>> {
        todo!()
    }
}
