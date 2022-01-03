use indexer_core::{hash::HashMap, pubkeys};
use metaplex_auction::processor::{BidderMetadata, BIDDER_METADATA_LEN};

use crate::{client::prelude::*, prelude::*, util, Client, Job, ThreadPoolHandle};

pub fn get(client: &Client, handle: ThreadPoolHandle) -> Result<()> {
    let mut map = HashMap::default();
    let mut count: usize = 0;

    client
        .get_program_accounts(pubkeys::auction(), RpcProgramAccountsConfig {
            filters: Some(vec![RpcFilterType::DataSize(
                BIDDER_METADATA_LEN.try_into().unwrap(),
            )]),
            ..RpcProgramAccountsConfig::default()
        })
        .context("Failed to retrieve bids for auction")?
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

    for (auction, bids) in map {
        todo!();
        // handle.push(Job::BidsForAuction(auction, bids));
    }

    Ok(())
}
