//! Common pubkey derivations

use std::borrow::Borrow;

use solana_sdk::pubkey::Pubkey;

mod ids {
    #![allow(missing_docs)]

    use solana_sdk::pubkeys;

    pubkeys!(metadata, "metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s");
    pubkeys!(vault, "vau1zxA2LbssAUEF7Gpw91zMM1LvXrvpzJtmZ58rPsn");
    pubkeys!(auction, "auctxRXPeJoc4817jDhf4HbjnhEcr1cCXenosMhK5R8");
    pubkeys!(metaplex, "p1exdMJcjVao65QdewkaZRUnU6VPSXhus9n2GzWfh98");
}

/// Find the address of a store given its owner's address
pub fn find_store_address(owner: impl Borrow<Pubkey>) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[
            "metaplex".as_bytes(),
            &ids::metaplex().to_bytes(),
            &owner.borrow().to_bytes(),
        ],
        &ids::metaplex(),
    )
}

/// Find the address of a store indexer page, given the store's address and a
/// page number
pub fn find_store_indexer(store: impl Borrow<Pubkey>, index: u64) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[
            "metaplex".as_bytes(),
            &ids::metaplex().to_bytes(),
            &store.borrow().to_bytes(),
            "index".as_bytes(),
            format!("{}", index).as_bytes(),
        ],
        &ids::metaplex(),
    )
}

/// Find the address of an `AuctionDataExtended` account, given the auction vault
pub fn find_auction_data_extended(vault: impl Borrow<Pubkey>) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[
            "auction".as_bytes(),
            &ids::auction().to_bytes(),
            &vault.borrow().to_bytes(),
            "extended".as_bytes(),
        ],
        &ids::auction(),
    )
}
