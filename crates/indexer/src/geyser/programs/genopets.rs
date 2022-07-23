use anchor_lang_v0_24::{AccountDeserialize, Discriminator};
use genostub::state::HabitatData;

use super::{accounts::geno_habitat_data, AccountUpdate, Client};
use crate::prelude::*;

pub(crate) async fn process(client: &Client, update: AccountUpdate) -> Result<()> {
    let discrim = &update.data[..8];

    if discrim == HabitatData::discriminator() {
        let habitat = HabitatData::try_deserialize(&mut &*update.data)
            .context("Failed to deserialize HabitatData")?;

        geno_habitat_data::process(
            client,
            update.key,
            habitat,
            update.slot,
            update.write_version,
        )
        .await?;
    }

    Ok(())
}
