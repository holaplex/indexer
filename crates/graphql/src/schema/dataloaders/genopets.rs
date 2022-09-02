use objects::genopets::{GenoHabitat, GenoRentalAgreement};
use scalars::{markers::TokenMint, PublicKey};
use tables::{geno_habitat_datas, geno_rental_agreements};

use super::prelude::*;

#[async_trait]
impl TryBatchFn<PublicKey<TokenMint>, Option<GenoHabitat>> for Batcher {
    async fn load(
        &mut self,
        addresses: &[PublicKey<TokenMint>],
    ) -> TryBatchMap<PublicKey<TokenMint>, Option<GenoHabitat>> {
        let conn = self.db()?;

        let rows: Vec<models::GenoHabitatData> = geno_habitat_datas::table
            .filter(geno_habitat_datas::habitat_mint.eq(any(addresses)))
            .load(&conn)
            .context("Failed to load Genopets habitats by mint")?;

        Ok(rows
            .into_iter()
            .map(|r| (r.habitat_mint.clone(), r.try_into()))
            .batch(addresses))
    }
}

#[async_trait]
impl TryBatchFn<PublicKey<GenoHabitat>, Option<GenoRentalAgreement>> for Batcher {
    async fn load(
        &mut self,
        addresses: &[PublicKey<GenoHabitat>],
    ) -> TryBatchMap<PublicKey<GenoHabitat>, Option<GenoRentalAgreement>> {
        let conn = self.db()?;

        let rows: Vec<models::GenoRentalAgreement> = geno_rental_agreements::table
            .filter(geno_rental_agreements::habitat_address.eq(any(addresses)))
            .load(&conn)
            .context("Failed to load Genopets rental agreements")?;

        Ok(rows
            .into_iter()
            .map(|r| (r.habitat_address.clone(), r.try_into()))
            .batch(addresses))
    }
}
