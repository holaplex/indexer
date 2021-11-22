use chrono::{offset::Local, NaiveDateTime};
use indexer_core::{
    db::{
        insert_into,
        models::Listing,
        tables::listings::{address, listings},
    },
    prelude::*,
    pubkeys::find_auction_data_extended,
};
use metaplex_auction::processor::{AuctionData, AuctionDataExtended, PriceFloor};

use crate::{util, Client, RcAuctionKeys, ThreadPoolHandle};

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

    // TODO: what timezone is any of this in????
    let addr = keys.auction.to_bytes();
    let auth_addr = auction.authority.to_bytes();
    let mint_addr = auction.token_mint.to_bytes();
    let store_addr = keys.store.to_bytes();
    let row = Listing {
        address: Borrowed(&addr),
        ends_at: auction
            .ended_at
            .map(|t| NaiveDateTime::from_timestamp(t, 0)),
        created_at: keys.created_at,
        ended: auction
            .ended(Local::now().naive_utc().timestamp())
            .context("Failed to check if auction was ended")?,
        authority: Borrowed(&auth_addr),
        token_mint: Borrowed(&mint_addr),
        store: Borrowed(&store_addr),
        last_bid: auction.last_bid,
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
        total_uncancelled_bids: if cfg!(debug_assertions) {
            None
        } else {
            todo!()
        },
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

    insert_into(listings)
        .values(&row)
        .on_conflict(address)
        .do_update()
        .set(&row)
        .execute(&db)
        .context("Failed to insert listing")?;

    Ok(())
}
