use indexer_core::pubkeys;
use metaplex_auction::processor::{
    AuctionData, AuctionDataExtended, BidderMetadata, BASE_AUCTION_DATA_SIZE,
};

use super::{
    accounts::{auction_data, bidder_metadata},
    AccountUpdate, Client,
};
use crate::{prelude::*, util};

pub(crate) async fn process(client: &Client, update: AccountUpdate) -> Result<()> {
    let accounts = util::account_data_as_info(
        update.key,
        update.data,
        pubkeys::AUCTION,
        0,
        |account_info| {
            let auction = if account_info.data_len() >= BASE_AUCTION_DATA_SIZE {
                AuctionData::from_account_info(&account_info).map_err(Into::into)
            } else {
                // TODO: this is a bug in the Metaplex code
                Err(anyhow!("Data length shorter than BASE_AUCTION_DATA_SIZE"))
            };
            let ext = AuctionDataExtended::from_account_info(&account_info);
            let bidder = BidderMetadata::from_account_info(&account_info);

            (auction, ext, bidder)
        },
    )
    .await?;

    match accounts {
        (Ok(a), Err(_), Err(_)) => auction_data::process(client, update.key, a).await,
        (Err(_), Ok(e), Err(_)) => auction_data::process_extended(client, update.key, e).await,
        (Err(_), Err(_), Ok(b)) => bidder_metadata::process(client, update.key, b).await,
        (Err(_), Err(_), Err(_)) => {
            debug!(
                "Account at {} was not AuctionData(Extended) or BidderMetadata",
                update.key
            );
            Ok(())
        },
        _ => Err(anyhow!(
            "Found ambiguous metaplex auction account at {}",
            update.key
        )),
    }
}
