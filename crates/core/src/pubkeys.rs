//! Common pubkey derivations

use std::borrow::Borrow;

use solana_program::{pubkey, pubkey::Pubkey};

/// Metaplex token metadata program key
pub static METADATA: Pubkey = pubkey!("metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s");
/// Metaplex token vault program key
pub static VAULT: Pubkey = pubkey!("vau1zxA2LbssAUEF7Gpw91zMM1LvXrvpzJtmZ58rPsn");
/// Metaplex auction program key
pub static AUCTION: Pubkey = pubkey!("auctxRXPeJoc4817jDhf4HbjnhEcr1cCXenosMhK5R8");
/// Metaplex auction processing program key
pub static METAPLEX: Pubkey = pubkey!("p1exdMJcjVao65QdewkaZRUnU6VPSXhus9n2GzWfh98");
/// SPL token program key
pub static TOKEN: Pubkey = pubkey!("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA");
/// MPL auction house program key
pub static AUCTION_HOUSE: Pubkey = pubkey!("hausS13jsjafwWwGqZTUQRmWyvyxn9EQpqMwV1PBBmk");
/// Metaplex candy machine program key
pub static CANDY_MACHINE: Pubkey = pubkey!("cndy3Z4yapfJBmL3ShUp5exZKqR3z33thTzeNMm2gRZ");
/// HPL graph program key
pub static GRAPH_PROGRAM: Pubkey = pubkey!("grphAFGNvCjLKHeEmPNa91eGJChcUhrdaYYharcZCTQ");
/// SPL name service program key
pub static NAME_SERVICE: Pubkey = pubkey!("namesLPneVptA9Z5rqUDD9tMTWEJwofgaYwp8cawRkX");
/// Cardinal token manager program key
pub static CARDINAL_TOKEN_MANAGER: Pubkey = pubkey!("mgr99QFMYByTqGPWmNqunV7vBLmWWXdSrHUfV8Jf3JM");
/// Cardinal time-driven invalidator program key
pub static CARDINAL_TIME_INVALIDATOR: Pubkey =
    pubkey!("tmeEDp1RgoDtZFtx6qod3HkbQmv9LMe36uqKVvsLTDE");
/// Cardinal use-driven invalidator program key
pub static CARDINAL_USE_INVALIDATOR: Pubkey =
    pubkey!("useZ65tbyvWpdYCLDJaegGK34Lnsi8S3jZdwx8122qp");
/// Cardinal paid claim approver program key
pub static CARDINAL_PAID_CLAIM_APPROVER: Pubkey =
    pubkey!("pcaBwhJ1YHp7UDA7HASpQsRUmUNwzgYaLQto2kSj1fR");
/// Cardinal namespaces program key
pub static NAMESPACES: Pubkey = pubkey!("nameXpT2PwZ2iA6DTNYTotTmiMYusBCYqwBLN2QgF4w");
/// Goki smart wallet program key
pub static GOKI_SMART_WALLET: Pubkey = pubkey!("GokivDYuQXPZCWRkwMhdH2h91KpDQXBEmpgBgs55bnpH");
/// Tribeca locked voter program key
pub static TRIBECA_LOCKED_VOTER: Pubkey = pubkey!("LocktDzaV1W2Bm9DeZeiyz4J9zs4fRqNiYqQyracRXw");
/// Tribeca governance program key
pub static TRIBECA_GOVERN: Pubkey = pubkey!("Govz1VyoyLD5BL6CSCxUJLVLsQHRwjfFj1prNsdNg5Jw");
/// Strata token bonding program key
pub static TOKEN_BONDING: Pubkey = pubkey!("TBondmkCYxaPCKG4CHYfVTcwQ8on31xnJrPzk8F8WsS");

/// Find the address of a store given its owner's address
pub fn find_store_address(owner: impl Borrow<Pubkey>) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[
            "metaplex".as_bytes(),
            &METAPLEX.to_bytes(),
            &owner.borrow().to_bytes(),
        ],
        &METAPLEX,
    )
}

/// Find the address of a store indexer page, given the store's address and a
/// page number
pub fn find_store_indexer(store: impl Borrow<Pubkey>, index: u64) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[
            "metaplex".as_bytes(),
            &METAPLEX.to_bytes(),
            &store.borrow().to_bytes(),
            "index".as_bytes(),
            format!("{}", index).as_bytes(),
        ],
        &METAPLEX,
    )
}

/// Find the address of an `AuctionDataExtended` account, given the auction vault
pub fn find_auction_data_extended(vault: impl Borrow<Pubkey>) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[
            "auction".as_bytes(),
            &AUCTION.to_bytes(),
            &vault.borrow().to_bytes(),
            "extended".as_bytes(),
        ],
        &AUCTION,
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
            &AUCTION.to_bytes(),
            &auction.borrow().to_bytes(),
            &bidder.borrow().to_bytes(),
            "metadata".as_bytes(),
        ],
        &AUCTION,
    )
}

/// Find the address of an `Edition` account, given the token mint
pub fn find_edition(mint: impl Borrow<Pubkey>) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[
            "metadata".as_bytes(),
            &METADATA.to_bytes(),
            &mint.borrow().to_bytes(),
            "edition".as_bytes(),
        ],
        &METADATA,
    )
}

/// find the address of an ``StoreConfig`` account given the store address
pub fn find_store_config(store: impl Borrow<Pubkey>) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[
            "metaplex".as_bytes(),
            &METAPLEX.to_bytes(),
            "config".as_bytes(),
            &store.borrow().to_bytes(),
        ],
        &METAPLEX,
    )
}
