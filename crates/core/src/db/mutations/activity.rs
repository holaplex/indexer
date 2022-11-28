use solana_program::pubkey::Pubkey;

use crate::{
    db::{
        custom_types::ActivityTypeEnum,
        insert_into,
        models::{Activity, Listing, Offer, Purchase},
        tables::{collection_mints, marketplace_activities, metadatas},
        PooledConnection,
    },
    error::Result,
    prelude::*,
    pubkeys,
    uuid::Uuid,
};

/// Insert listing activity to `marketplace_activities` table
///
/// # Errors
/// This function fails if the `marketplace_activities` row insert fails

pub fn listing<'a>(
    db: &PooledConnection,
    listing_id: Uuid,
    listing: &Listing<'a>,
    activity_type: ActivityTypeEnum,
) -> Result<()> {
    let auction_house: Pubkey = listing.auction_house.to_string().parse()?;

    if auction_house == pubkeys::OPENSEA_AUCTION_HOUSE {
        return Ok(());
    }

    let collection_id = collection_mints::table
        .inner_join(metadatas::table.on(metadatas::mint_address.eq(collection_mints::mint)))
        .filter(metadatas::address.eq(listing.metadata.clone().to_string()))
        .select(collection_mints::collection_id)
        .first::<String>(db)
        .optional()?;

    let activity = Activity {
        id: listing_id,
        metadata: listing.metadata.clone(),
        price: listing.price,
        auction_house: listing.auction_house.clone(),
        created_at: Local::now().naive_utc(),
        marketplace_program: listing.marketplace_program.clone(),
        buyer: None,
        seller: Some(listing.seller.clone()),
        collection_id: collection_id.map(Into::into),
        activity_type,
    };

    insert_into(marketplace_activities::table)
        .values(activity)
        .execute(db)?;

    Ok(())
}

/// Insert offer activity to `marketplace_activities` table
///
/// # Errors
/// This function fails if the `marketplace_activities` row insert fails

pub fn offer<'a>(
    db: &PooledConnection,
    offer_id: Uuid,
    offer: &Offer<'a>,
    activity_type: ActivityTypeEnum,
) -> Result<()> {
    let auction_house: Pubkey = offer.auction_house.to_string().parse()?;

    if auction_house == pubkeys::OPENSEA_AUCTION_HOUSE {
        return Ok(());
    }

    let collection_id = collection_mints::table
        .inner_join(metadatas::table.on(metadatas::mint_address.eq(collection_mints::mint)))
        .filter(metadatas::address.eq(offer.metadata.clone().to_string()))
        .select(collection_mints::collection_id)
        .first::<String>(db)
        .optional()?;

    let activity = Activity {
        id: offer_id,
        metadata: offer.metadata.clone(),
        price: offer.price,
        auction_house: offer.auction_house.clone(),
        created_at: Local::now().naive_utc(),
        marketplace_program: offer.marketplace_program.clone(),
        buyer: Some(offer.buyer.clone()),
        seller: None,
        collection_id: collection_id.map(Into::into),
        activity_type,
    };

    insert_into(marketplace_activities::table)
        .values(activity)
        .execute(db)?;

    Ok(())
}

/// Insert purchase activity to `marketplace_activities` table
///
/// # Errors
/// This function fails if the `marketplace_activities` row insert fails

pub fn purchase<'a>(
    db: &PooledConnection,
    purchase_id: Uuid,
    purchase: &Purchase<'a>,
    activity_type: ActivityTypeEnum,
) -> Result<()> {
    let collection_id = collection_mints::table
        .inner_join(metadatas::table.on(metadatas::mint_address.eq(collection_mints::mint)))
        .filter(metadatas::address.eq(purchase.metadata.clone().to_string()))
        .select(collection_mints::collection_id)
        .first::<String>(db)
        .optional()?;

    let activity = Activity {
        id: purchase_id,
        metadata: purchase.metadata.clone(),
        price: purchase.price,
        auction_house: purchase.auction_house.clone(),
        created_at: Local::now().naive_utc(),
        marketplace_program: purchase.marketplace_program.clone(),
        buyer: Some(purchase.buyer.clone()),
        seller: Some(purchase.seller.clone()),
        collection_id: collection_id.map(Into::into),
        activity_type,
    };

    insert_into(marketplace_activities::table)
        .values(activity)
        .execute(db)?;

    Ok(())
}
