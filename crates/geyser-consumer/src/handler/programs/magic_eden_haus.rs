use borsh::BorshDeserialize;
use indexer::prelude::*;
use indexer_core::{
    db::{
        custom_types::ActivityTypeEnum,
        models::{Listing, Offer, Purchase},
        mutations::activity,
        tables::{listings, offers, purchases},
        update,
    },
    pubkeys,
    util::{self, unix_timestamp},
    uuid::Uuid,
};
use solana_client::{
    client_error::{ClientError, ClientErrorKind},
    rpc_request::RpcError,
};

use super::{
    instructions::{
        buy::upsert_into_offers_table, execute_sale::upsert_into_purchases_table,
        sell::upsert_into_listings_table,
    },
    Client,
};

const BUY: [u8; 8] = [102, 6, 61, 18, 1, 218, 235, 234];
const SELL: [u8; 8] = [51, 230, 133, 164, 1, 127, 131, 173];
const EXECUTE_SALE: [u8; 8] = [37, 74, 217, 157, 79, 49, 35, 6];
const EXECUTE_SALEV2: [u8; 8] = [91, 220, 49, 223, 204, 129, 53, 193];
const CANCEL_SELL: [u8; 8] = [198, 198, 130, 203, 163, 95, 175, 75];
const CANCEL_BUY: [u8; 8] = [238, 76, 36, 218, 132, 177, 224, 233];

#[derive(BorshDeserialize, Debug, Clone)]
struct MEInstructionData {
    trade_state_bump: u8,
    _escrow_payment_bump: u8,
    buyer_price: u64,
    token_size: u64,
    expiry: i64,
}

async fn process_execute_sale(
    client: &Client,
    mut data: &[u8],
    accounts: &[Pubkey],
    slot: u64,
    timestamp: NaiveDateTime,
) -> Result<()> {
    let params = MEInstructionData::deserialize(&mut data)
        .context("failed to deserialize ME ExecuteSale instruction")?;

    let accts: Vec<_> = accounts.iter().map(ToString::to_string).collect();

    upsert_into_purchases_table(
        client,
        Purchase {
            id: None,
            buyer: Owned(accts[0].clone()),
            seller: Owned(accts[1].clone()),
            auction_house: Owned(accts[9].clone()),
            marketplace_program: Owned(pubkeys::ME_HAUS.to_string()),
            metadata: Owned(accts[5].clone()),
            token_size: params.token_size.try_into()?,
            price: params.buyer_price.try_into()?,
            created_at: timestamp,
            slot: slot.try_into()?,
            write_version: None,
        },
        accts[11].clone(),
        accts[13].clone(),
    )
    .await
    .context("failed to insert listing!")?;

    Ok(())
}

async fn process_sale(
    client: &Client,
    mut data: &[u8],
    accounts: &[Pubkey],
    slot: u64,
    timestamp: NaiveDateTime,
) -> Result<()> {
    let params = MEInstructionData::deserialize(&mut data)
        .context("failed to deserialize ME Sell instruction")?;

    let accts: Vec<_> = accounts.iter().map(ToString::to_string).collect();

    let seller = accts[0].clone();
    let auction_house = accts[7].clone();
    let metadata = accts[5].clone();
    let price = i64::try_from(params.buyer_price)?;
    let token_size = i64::try_from(params.token_size)?;
    let slot = i64::try_from(slot)?;

    let purchase_id = client
        .db()
        .run({
            move |db| {
                purchases::table
                    .filter(
                        purchases::seller
                            .eq(seller)
                            .and(purchases::auction_house.eq(auction_house))
                            .and(purchases::metadata.eq(metadata))
                            .and(purchases::price.eq(price))
                            .and(
                                purchases::token_size
                                    .eq(token_size)
                                    .and(purchases::slot.eq(slot)),
                            ),
                    )
                    .select(purchases::id)
                    .first::<Uuid>(db)
                    .optional()
                    .context("failed to get purchase ids")
            }
        })
        .await?;

    upsert_into_listings_table(client, Listing {
        id: None,
        trade_state: Owned(accts[8].clone()),
        auction_house: Owned(accts[7].clone()),
        marketplace_program: Owned(pubkeys::ME_HAUS.to_string()),
        seller: Owned(accts[0].clone()),
        metadata: Owned(accts[5].clone()),
        purchase_id,
        price: params.buyer_price.try_into()?,
        token_size: params.token_size.try_into()?,
        trade_state_bump: params.trade_state_bump.try_into()?,
        created_at: timestamp,
        canceled_at: Some(None),
        slot,
        write_version: None,
        expiry: match params.expiry {
            e if e <= 0 => None,
            _ => Some(util::unix_timestamp(params.expiry)?),
        },
    })
    .await
    .context("failed to insert listing!")?;

    Ok(())
}

async fn process_buy(
    client: &Client,
    mut data: &[u8],
    accounts: &[Pubkey],
    slot: u64,
    timestamp: NaiveDateTime,
) -> Result<()> {
    let params = MEInstructionData::deserialize(&mut data)
        .context("failed to deserialize ME Buy instruction")?;

    if accounts.len() != 12 {
        debug!("invalid accounts for BuyInstruction");
        return Ok(());
    }

    let accts: Vec<_> = accounts.iter().map(ToString::to_string).collect();
    let buyer = accts[0].clone();
    let auction_house = accts[6].clone();
    let metadata = accts[3].clone();
    let price = i64::try_from(params.buyer_price)?;
    let token_size = i64::try_from(params.token_size)?;
    let slot = i64::try_from(slot)?;

    let purchase_id: Option<Uuid> = client
        .db()
        .run({
            move |db| {
                purchases::table
                    .filter(
                        purchases::buyer
                            .eq(buyer)
                            .and(purchases::auction_house.eq(auction_house))
                            .and(purchases::metadata.eq(metadata))
                            .and(purchases::price.eq(price))
                            .and(
                                purchases::token_size
                                    .eq(token_size)
                                    .and(purchases::slot.eq(slot)),
                            ),
                    )
                    .select(purchases::id)
                    .first::<Uuid>(db)
                    .optional()
                    .context("failed to get purchase ids")
            }
        })
        .await?;

    let offer = Offer {
        id: None,
        trade_state: Owned(accts[7].clone()),
        auction_house: Owned(accts[6].clone()),
        marketplace_program: Owned(pubkeys::ME_HAUS.to_string()),
        buyer: Owned(accts[0].clone()),
        metadata: Owned(accts[3].clone()),
        token_account: None,
        purchase_id,
        price,
        token_size: params.token_size.try_into()?,
        trade_state_bump: params.trade_state_bump.try_into()?,
        created_at: timestamp,
        canceled_at: Some(None),
        slot,
        write_version: None,
        expiry: match params.expiry {
            e if e <= 0 => None,
            _ => Some(util::unix_timestamp(params.expiry)?),
        },
    };

    upsert_into_offers_table(client, offer)
        .await
        .context("failed to insert offer")?;

    Ok(())
}

async fn process_cancel_sale(
    client: &Client,
    accounts: &[Pubkey],
    slot: u64,
    timestamp: NaiveDateTime,
) -> Result<()> {
    let accts: Vec<_> = accounts.iter().map(ToString::to_string).collect();

    let trade_state = accts[6].clone();
    let slot = i64::try_from(slot)?;

    client
        .db()
        .run(move |db| {
            let listing = update(
                listings::table.filter(
                    listings::trade_state
                        .eq(trade_state)
                        .and(listings::purchase_id.is_null())
                        .and(listings::canceled_at.is_null()),
                ),
            )
            .set((
                listings::canceled_at.eq(Some(timestamp)),
                listings::slot.eq(slot),
            ))
            .returning(listings::all_columns)
            .get_result::<Listing>(db)?;

            activity::listing(
                db,
                listing.id.unwrap(),
                &listing.clone(),
                ActivityTypeEnum::ListingCanceled,
            )
        })
        .await
        .context("failed to cancel ME listing ")?;

    Ok(())
}

async fn process_cancel_buy(
    client: &Client,
    accounts: &[Pubkey],
    slot: u64,
    timestamp: NaiveDateTime,
) -> Result<()> {
    let accts: Vec<_> = accounts.iter().map(ToString::to_string).collect();
    let trade_state = accts[5].clone();
    let slot = i64::try_from(slot)?;

    client
        .db()
        .run(move |db| {
            let offer = update(
                offers::table.filter(
                    offers::trade_state
                        .eq(trade_state)
                        .and(offers::purchase_id.is_null())
                        .and(offers::canceled_at.is_null()),
                ),
            )
            .set((
                offers::canceled_at.eq(Some(timestamp)),
                offers::slot.eq(slot),
            ))
            .returning(offers::all_columns)
            .get_result::<Offer>(db)?;

            activity::offer(
                db,
                offer.id.unwrap(),
                &offer.clone(),
                ActivityTypeEnum::OfferCanceled,
            )
        })
        .await
        .context("failed to cancel ME bid ")?;

    Ok(())
}

pub(crate) async fn process_instruction(
    client: &Client,
    data: &[u8],
    accounts: &[Pubkey],
    slot: u64,
) -> Result<()> {
    let (discriminator, params) = data.split_at(8);
    let discriminator = <[u8; 8]>::try_from(discriminator)?;

    let block_time = get_block_time(client, slot)?;

    match discriminator {
        BUY => process_buy(client, params, accounts, slot, block_time).await,
        SELL => process_sale(client, params, accounts, slot, block_time).await,
        EXECUTE_SALE => process_execute_sale(client, params, accounts, slot, block_time).await,
        EXECUTE_SALEV2 => process_execute_sale(client, params, accounts, slot, block_time).await,
        CANCEL_SELL => process_cancel_sale(client, accounts, slot, block_time).await,
        CANCEL_BUY => process_cancel_buy(client, accounts, slot, block_time).await,
        _ => Ok(()),
    }
}

pub(crate) fn get_block_time(client: &Client, slot: u64) -> Result<NaiveDateTime> {
    match client.rpc().get_block_time(slot) {
        Ok(bt) => unix_timestamp(bt),
        // Catch slot-not-found errors, likely due to race condition between
        // Geyser and RPC
        Err(ClientError {
            request: _,
            kind:
                ClientErrorKind::RpcError(RpcError::RpcResponseError {
                    code: -32009 | -32004,
                    ..
                }),
        }) => Ok(Utc::now().naive_utc()),
        Err(e) => Err(e).context("Error getting block time"),
    }
}
