use indexer_core::db::{insert_into, models::Bid, tables::bids};
use metaplex_auction::processor::BidderMetadata as BidderMetadataAccount;

use super::Client;
use crate::prelude::*;

pub(crate) async fn process(
    client: &Client,
    _key: Pubkey,
    meta: BidderMetadataAccount,
) -> Result<()> {
    let BidderMetadataAccount {
        bidder_pubkey,
        auction_pubkey,
        last_bid,
        last_bid_timestamp,
        cancelled,
    } = meta;

    let values = Bid {
        listing_address: Owned(bs58::encode(auction_pubkey).into_string()),
        bidder_address: Owned(bs58::encode(bidder_pubkey).into_string()),
        last_bid_time: NaiveDateTime::from_timestamp(last_bid_timestamp, 0),
        last_bid_amount: last_bid
            .try_into()
            .context("Last bid amount was too high to store")?,
        cancelled,
    };

    client
        .db()
        .run(move |db| {
            insert_into(bids::table)
                .values(&values)
                .on_conflict((bids::listing_address, bids::bidder_address))
                .do_update()
                .set(&values)
                .execute(db)
        })
        .await
        .context("Failed to store bidder metadata")?;

    Ok(())
}
