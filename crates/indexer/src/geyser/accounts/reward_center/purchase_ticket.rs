use indexer_core::{
    db::{
        insert_into,
        models::{
            AuctionHouse, CurrentMetadataOwner, Purchase as DbPurchase,
            RewardsPurchaseTicket as DbRewardsPurchaseTicket,
        },
        mutations,
        tables::{
            current_metadata_owners, listings, metadatas, offers, reward_centers, rewards_listings,
        },
    },
    prelude::*,
    pubkeys, util,
};
use mpl_reward_center::state::PurchaseTicket;
use solana_program::pubkey;

use super::super::Client;
use crate::prelude::*;

pub(crate) async fn process(
    client: &Client,
    key: Pubkey,
    account_data: PurchaseTicket,
    slot: u64,
    write_version: u64,
) -> Result<()> {
    let row = DbRewardsPurchaseTicket {
        address: Owned(bs58::encode(key).into_string()),
        reward_center_address: Owned(bs58::encode(account_data.reward_center).into_string()),
        seller: Owned(bs58::encode(account_data.seller).into_string()),
        buyer: Owned(bs58::encode(account_data.buyer).into_string()),
        metadata: Owned(bs58::encode(account_data.metadata).into_string()),
        price: account_data
            .price
            .try_into()
            .context("Price is too big to store"),
        token_size: account_data
            .token_size
            .try_into()
            .context("Token size is too big to store"),
        created_at: util::unix_timestamp(account_data.created_at)?,
        slot: slot.try_into()?,
        write_version: write_version.try_into()?,
    };

    client
        .db()
        .run(move |db| {
            let auction_house = reward_centers::table
                .select(reward_center::all_columns)
                .filter(reward_centers::address.eq(row.address))
                .first::<AuctionHouse>(db)?;
            let current_metadata_owner = current_metadata_owners::table
                .select(current_metadata_owner::token_account)
                .inner_join(metadatas::table.on(metadatas::metadata_address.eq(row.metadata)))
                .filter(metadata::meta.eq(row.address))
                .first::<CurrentMetadataOwner>(db)?;

            let row = Purchase {
                id: None,
                buyer: row.buyer.clone(),
                seller: row.seller.clone(),
                auction_house: row.auction_house.clone(),
                marketplace_program: Owned(pubkeys::REWARD_CENTER.to_string()),
                metadata: row.metadata,
                token_size: row.token_size,
                price: row.price,
                created_at: row.created_at,
                slot: row.slot,
                write_version: Some(row.write_version),
            };

            let listing_exists = select(exists(
                listings::table.filter(
                    listings::trade_state
                        .eq(listing.trade_state.clone())
                        .and(listings::metadata.eq(listing.metadata.clone())),
                ),
            ))
            .get_result::<bool>(db)?;

            db.build_transaction().read_write().run(|| {
                let purchase_id = insert_into(purchases::table)
                    .values(&row)
                    .on_conflict(on_constraint("purchases_unique_fields"))
                    .do_update()
                    .set(&row)
                    .returning(purchases::id)
                    .get_result::<Uuid>(db)?;

                update(
                    offers::table.filter(
                        offers::auction_house
                            .eq(row.auction_house.clone())
                            .and(offers::buyer.eq(row.buyer.clone()))
                            .and(offers::metadata.eq(row.metadata.clone()))
                            .and(offers::purchase_id.is_null())
                            .and(offers::canceled_at.is_null()),
                    ),
                )
                .set(offers::purchase_id.eq(Some(purchase_id)))
                .execute(db)?;

                update(
                    listings::table.filter(
                        listings::auction_house
                            .eq(row.auction_house.clone())
                            .and(listings::seller.eq(row.seller.clone()))
                            .and(listings::metadata.eq(row.metadata.clone()))
                            .and(listings::price.eq(row.price))
                            .and(listings::token_size.eq(row.token_size))
                            .and(listings::purchase_id.is_null())
                            .and(listings::canceled_at.is_null()),
                    ),
                )
                .set(listings::purchase_id.eq(Some(purchase_id)))
                .execute(db)?;

                Result::Ok(())
            });
        })
        .await
        .context("Failed to insert rewards listing")?;
}
