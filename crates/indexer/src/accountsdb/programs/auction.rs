use indexer_core::pubkeys;
use metaplex_auction::processor::{
    AuctionData as AuctionDataAccount, AuctionDataExtended, BASE_AUCTION_DATA_SIZE,
};

use super::{accounts::auction_data, Client};
use crate::{prelude::*, util};

pub(crate) async fn process(client: &Client, key: Pubkey, mut data: Vec<u8>) -> Result<()> {
    let mut zero_lamports = 0_u64;
    let owner = pubkeys::auction();
    let account_info = util::account_data_as_info(&key, &mut data, &owner, &mut zero_lamports);

    let auction = if account_info.data_len() >= BASE_AUCTION_DATA_SIZE {
        AuctionDataAccount::from_account_info(&account_info).map_err(Into::into)
    } else {
        // TODO: this is a bug in the Metaplex code
        Err(anyhow!("Data length shorter than BASE_AUCTION_DATA_SIZE"))
    };
    let ext = AuctionDataExtended::from_account_info(&account_info);

    match (auction, ext) {
        (Ok(_), Ok(_)) => Err(anyhow!(
            "Found ambiguous AuctionData(Extended) account at {}",
            key
        )),
        (Ok(a), Err(_)) => auction_data::process(client, key, a).await,
        (Err(_), Ok(e)) => auction_data::process_extended(client, key, e).await,
        (Err(_), Err(_)) => Ok(()),
    }
}
