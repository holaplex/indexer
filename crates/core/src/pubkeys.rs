//! Common pubkey derivations

use std::borrow::Borrow;

use solana_program::{pubkey, pubkey::Pubkey};

/// SOL MINT
pub static SOL: Pubkey = pubkey!("So11111111111111111111111111111111111111112");
/// Metaplex token metadata program key
pub static METADATA: Pubkey = pubkey!("metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s");
/// Metaplex token vault program key
pub static VAULT: Pubkey = pubkey!("vau1zxA2LbssAUEF7Gpw91zMM1LvXrvpzJtmZ58rPsn");
/// Metaplex auction program key
pub static AUCTION: Pubkey = pubkey!("auctxRXPeJoc4817jDhf4HbjnhEcr1cCXenosMhK5R8");
/// ``MagicEden`` program key
pub static ME_ESCROW: Pubkey = pubkey!("MEisE1HzehtrDpAAT8PnLHjpSSkRYakotTuJRPjTpo8");
/// ``MagicEden`` program key 2
pub static ME_HAUS: Pubkey = pubkey!("M2mx93ekt1fmXSVkTrUL9xVFHkmME8HTUi5Cyc5aF7K");
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
/// Cardinal .twitter namespace pubkey
pub static CARDINAL_TWITTER_NAMESPACE: Pubkey =
    pubkey!("2zwXjjGEUrFMyE2CF2Ju4CJwMzwdbBMYnF2boEzgPhGu");
/// `OpenSea` Auction house program pubkey
pub static OPENSEA_AUCTION_HOUSE: Pubkey = pubkey!("3o9d13qUvEuuauhFrVom1vuCzgNsJifeaBYDPquaT73Y");
/// Spl Governance programs pubkey
pub const SPL_GOVERNANCE: [Pubkey; 22] = [
    pubkey!("gUAedF544JeE6NYbQakQvribHykUNgaPJqcgf3UQVnY"),
    pubkey!("GqTPL6qRf5aUuqscLh8Rg2HTxPUXfhhAXDptTLhp1t2J"),
    pubkey!("GovHgfDPyQ1GwazJTDY2avSVY8GGcpmCapmmCsymRaGe"),
    pubkey!("GovER5Lthms3bLBqWub97yVrMmEogzX7xNjdXpPPCVZw"),
    pubkey!("J9uWvULFL47gtCPvgR3oN7W357iehn5WF2Vn9MJvcSxz"),
    pubkey!("JPGov2SBA6f7XSJF5R4Si5jEJekGiyrwP2m7gSEqLUs"),
    pubkey!("5hAykmD4YGcQ7Am3N7nC9kyELq6CThAkU82nhNKDJiCy"),
    pubkey!("gSF1T5PdLc2EutzwAyeExvdW27ySDtFp88ri5Aymah6"),
    pubkey!("AVoAYTs36yB5izAaBkxRG67wL1AMwG3vo41hKtUSb8is"),
    pubkey!("GmtpXy362L8cZfkRmTZMYunWVe8TyRjX5B7sodPZ63LJ"),
    pubkey!("GMpXgTSJt2nJ7zjD1RwbT2QyPhKqD2MjAZuEaLsfPYLF"),
    pubkey!("bqTjmeob6XTdfh12px2fZq4aJMpfSY1R1nHZ44VgVZD"),
    pubkey!("Ghope52FuF6HU3AAhJuAAyS2fiqbVhkAotb7YprL5tdS"),
    pubkey!("5sGZEdn32y8nHax7TxEyoHuPS3UXfPWtisgm8kqxat8H"),
    pubkey!("smfjietFKFJ4Sbw1cqESBTpPhF4CwbMwN8kBEC1e5ui"),
    pubkey!("GMnke6kxYvqoAXgbFGnu84QzvNHoqqTnijWSXYYTFQbB"),
    pubkey!("GCockTxUjxuMdojHiABVZ5NKp6At8eTKDiizbPjiCo4m"),
    pubkey!("HT19EcD68zn7NoCF79b2ucQF8XaMdowyPt5ccS6g1PUx"),
    pubkey!("GRNPT8MPw3LYY6RdjsgKeFji5kMiG1fSxnxDjDBu4s73"),
    pubkey!("ALLGnZikNaJQeN4KCAbDjZRSzvSefUdeTpk18yfizZvT"),
    pubkey!("A7kmu2kUcnQwAVn8B4znQmGJeUrsJ1WEhYVMtmiBLkEr"),
    pubkey!("AEauWRrpn9Cs6GXujzdp1YhMmv2288kBt3SdEcPYEerr"),
];

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
