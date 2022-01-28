//! Common pubkey derivations

use std::{borrow::Borrow, os::unix::prelude::OsStrExt};

use solana_sdk::pubkey::Pubkey;

mod ids {
    #![allow(missing_docs)]

    use solana_sdk::pubkeys;

    pubkeys!(metadata, "metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s");
    pubkeys!(vault, "vau1zxA2LbssAUEF7Gpw91zMM1LvXrvpzJtmZ58rPsn");
    pubkeys!(auction, "auctxRXPeJoc4817jDhf4HbjnhEcr1cCXenosMhK5R8");
    pubkeys!(metaplex, "p1exdMJcjVao65QdewkaZRUnU6VPSXhus9n2GzWfh98");
    pubkeys!(
        spl_name_service,
        "namesLPneVptA9Z5rqUDD9tMTWEJwofgaYwp8cawRkX"
    );
    pubkeys!(
        twitter_root_name_service,
        "4YcexoW3r78zz16J2aqmukBLRwGq6rAvWzJpkYAXqebv"
    );
    pubkeys!(
        twitter_verification_authority,
        "FvPH7PrVrLGKPfqaf3xJodFTjZriqrAXXLTVWEorTFBi"
    );
}

pub use ids::{
    auction, metadata, metaplex, spl_name_service, twitter_root_name_service,
    twitter_verification_authority, vault,
};

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

/// Find the address of a twitter account from user wallet address
pub fn find_twitter_handle_address(pubkey: &str) -> (Pubkey, Vec<u8>) {
    let hashed_name = solana_sdk::hash::hashv(&[(spl_name_service::state::HASH_PREFIX.to_owned()
        + pubkey)
        .as_bytes()])
    .as_ref()
    .to_vec();

    spl_name_service::state::get_seeds_and_key(
        &ids::spl_name_service(),
        hashed_name,
        Some(&ids::twitter_verification_authority()),
        Some(&ids::twitter_root_name_service()),
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

/// Find the address of a `BidderMetadata` account, given the auction and bidder
pub fn find_bidder_metadata(
    auction: impl Borrow<Pubkey>,
    bidder: impl Borrow<Pubkey>,
) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[
            "auction".as_bytes(),
            &ids::auction().to_bytes(),
            &auction.borrow().to_bytes(),
            &bidder.borrow().to_bytes(),
            "metadata".as_bytes(),
        ],
        &ids::auction(),
    )
}

/// Find the address of an `Edition` account, given the token mint
pub fn find_edition(mint: impl Borrow<Pubkey>) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[
            "metadata".as_bytes(),
            &ids::metadata().to_bytes(),
            &mint.borrow().to_bytes(),
            "edition".as_bytes(),
        ],
        &ids::metadata(),
    )
}
