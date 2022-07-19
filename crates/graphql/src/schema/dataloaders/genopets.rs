use objects::genopets::{GenoHabitat, GenoRentalAgreement};
use scalars::PublicKey;
use tables::geno_rental_agreements;

use super::prelude::*;

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
