use chrono::{offset::Local, Duration, NaiveDateTime};
use indexer_core::{
    db::{
        insert_into,
        models::{AuctionData, AuctionDataExt},
        tables::{auction_datas, auction_datas_ext},
    },
    prelude::*,
};
use metaplex_auction::processor::{
    AuctionData as AuctionDataAccount, AuctionDataExtended, BidState, PriceFloor,
    BASE_AUCTION_DATA_SIZE,
};

use crate::{prelude::*, util, Client};

async fn process_auction_data(
    client: &Client,
    key: Pubkey,
    auction: AuctionDataAccount,
) -> Result<()> {
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
    let (ends_at, _ended, last_bid_time) = get_end_info(&auction, now)?;

    let values = AuctionData {
        address: Owned(bs58::encode(key).into_string()),
        ends_at,
        authority: Owned(bs58::encode(auction.authority).into_string()),
        token_mint: Owned(bs58::encode(auction.token_mint).into_string()),
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
    };

    client
        .db(move |db| {
            insert_into(auction_datas::table)
                .values(&values)
                .on_conflict(auction_datas::address)
                .do_update()
                .set(&values)
                .execute(db)
        })
        .await
        .context("Failed to insert AuctionData")?;

    Ok(())
}

async fn process_auction_data_extended(
    client: &Client,
    key: Pubkey,
    ext: AuctionDataExtended,
) -> Result<()> {
    let values = AuctionDataExt {
        address: Owned(bs58::encode(key).into_string()),
        gap_tick_size: ext.gap_tick_size_percentage.map(Into::into),
        instant_sale_price: ext
            .instant_sale_price
            .map(TryFrom::try_from)
            .transpose()
            .context("Instant sale price is too high to store")?,
        name: Owned(
            ext.name
                .as_ref()
                .map(|n| std::str::from_utf8(n))
                .transpose()
                .context("Couldn't parse auction name")?
                .unwrap_or("")
                .trim_end_matches('\0')
                .to_owned(),
        ),
    };

    client
        .db(move |db| {
            insert_into(auction_datas_ext::table)
                .values(&values)
                .on_conflict(auction_datas_ext::address)
                .do_update()
                .set(&values)
                .execute(db)
        })
        .await
        .context("Failed to insert AuctionDataExtended")?;

    Ok(())
}

pub(super) async fn process(
    client: &Client,
    key: Pubkey,
    mut data: Vec<u8>,
    owner: Pubkey,
) -> Result<()> {
    info!("starting new auction processing");
    let mut zero_lamports = 0_u64;

    let account_info = util::account_data_as_info(&key, &mut data, &owner, &mut zero_lamports);

    let auction = if account_info.data_len() >= BASE_AUCTION_DATA_SIZE {
        AuctionDataAccount::from_account_info(&account_info).map_err(Into::into)
    } else {
        // TODO: this is a bug in the Metaplex code
        Err(anyhow!("data length shorter than BASE_AUCTION_DATA_SIZE"))
    };
    let ext = AuctionDataExtended::from_account_info(&account_info);

    match (auction, ext) {
        (Ok(_), Ok(_)) => Err(anyhow!(
            "Found ambiguous AuctionData(Extended) account at {}",
            key
        )),
        (Ok(a), Err(_)) => process_auction_data(client, key, a).await,
        (Err(_), Ok(e)) => process_auction_data_extended(client, key, e).await,
        (Err(_), Err(_)) => {
            debug!("Account at {} was not AuctionData(Extended)", key);
            Ok(())
        },
    }
}

// TODO: handle bids
// pub fn process_solo_bids(client: &Client, auction: Pubkey, bids: BidList) -> Result<()> {
//     let db = client.db()?;
//     let auction_addr = bs58::encode(auction).into_string();

//     if select(exists(
//         listings::table.filter(listings::address.eq(&auction_addr)),
//     ))
//     .get_result(&db)
//     .context("Failed to check database for existing auction")?
//     {
//         store_bids(&auction, &auction_addr, bids, &db)?;
//     }

//     Ok(())
// }

// fn store_bids<B: Borrow<BidderMetadata>>(
//     auction_key: &Pubkey,
//     auction_address: &str,
//     bids: impl IntoIterator<Item = B>,
//     db: &Connection,
// ) -> Result<()> {
//     debug_assert!(bs58::encode(auction_key).into_string() == auction_address);

//     for bid in bids {
//         let bid = bid.borrow();

//         debug_assert!(&bid.auction_pubkey == auction_key);

//         let bid_row = Bid {
//             listing_address: Borrowed(auction_address),
//             bidder_address: Owned(bs58::encode(bid.bidder_pubkey).into_string()),
//             last_bid_time: NaiveDateTime::from_timestamp(bid.last_bid_timestamp, 0),
//             last_bid_amount: bid
//                 .last_bid
//                 .try_into()
//                 .context("Last bid amount is too high to store!")?,
//             cancelled: bid.cancelled,
//         };

//         insert_into(bids::table)
//             .values(&bid_row)
//             .on_conflict((bids::listing_address, bids::bidder_address))
//             .do_update()
//             .set(&bid_row)
//             .execute(db)
//             .context("Failed to insert listing bid")?;
//     }

//     Ok(())
// }

/// Returns a tuple of `(ends_at, ended, last_bid_time)`
fn get_end_info(
    auction: &AuctionDataAccount,
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
