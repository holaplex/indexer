use std::borrow::Borrow;

use chrono::{offset::Local, Duration, NaiveDateTime};
use indexer_core::{
    db::{
        insert_into,
        models::{Bid, Listing},
        select,
        tables::{bids, listings},
        Connection,
    },
    prelude::*,
    pubkeys::find_auction_data_extended,
};
use metaplex_auction::processor::{
    AuctionData, AuctionDataExtended, BidState, BidderMetadata, PriceFloor,
};

use super::bidder_metadata::BidMap;
use crate::{
    bits::bidder_metadata::BidList, prelude::*, util, Client, RcAuctionKeys, ThreadPoolHandle,
};

pub fn process(
    client: &Client,
    token_account: Pubkey,
) -> Result<()> {
  
  Ok(())
}
