use borsh::BorshDeserialize;
use indexer_core::{
    db::{
        custom_types::ListingEventLifecycleEnum,
        insert_into,
        models::{FeedEventWallet, Listing, ListingEvent, SellInstruction},
        select,
        tables::{feed_event_wallets, feed_events, listing_events, listings, sell_instructions},
        Error as DbError,
    },
    uuid::Uuid,
};

use super::Client;
use crate::prelude::*;

#[derive(BorshDeserialize, Debug, Clone)]
pub struct InstructionParameters {
    trade_state_bump: u8,
    free_trade_state_bump: u8,
    program_as_signer_bump: u8,
    buyer_price: u64,
    token_size: u64,
}

pub(crate) async fn process(client: &Client, data: &[u8], accounts: &[Pubkey]) -> Result<()> {
    let params = InstructionParameters::try_from_slice(data).context("failed to deserialize")?;

    if accounts.len() != 12 {
        debug!("invalid accounts for SellInstruction");
        return Ok(());
    }

    let accts: Vec<String> = accounts.iter().map(ToString::to_string).collect();

    let row = SellInstruction {
        wallet: Owned(accts[0].clone()),
        token_account: Owned(accts[1].clone()),
        metadata: Owned(accts[2].clone()),
        authority: Owned(accts[3].clone()),
        auction_house: Owned(accts[4].clone()),
        auction_house_fee_account: Owned(accts[5].clone()),
        seller_trade_state: Owned(accts[6].clone()),
        free_seller_trader_state: Owned(accts[7].clone()),
        program_as_signer: Owned(accts[10].clone()),
        trade_state_bump: params.trade_state_bump.try_into()?,
        free_trade_state_bump: params.free_trade_state_bump.try_into()?,
        program_as_signer_bump: params.program_as_signer_bump.try_into()?,
        buyer_price: params.buyer_price.try_into()?,
        token_size: params.token_size.try_into()?,
        created_at: Utc::now().naive_utc(),
    };

    upsert_into_listings_table(client, row.clone())
        .await
        .context("failed to insert listing!")?;

    client
        .db()
        .run(move |db| {
            insert_into(sell_instructions::table)
                .values(&row)
                .execute(db)
        })
        .await
        .context("failed to insert sell instruction ")?;
    Ok(())
}

async fn upsert_into_listings_table<'a>(
    client: &Client,
    data: SellInstruction<'static>,
) -> Result<()> {
    let row = Listing {
        trade_state: data.seller_trade_state.clone(),
        bookkeeper: data.wallet.clone(),
        auction_house: data.auction_house.clone(),
        seller: data.wallet.clone(),
        metadata: data.metadata.clone(),
        purchase_id: None,
        price: data.buyer_price,
        token_size: data.token_size,
        bump: None,
        trade_state_bump: data.trade_state_bump,
        created_at: data.created_at,
        canceled_at: None,
    };

    client
        .db()
        .run(move |db| {
            let listing_exists = select(exists(
                listings::table.filter(
                    listings::trade_state
                        .eq(row.trade_state.clone())
                        .and(listings::bookkeeper.eq(row.bookkeeper.clone()))
                        .and(listings::auction_house.eq(row.auction_house.clone()))
                        .and(listings::seller.eq(row.seller.clone()))
                        .and(listings::metadata.eq(row.metadata.clone()))
                        .and(listings::price.eq(row.price))
                        .and(listings::token_size.eq(row.token_size))
                        .and(listings::trade_state_bump.eq(row.trade_state_bump)),
                ),
            ))
            .get_result::<bool>(db);

            if Ok(true) == listing_exists {
                return Ok(());
            }

            let listing_uuid = insert_into(listings::table)
                .values(&row)
                .on_conflict_do_nothing()
                .returning(listings::id)
                .get_results::<Uuid>(db)?
                .get(0)
                .context("failed to get inserted listing")?
                .to_string();

            db.build_transaction().read_write().run(|| {
                let feed_event_id = insert_into(feed_events::table)
                    .default_values()
                    .returning(feed_events::id)
                    .get_result::<Uuid>(db)
                    .context("Failed to insert feed event")?;

                let listing_event = insert_into(listing_events::table)
                    .values(&ListingEvent {
                        feed_event_id,
                        lifecycle: ListingEventLifecycleEnum::Created,
                        listing_receipt_address: Owned(listing_uuid),
                    })
                    .execute(db);

                if Err(DbError::RollbackTransaction) == listing_event {
                    return Ok(());
                }

                insert_into(feed_event_wallets::table)
                    .values(&FeedEventWallet {
                        wallet_address: row.seller,
                        feed_event_id,
                    })
                    .execute(db)
                    .context("Failed to insert listing feed event wallet")?;

                Result::<_>::Ok(())
            })
        })
        .await
        .context("Failed to insert listing!")?;

    Ok(())
}
