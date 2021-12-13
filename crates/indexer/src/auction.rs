use chrono::{offset::Local, NaiveDateTime};
use indexer_core::{
    db::{insert_into, models::Listing, tables::listings},
    prelude::*,
    pubkeys,
    pubkeys::find_auction_data_extended,
};
use metaplex_auction::processor::{
    AuctionData, AuctionDataExtended, BidState, BidderMetadata, PriceFloor, BIDDER_METADATA_LEN,
};

use crate::{client::prelude::*, prelude::*, util, Client, RcAuctionKeys, ThreadPoolHandle};

pub fn process(client: &Client, keys: &RcAuctionKeys, _handle: ThreadPoolHandle) -> Result<()> {
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

    let BidsInfo {
        last_time: last_bid_time,
        count: total_uncancelled_bids,
    } = query_bids(client, &keys.auction, &auction, &ext);

    let total_uncancelled_bids = total_uncancelled_bids
        .map(|c| c.try_into().context("Bid count is too high to store!"))
        .transpose()?;

    // TODO: what timezone is any of this in????
    let row = Listing {
        address: Owned(bs58::encode(keys.auction).into_string()),
        ends_at: auction
            .ended_at
            .map(|t| NaiveDateTime::from_timestamp(t, 0)),
        created_at: keys.created_at,
        ended: auction
            .ended(Local::now().naive_utc().timestamp())
            .context("Failed to check if auction was ended")?,
        authority: Owned(bs58::encode(auction.authority).into_string()),
        token_mint: Owned(bs58::encode(auction.token_mint).into_string()),
        store_owner: Owned(bs58::encode(keys.store_owner).into_string()),
        last_bid: auction.last_bid,
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
        .values(&row)
        .on_conflict(listings::address)
        .do_update()
        .set(&row)
        .execute(&db)
        .context("Failed to insert listing")?;

    Ok(())
}

struct BidsInfo {
    count: Option<usize>,
    last_time: Option<NaiveDateTime>,
}

fn query_bids(
    client: &Client,
    auction_key: &Pubkey,
    auction: &AuctionData,
    ext: &AuctionDataExtended,
) -> BidsInfo {
    // TODO: work-in-progress for proper accounting of bidder metadata.
    //       there's a lot of moving parts so this will have to be resolved as
    //       its own issue

    let last_time =
        if matches!(auction.bid_state, BidState::EnglishAuction { .. }) {
            client
                .get_program_accounts(pubkeys::auction(), RpcProgramAccountsConfig {
                    filters: Some(vec![
                        RpcFilterType::DataSize(BIDDER_METADATA_LEN.try_into().unwrap()),
                        RpcFilterType::Memcmp(Memcmp {
                            offset: 32,
                            bytes: MemcmpEncodedBytes::Bytes(auction_key.to_bytes().into()),
                            encoding: None,
                        }),
                    ]),
                    ..RpcProgramAccountsConfig::default()
                })
                .context("Failed to retrieve bids for auction")
                .and_then(|bids| {
                    bids.into_iter()
                        .map(|(key, mut acct)| {
                            BidderMetadata::from_account_info(&util::account_as_info(
                                &key, false, false, &mut acct,
                            ))
                            .context("Failed to parse bidder metadata")
                        })
                        .try_fold(None::<i64>, |last_time, bid| {
                            bid.map(|bid| {
                                Some(last_time.map_or(bid.last_bid_timestamp, |l| {
                                    l.max(bid.last_bid_timestamp)
                                }))
                            })
                        })
                })
                .map_err(|e| error!("Failed to count bids: {:?}", e))
                .ok()
                .flatten()
                .map(|i| NaiveDateTime::from_timestamp(i, 0))
        } else {
            None
        };

    let count = if ext.instant_sale_price.is_some() {
        None
    } else {
        match auction.bid_state {
            BidState::EnglishAuction { ref bids, .. } => Some(bids.len()),
            BidState::OpenEdition { .. } => None,
        }
    };

    BidsInfo { count, last_time }
}
