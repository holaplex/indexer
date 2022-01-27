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
use crate::{bits::bidder_metadata::BidList, prelude::*, util, Client, RcAuctionKeys};

pub async fn process(client: &Client, keys: &RcAuctionKeys, bid_map: &BidMap) -> Result<()> {
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

    let total_uncancelled_bids;
    let highest_bid;

    match auction.bid_state {
        BidState::EnglishAuction { ref bids, .. } => {
            total_uncancelled_bids = Some(
                bids.len()
                    .try_into()
                    .context("Bid count is too high to store!")?,
            );

            highest_bid = bids
                .iter()
                .map(|b| b.1)
                .max()
                .map(|h| h.try_into().context("Highest bid is too high to store!"))
                .transpose()?;
        },
        BidState::OpenEdition { .. } => {
            total_uncancelled_bids = None;
            highest_bid = None;
        },
    }

    let now = Local::now().naive_utc();
    let (ends_at, ended, last_bid_time) = get_end_info(&auction, now)?;
    let auction_address = bs58::encode(keys.auction).into_string();

    let values = Listing {
        address: Borrowed(&auction_address),
        ends_at,
        created_at: keys.created_at,
        ended,
        authority: Owned(bs58::encode(auction.authority).into_string()),
        token_mint: Owned(bs58::encode(auction.token_mint).into_string()),
        store_owner: Owned(bs58::encode(keys.store_owner).into_string()),
        highest_bid,
        last_bid_time,
        // TODO: horrible abuse of the NaiveDateTime struct but Metaplex does
        //       roughly the same thing with the solana UnixTimestamp struct.
        end_auction_gap: auction
            .end_auction_gap
            .map(|g| NaiveDateTime::from_timestamp(g, 0)),
        price_floor: match auction.price_floor {
            PriceFloor::None(_) => None,
            PriceFloor::MinimumPrice(p) => Some(
                p[0].try_into()
                    .context("Price floor is too high to store")?,
            ),
            PriceFloor::BlindedPrice(_) => Some(-1),
        },
        total_uncancelled_bids,
        gap_tick_size: ext.gap_tick_size_percentage.map(Into::into),
        instant_sale_price: ext
            .instant_sale_price
            .map(TryFrom::try_from)
            .transpose()
            .context("Instant sale price is too high to store")?,
        name: Borrowed(
            ext.name
                .as_ref()
                .map(|n| std::str::from_utf8(n))
                .transpose()
                .context("Couldn't parse auction name")?
                .unwrap_or("")
                .trim_end_matches('\0'),
        ),
    };

    let db = client.db()?;

    insert_into(listings::table)
        .values(&values)
        .on_conflict(listings::address)
        .do_update()
        .set(&values)
        .execute(&db)
        .context("Failed to insert listing")?;

    debug_assert!(!bid_map.read().await.is_empty());

    store_bids(
        &keys.auction,
        &auction_address,
        bid_map
            .read()
            .await
            .get(&keys.auction)
            .into_iter()
            .flatten(),
        &db,
    )?;

    Ok(())
}

pub fn process_solo_bids(client: &Client, auction: Pubkey, bids: BidList) -> Result<()> {
    let db = client.db()?;
    let auction_addr = bs58::encode(auction).into_string();

    if select(exists(
        listings::table.filter(listings::address.eq(&auction_addr)),
    ))
    .get_result(&db)
    .context("Failed to check database for existing auction")?
    {
        store_bids(&auction, &auction_addr, bids, &db)?;
    }

    Ok(())
}

fn store_bids<B: Borrow<BidderMetadata>>(
    auction_key: &Pubkey,
    auction_address: &str,
    bids: impl IntoIterator<Item = B>,
    db: &Connection,
) -> Result<()> {
    debug_assert!(bs58::encode(auction_key).into_string() == auction_address);

    for bid in bids {
        let bid = bid.borrow();

        debug_assert!(&bid.auction_pubkey == auction_key);

        let bid_row = Bid {
            listing_address: Borrowed(auction_address),
            bidder_address: Owned(bs58::encode(bid.bidder_pubkey).into_string()),
            last_bid_time: NaiveDateTime::from_timestamp(bid.last_bid_timestamp, 0),
            last_bid_amount: bid
                .last_bid
                .try_into()
                .context("Last bid amount is too high to store!")?,
            cancelled: bid.cancelled,
        };

        insert_into(bids::table)
            .values(&bid_row)
            .on_conflict((bids::listing_address, bids::bidder_address))
            .do_update()
            .set(&bid_row)
            .execute(db)
            .context("Failed to insert listing bid")?;
    }

    Ok(())
}

/// Returns a tuple of `(ends_at, ended, last_bid_time)`
fn get_end_info(
    auction: &AuctionData,
    now: NaiveDateTime,
) -> Result<(Option<NaiveDateTime>, bool, Option<NaiveDateTime>)> {
    let ends_at = auction
        .ended_at
        .map(|t| NaiveDateTime::from_timestamp(t, 0));

    let gap_time = auction.end_auction_gap.map(Duration::seconds);

    let last_bid_time = auction
        .last_bid
        .map(|l| NaiveDateTime::from_timestamp(l, 0));

    // Based on AuctionData::ended
    let ends_at = match (ends_at, gap_time, last_bid_time) {
        (Some(end), Some(gap), Some(last)) => Some(
            end.max(
                last.checked_add_signed(gap)
                    .ok_or_else(|| anyhow!("Failed to adjust auction end by gap time"))?,
            ),
        ),
        (end, ..) => end,
    };

    let ended = ends_at.map_or(false, |e| now > e);

    Ok((ends_at, ended, last_bid_time))
}
