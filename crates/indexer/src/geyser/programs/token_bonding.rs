use anchor_lang_v0_22_1::AccountDeserialize;
use spl_token_bonding::state::TokenBondingV0;

use super::{accounts::bonding_change, AccountUpdate, Client};
use crate::prelude::*;

pub(crate) async fn process(client: &Client, update: AccountUpdate) -> Result<()> {
    if let Ok(token_bonding) =
        TokenBondingV0::try_deserialize_unchecked(&mut update.data.as_slice())
    {
        bonding_change::process_token_bonding(
            client,
            update.key,
            i64::try_from(update.slot)?,
            token_bonding,
        )
        .await?;
    }
    Ok(())
}
