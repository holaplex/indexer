use crate::{
    db::{insert_into, models::Listing, on_constraint, tables::listings, PooledConnection},
    error::Result,
    prelude::*,
    uuid::Uuid,
};

/// adds a generice listing row to the database
pub fn insert<'a>(db: &PooledConnection, listing: &Listing<'a>) -> Result<Uuid> {
    insert_into(listings::table)
        .values(listing)
        .on_conflict(on_constraint("listings_unique_fields"))
        .do_update()
        .set(listing)
        .returning(listings::id)
        .get_result::<Uuid>(db)
        .map_err(Into::into)
}
