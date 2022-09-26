use crate::{
    db::{insert_into, models::Offer, on_constraint, tables::offers, PooledConnection},
    error::Result,
    prelude::*,
    uuid::Uuid,
};

/// adds a generice offer row to the database
pub fn insert<'a>(db: &PooledConnection, offer: &Offer<'a>) -> Result<Uuid> {
    insert_into(offers::table)
        .values(offer)
        .on_conflict(on_constraint("offers_unique_fields"))
        .do_update()
        .set(offer)
        .returning(offers::id)
        .get_result::<Uuid>(db)
        .map_err(Into::into)
}
