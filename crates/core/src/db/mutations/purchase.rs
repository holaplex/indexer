use crate::{
    db::{insert_into, models::Purchase, on_constraint, tables::purchases, PooledConnection},
    error::Result,
    prelude::*,
    uuid::Uuid,
};

/// Insert generic purchase to purchases table
///
/// # Errors
/// This function fails if the offer row upsert fails
pub fn insert<'a>(db: &PooledConnection, purchase: &Purchase<'a>) -> Result<Uuid> {
    insert_into(purchases::table)
        .values(purchase)
        .on_conflict(on_constraint("purchases_unique_fields"))
        .do_update()
        .set(purchase)
        .returning(purchases::id)
        .get_result::<Uuid>(db)
        .map_err(Into::into)
}
