use indexer_core::{prelude::*, pubkeys::find_auction_data_extended};
use metaplex_auction::processor::{AuctionData, AuctionDataExtended};

use crate::{util, Client, RcAuctionKeys, ThreadPoolHandle};

pub fn process(client: &Client, keys: &RcAuctionKeys, _handle: &ThreadPoolHandle) -> Result<()> {
    let (ext, _bump) = find_auction_data_extended(&keys.vault);

    let mut acct = client
        .get_account(&keys.auction)
        .context("Failed to get auction data")?;

    let auction = AuctionData::from_account_info(&util::account_as_info(
        &keys.auction,
        false,
        false,
        &mut acct,
    ))
    .context("Failed to parse AuctionData")?;

    let mut acct = client
        .get_account(&ext)
        .context("Failed to get extended auction data")?;

    let ext = AuctionDataExtended::from_account_info(&util::account_as_info(
        &ext, false, false, &mut acct,
    ))
    .context("Failed to parse AuctionDataExtended")?;

    // TODO
    info!("Auction! {:?}", auction);
    info!("Auction extended! {:?}", ext);

    Ok(())
}
