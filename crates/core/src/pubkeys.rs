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
    pubkeys!(token, "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA");
    pubkeys!(auction_house, "hausS13jsjafwWwGqZTUQRmWyvyxn9EQpqMwV1PBBmk");
    pubkeys!(candy_machine, "cndy3Z4yapfJBmL3ShUp5exZKqR3z33thTzeNMm2gRZ");
    pubkeys!(graph_program, "grphSXQnjAoPXSG5p1aJ7ZFw2A1akqP3pkXvjfbSJef");
    pubkeys!(name_service, "namesLPneVptA9Z5rqUDD9tMTWEJwofgaYwp8cawRkX");
    pubkeys!(
        cardinal_token_manager,
        "mgr99QFMYByTqGPWmNqunV7vBLmWWXdSrHUfV8Jf3JM"
    );
    pubkeys!(
        cardinal_time_invalidator,
        "tmeEDp1RgoDtZFtx6qod3HkbQmv9LMe36uqKVvsLTDE"
    );
    pubkeys!(
        cardinal_use_invalidator,
        "useZ65tbyvWpdYCLDJaegGK34Lnsi8S3jZdwx8122qp"
    );
    pubkeys!(
        cardinal_paid_claim_approver,
        "pcaBwhJ1YHp7UDA7HASpQsRUmUNwzgYaLQto2kSj1fR"
    );
    pubkeys!(
        goki_smart_wallet,
        "GokivDYuQXPZCWRkwMhdH2h91KpDQXBEmpgBgs55bnpH"
    );
    pubkeys!(
        tribeca_locked_voter,
        "LocktDzaV1W2Bm9DeZeiyz4J9zs4fRqNiYqQyracRXw"
    );
    pubkeys!(
        tribeca_govern,
        "Govz1VyoyLD5BL6CSCxUJLVLsQHRwjfFj1prNsdNg5Jw"
    );
    pubkeys!(token_bonding, "TBondmkCYxaPCKG4CHYfVTcwQ8on31xnJrPzk8F8WsS");
}

pub use ids::{
    auction, auction_house, candy_machine, cardinal_paid_claim_approver, cardinal_time_invalidator,
    cardinal_token_manager, cardinal_use_invalidator, goki_smart_wallet, graph_program, metadata,
    metaplex, name_service, token, token_bonding, tribeca_govern, tribeca_locked_voter, vault,
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

/// find the address of an ``StoreConfig`` account given the store address
pub fn find_store_config(store: impl Borrow<Pubkey>) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[
            "metaplex".as_bytes(),
            &ids::metaplex().to_bytes(),
            "config".as_bytes(),
            &store.borrow().to_bytes(),
        ],
        &ids::metaplex(),
    )
}
