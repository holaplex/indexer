use std::{panic::AssertUnwindSafe, sync::Arc};

use indexer_core::{hash::HashMap, pubkeys};
use metaplex_auction::processor::{BidderMetadata, BIDDER_METADATA_LEN};
use parking_lot::RwLock;

use crate::{client::prelude::*, prelude::*, util, Client, ThreadPoolHandle};

type BidMapInner = RwLock<HashMap<Pubkey, Vec<BidderMetadata>>>;
pub struct BidMap(AssertUnwindSafe<Arc<BidMapInner>>);

impl std::ops::Deref for BidMap {
    type Target = BidMapInner;

    fn deref(&self) -> &Self::Target {
        &*self.0
    }
}

impl Default for BidMap {
    fn default() -> Self {
        Self(AssertUnwindSafe(Arc::new(RwLock::new(HashMap::default()))))
    }
}

impl Clone for BidMap {
    fn clone(&self) -> Self {
        Self(AssertUnwindSafe(Arc::clone(&*self.0)))
    }
}

pub fn get(client: &Client, bid_map: &BidMap, _handle: ThreadPoolHandle) -> Result<()> {
    let mut map = HashMap::default();
    let mut count: usize = 0;

    let start_time = Local::now();

    let res = client.get_program_accounts(pubkeys::auction(), RpcProgramAccountsConfig {
        filters: Some(vec![RpcFilterType::DataSize(
            BIDDER_METADATA_LEN.try_into().unwrap(),
        )]),
        ..RpcProgramAccountsConfig::default()
    });

    let end_time = Local::now();

    info!(
        "Bidder metadata call completed in {}",
        util::duration_hhmmssfff(end_time - start_time)
    );

    res.context("Failed to retrieve bids for auction")?
        .into_iter()
        .filter_map(|(key, mut acct)| {
            let parsed = BidderMetadata::from_account_info(&util::account_as_info(
                &key, false, false, &mut acct,
            ))
            .map_err(|e| debug!("Failed to parse possible bidder metadata: {:?}", e))
            .ok()?;

            let (key2, _bump) =
                pubkeys::find_bidder_metadata(parsed.auction_pubkey, parsed.bidder_pubkey);

            if key != key2 {
                debug!("Failed to assert derivation of bidder metadata PDA");
                return None;
            }

            Some(parsed)
        })
        .for_each(|acct| {
            count += 1;
            map.entry(acct.auction_pubkey)
                .or_insert_with(Vec::new)
                .push(acct);
        });

    *bid_map.write() = map;

    Ok(())
}
