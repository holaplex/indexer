use crate::{
    db::{
        insert_into,
        models::{CollectionActivity, Listing, Offer, Purchase},
        tables::{collection_activities, collection_mints, metadatas},
        PooledConnection,
    },
    error::Result,
    prelude::*,
    uuid::Uuid,
};

/// Insert listing activity to `collection_activities` table
///
/// # Errors
/// This function fails if the `collection_activities` row insert fails

pub fn listing<'a>(
    db: &PooledConnection,
    listing_id: Uuid,
    listing: &Listing<'a>,
    event: &str,
) -> Result<()> {
    let collection_id = collection_mints::table
        .inner_join(metadatas::table.on(metadatas::mint_address.eq(collection_mints::mint)))
        .filter(metadatas::address.eq(listing.metadata.clone().to_string()))
        .select(collection_mints::collection_id)
        .first::<String>(db)?;

    let activity = CollectionActivity {
        id: listing_id,
        metadata: listing.metadata.clone(),
        price: listing.price,
        auction_house: listing.auction_house.clone(),
        created_at: listing.created_at,
        marketplace_program: listing.marketplace_program.clone(),
        wallets: vec![listing.seller.clone()],
        collection_id: collection_id.into(),
        activity_type: Owned(event.to_string()),
    };

    insert_into(collection_activities::table)
        .values(activity)
        .execute(db)?;

    Ok(())
}

/// Insert offer activity to `collection_activities` table
///
/// # Errors
/// This function fails if the `collection_activities` row insert fails

pub fn offer<'a>(
    db: &PooledConnection,
    offer_id: Uuid,
    offer: &Offer<'a>,
    event: &str,
) -> Result<()> {
    let collection_id = collection_mints::table
        .inner_join(metadatas::table.on(metadatas::mint_address.eq(collection_mints::mint)))
        .filter(metadatas::address.eq(offer.metadata.clone().to_string()))
        .select(collection_mints::collection_id)
        .first::<String>(db)?;

    let activity = CollectionActivity {
        id: offer_id,
        metadata: offer.metadata.clone(),
        price: offer.price,
        auction_house: offer.auction_house.clone(),
        created_at: offer.created_at,
        marketplace_program: offer.marketplace_program.clone(),
        wallets: vec![offer.buyer.clone()],
        collection_id: collection_id.into(),
        activity_type: Owned(event.to_string()),
    };

    insert_into(collection_activities::table)
        .values(activity)
        .execute(db)?;

    Ok(())
}

/// Insert purchase activity to `collection_activities` table
///
/// # Errors
/// This function fails if the `collection_activities` row insert fails

pub fn purchase<'a>(
    db: &PooledConnection,
    purchase_id: Uuid,
    purchase: &Purchase<'a>,
    event: &str,
) -> Result<()> {
    let collection_id = collection_mints::table
        .inner_join(metadatas::table.on(metadatas::mint_address.eq(collection_mints::mint)))
        .filter(metadatas::address.eq(purchase.metadata.clone().to_string()))
        .select(collection_mints::collection_id)
        .first::<String>(db)?;

    let activity = CollectionActivity {
        id: purchase_id,
        metadata: purchase.metadata.clone(),
        price: purchase.price,
        auction_house: purchase.auction_house.clone(),
        created_at: purchase.created_at,
        marketplace_program: purchase.marketplace_program.clone(),
        wallets: vec![purchase.seller.clone(), purchase.buyer.clone()],
        collection_id: collection_id.into(),
        activity_type: Owned(event.to_string()),
    };

    insert_into(collection_activities::table)
        .values(activity)
        .execute(db)?;

    Ok(())
}
