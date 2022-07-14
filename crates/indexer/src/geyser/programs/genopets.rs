use genostub::state::HabitatData;

use super::{accounts::geno_habitat_data, AccountUpdate, Client};
use crate::prelude::*;

pub(crate) async fn process(client: &Client, update: AccountUpdate) -> Result<()> {
    let habitat: HabitatData = todo!();

    geno_habitat_data::process(
        client,
        update.key,
        habitat,
        update.slot,
        update.write_version,
    )
    .await
}
