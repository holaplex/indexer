use std::borrow::Borrow;

use solana_sdk::{pubkey::Pubkey, pubkeys};

pubkeys!(metadata_id, "metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s");
pubkeys!(vault_id, "vau1zxA2LbssAUEF7Gpw91zMM1LvXrvpzJtmZ58rPsn");
pubkeys!(auction_id, "auctxRXPeJoc4817jDhf4HbjnhEcr1cCXenosMhK5R8");
pubkeys!(metaplex_id, "p1exdMJcjVao65QdewkaZRUnU6VPSXhus9n2GzWfh98");

pub fn find_store_address(owner: impl Borrow<Pubkey>) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[
            "metaplex".as_bytes(),
            &metaplex_id().to_bytes(),
            &owner.borrow().to_bytes(),
        ],
        &metaplex_id(),
    )
}

pub fn find_store_indexer(store: impl Borrow<Pubkey>, index: u64) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[
            "metaplex".as_bytes(),
            &metaplex_id().to_bytes(),
            &store.borrow().to_bytes(),
            "index".as_bytes(),
            format!("{}", index).as_bytes(),
        ],
        &metaplex_id(),
    )
}
