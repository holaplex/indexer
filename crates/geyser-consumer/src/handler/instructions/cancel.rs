use borsh::BorshDeserialize;
use indexer_core::db::{
    custom_types::ActivityTypeEnum,
    insert_into,
    models::{CancelInstruction, Listing, Offer},
    mutations::activity,
    select,
    tables::{cancel_instructions, listings, offers},
    update,
};
use mpl_auction_house::instruction::Cancel;

use super::Client;
use crate::prelude::*;

pub(crate) async fn process(
    client: &Client,
    data: &[u8],
    accounts: &[Pubkey],
    slot: u64,
) -> Result<()> {
    let params = Cancel::try_from_slice(data).context("failed to deserialize")?;

    if accounts.len() != 8 {
        debug!("invalid accounts for CancelInstruction");
        return Ok(());
    }

    let accts: Vec<_> = accounts.iter().map(ToString::to_string).collect();

    let row = CancelInstruction {
        wallet: Owned(accts[0].clone()),
        token_account: Owned(accts[1].clone()),
        token_mint: Owned(accts[2].clone()),
        authority: Owned(accts[3].clone()),
        auction_house: Owned(accts[4].clone()),
        auction_house_fee_account: Owned(accts[5].clone()),
        trade_state: Owned(accts[6].clone()),
        buyer_price: params.buyer_price.try_into()?,
        token_size: params.token_size.try_into()?,
        created_at: Utc::now().naive_utc(),
        slot: slot.try_into()?,
    };

    client
        .db()
        .run(move |db| {
            insert_into(cancel_instructions::table)
                .values(&row)
                .execute(db)?;
            db.build_transaction().read_write().run(|| {
                let listing_trade_state = select(exists(
                    listings::table.filter(
                        listings::trade_state
                            .eq(row.trade_state.clone())
                            .and(listings::purchase_id.is_null())
                            .and(listings::canceled_at.is_null()),
                    ),
                ))
                .get_result::<bool>(db);

                if Ok(true) == listing_trade_state {
                    let listing = update(
                        listings::table.filter(
                            listings::trade_state
                                .eq(row.trade_state.clone())
                                .and(listings::purchase_id.is_null())
                                .and(listings::canceled_at.is_null()),
                        ),
                    )
                    .set((
                        listings::canceled_at.eq(Some(row.created_at)),
                        listings::slot.eq(row.slot),
                    ))
                    .returning(listings::all_columns)
                    .get_result::<Listing>(db)
                    .optional()?;

                    if let Some(listing) = listing {
                        activity::listing(
                            db,
                            listing.id.unwrap(),
                            &listing.clone(),
                            ActivityTypeEnum::ListingCanceled,
                        )?;
                    };
                } else {
                    let offer = update(
                        offers::table.filter(
                            offers::trade_state
                                .eq(row.trade_state.clone())
                                .and(offers::purchase_id.is_null())
                                .and(offers::canceled_at.is_null()),
                        ),
                    )
                    .set((
                        offers::canceled_at.eq(Some(row.created_at)),
                        offers::slot.eq(row.slot),
                    ))
                    .returning(offers::all_columns)
                    .get_result::<Offer>(db)
                    .optional()?;

                    if let Some(offer) = offer {
                        activity::offer(
                            db,
                            offer.id.unwrap(),
                            &offer.clone(),
                            ActivityTypeEnum::OfferCanceled,
                        )?;
                    }
                }

                Result::<_>::Ok(())
            })
        })
        .await
        .context("failed to insert cancel instruction ")?;

    Ok(())
}
