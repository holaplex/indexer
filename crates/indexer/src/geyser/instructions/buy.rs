use borsh::BorshDeserialize;
use indexer_core::db::{insert_into, models::BidReceipt, tables::bid_receipts};

use super::Client;
use crate::prelude::*;

// Buy instruction accounts

// #1 Wallet
// #2 - Payment Account
// #3 - Transfer Authority
// #4 - Treasury Mint
// #5 - Token Account
// #6 - Metadata
// #7 - Escrow Payment Account
// #8 - Authority
// #9 - Auction House
// #10 - Auction House Fee Account
// #11 - Buyer Trade State
// #12 - Token Program
// #13 - System Program

#[derive(BorshDeserialize, Debug, Clone)]
pub struct InstructionParameters {
    trade_state_bump: u8,
    escrow_payment_bump: u8,
    buyer_price: u64,
    token_size: u64,
}

pub(crate) fn process(client: &Client, data: &[u8], accounts: &[Pubkey]) -> Result<()> {
    let params = InstructionParameters::try_from_slice(data)
        .context("failed to deserialize")?;
    //BuyInstructionParameters { trade_state_bump: 254, escrow_payment_bump: 254, buyer_price: 2750000000, token_size: 1 }

    let rows = BidReceipt {
        address: Owned(String::default()),
        trade_state: Owned(
            accounts
                .get(10)
                .context("failed to get trade state pubkey")?
                .to_string(),
        ),
        bookkeeper: Owned(
            accounts
                .get(0)
                .context("failed to get bookkeeper pubkey")?
                .to_string(),
        ),
        auction_house: Owned(
            accounts
                .get(8)
                .context("failed to get auction_house pubkey")?
                .to_string(),
        ),
        buyer: Owned(
            accounts
                .get(0)
                .context("failed to get wallet pubkey")?
                .to_string(),
        ),
        metadata: Owned(
            accounts
                .get(5)
                .context("failed to get metadata pubkey")?
                .to_string(),
        ),
        token_account: accounts
            .get(4)
            .map(|t| Owned(t.to_string())),
            
        purchase_receipt: None,
        price: params.buyer_price.try_into()?,
        token_size: params.token_size.try_into()?,
        bump: 0, //Make it optional,
        trade_state_bump: params.trade_state_bump.try_into()?,
        created_at: Utc::now().naive_utc(),
        canceled_at: None,
    };

    dbg!("{:?}", rows);
    Ok(())
}
