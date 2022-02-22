use objects::auction_house::AuctionHouse;
use tables::auction_houses;

use super::prelude::*;

pub struct AuctionHouseBatcher {
    pub db_pool: Arc<Pool>,
}

#[async_trait]
impl BatchFn<String, Vec<AuctionHouse>> for AuctionHouseBatcher {
    async fn load(&mut self, addresses: &[String]) -> HashMap<String, Vec<AuctionHouse>> {
        let conn = self.db_pool.get().unwrap();
        let mut hash_map = HashMap::new();

        for auction_house in addresses {
            hash_map.insert(auction_house.clone(), Vec::new());
        }

        let rows: Vec<models::AuctionHouse> = auction_houses::table
            .filter(auction_houses::address.eq(any(addresses)))
            .load(&conn)
            .unwrap();

        rows.into_iter()
            .fold(hash_map, |mut acc, ah: models::AuctionHouse| {
                let ah = AuctionHouse::from(ah);
                acc.entry(ah.address.clone()).and_modify(|ahs| {
                    ahs.push(ah);
                });
                acc
            })
    }
}
