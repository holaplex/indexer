use borsh::BorshDeserialize;
use solana_program::pubkey;

use super::{accounts::name_service, AccountUpdate, Client};
use crate::prelude::*;

const HEADER_LENGTH: usize = 96;

static TWITTER_VERIFICATION_AUTHORITY: Pubkey =
    pubkey!("FvPH7PrVrLGKPfqaf3xJodFTjZriqrAXXLTVWEorTFBi");
static TWITTER_ROOT_PARENT_REGISTRY_KEY: Pubkey =
    pubkey!("4YcexoW3r78zz16J2aqmukBLRwGq6rAvWzJpkYAXqebv");

#[derive(BorshDeserialize, PartialEq, Debug, Clone)]
struct Header {
    parent: [u8; 32],
    owner: [u8; 32],
    class: [u8; 32],
}

pub(crate) async fn process(client: &Client, update: AccountUpdate) -> Result<()> {
    if update.data.len() <= HEADER_LENGTH {
        return Ok(());
    }

    let header_bytes: [u8; HEADER_LENGTH] = update.data[..HEADER_LENGTH].try_into()?;

    let header = Header::try_from_slice(header_bytes.as_slice())
        .context("failed to deserialize header data")?;

    let parent = Pubkey::new(header.parent.as_slice());
    let class = Pubkey::new(header.class.as_slice());

    if parent != TWITTER_ROOT_PARENT_REGISTRY_KEY || class != TWITTER_VERIFICATION_AUTHORITY {
        return Ok(());
    }

    let wallet = Pubkey::new(header.owner.as_slice());

    let data: Vec<u8> = update.data[HEADER_LENGTH..].to_vec();

    name_service::process(
        client,
        update.key,
        update.slot,
        update.write_version,
        wallet,
        data,
    )
    .await
}
