use borsh::BorshDeserialize;
use indexer_core::{
    db::{
        custom_types::OfferEventLifecycleEnum,
        insert_into,
        models::{BuyInstruction, FeedEventWallet, Offer, OfferEvent},
        select,
        tables::{
            buy_instructions, current_metadata_owners, feed_event_wallets, feed_events, metadatas,
            offer_events, offers,
        },
    },
    uuid::Uuid,
};
use mpl_auction_house::instruction::Buy;

use super::Client;
use crate::prelude::*;

pub(crate) async fn process(client: &Client, data: &[u8], accounts: &[Pubkey]) -> Result<()> {
    let params = Buy::try_from_slice(data).context("failed to deserialize")?;

    if accounts.len() != 14 {
        debug!("invalid accounts for BuyInstruction");
        return Ok(());
    }

    let accts: Vec<String> = accounts.iter().map(ToString::to_string).collect();

    let row = BuyInstruction {
        wallet: Owned(accts[0].clone()),
        payment_account: Owned(accts[1].clone()),
        transfer_authority: Owned(accts[2].clone()),
        treasury_mint: Owned(accts[3].clone()),
        token_account: Owned(accts[4].clone()),
        metadata: Owned(accts[5].clone()),
        escrow_payment_account: Owned(accts[6].clone()),
        authority: Owned(accts[7].clone()),
        auction_house: Owned(accts[8].clone()),
        auction_house_fee_account: Owned(accts[9].clone()),
        buyer_trade_state: Owned(accts[10].clone()),
        trade_state_bump: params.trade_state_bump.try_into()?,
        escrow_payment_bump: params.escrow_payment_bump.try_into()?,
        buyer_price: params.buyer_price.try_into()?,
        token_size: params.token_size.try_into()?,
        created_at: Utc::now().naive_utc(),
    };

    upsert_into_offers_table(client, row.clone())
        .await
        .context("failed to insert offer")?;

    client
        .db()
        .run(move |db| {
            insert_into(buy_instructions::table)
                .values(&row)
                .execute(db)
        })
        .await
        .context("failed to insert buy instruction ")?;
    Ok(())
}

async fn upsert_into_offers_table<'a>(
    client: &Client,
    data: BuyInstruction<'static>,
) -> Result<()> {
    let row = Offer {
        id: None,
        trade_state: data.buyer_trade_state,
        auction_house: data.auction_house,
        buyer: data.wallet,
        metadata: data.metadata,
        token_account: Some(data.token_account),
        purchase_id: None,
        price: data.buyer_price,
        token_size: data.token_size,
        trade_state_bump: data.trade_state_bump,
        created_at: data.created_at,
        canceled_at: None,
    };

    client
        .db()
        .run(move |db| {
            let offer = select(exists(
                offers::table.filter(
                    offers::trade_state
                        .eq(row.trade_state.clone())
                        .and(offers::metadata.eq(row.metadata.clone())),
                ),
            ))
            .get_result::<bool>(db);

            if Ok(true) == offer {
                return Ok(None);
            }

            let offer_uuid = insert_into(offers::table)
                .values(&row)
                .on_conflict_do_nothing()
                .returning(offers::id)
                .get_results::<Uuid>(db)?
                .get(0)
                .context("failed to get inserted offer")?
                .to_string();

            let uuid = offer_uuid.clone();

            db.build_transaction().read_write().run(|| {
                let metadata_owner: String = current_metadata_owners::table
                    .inner_join(
                        metadatas::table
                            .on(metadatas::mint_address.eq(current_metadata_owners::mint_address)),
                    )
                    .filter(metadatas::address.eq(row.metadata.clone()))
                    .select(current_metadata_owners::owner_address)
                    .first(db)?;

                let feed_event_id = insert_into(feed_events::table)
                    .default_values()
                    .returning(feed_events::id)
                    .get_result::<Uuid>(db)
                    .context("Failed to insert feed event")?;

                insert_into(offer_events::table)
                    .values(&OfferEvent {
                        feed_event_id,
                        lifecycle: OfferEventLifecycleEnum::Created,
                        bid_receipt_address: Owned(offer_uuid),
                    })
                    .execute(db)
                    .context("failed to insert offer created event")?;

                insert_into(feed_event_wallets::table)
                    .values(&FeedEventWallet {
                        wallet_address: row.buyer,
                        feed_event_id,
                    })
                    .execute(db)
                    .context("Failed to insert offer feed event wallet for buyer")?;

                insert_into(feed_event_wallets::table)
                    .values(&FeedEventWallet {
                        wallet_address: Owned(metadata_owner),
                        feed_event_id,
                    })
                    .execute(db)
                    .context("Failed to insert offer feed event wallet for metadata owner")?;

                Result::<_>::Ok(Some((feed_event_id, uuid)))
            })
        })
        .await
        .context("Failed to insert offer!")?;

    Ok(())
}
