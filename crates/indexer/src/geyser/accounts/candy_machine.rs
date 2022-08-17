use futures_util::{future::join_all, Future};
use indexer_core::{
    db::{
        custom_types::{
            EndSettingType as DbEndSettingType, WhitelistMintMode as DbWhitelistMintMode,
        },
        insert_into,
        models::{
            CMCollectionPDA, CMConfigLine, CMCreator, CMEndSetting, CMGateKeeperConfig,
            CMHiddenSetting, CMWhitelistMintSetting, CandyMachine as DbCandyMachine,
            CandyMachineData as CMData,
        },
        tables::{
            candy_machine_collection_pdas, candy_machine_config_lines, candy_machine_creators,
            candy_machine_datas, candy_machine_end_settings, candy_machine_gate_keeper_configs,
            candy_machine_hidden_settings, candy_machine_whitelist_mint_settings, candy_machines,
        },
    },
    prelude::*,
};
use mpl_candy_machine::{
    CandyMachine, CandyMachineData, CollectionPDA, ConfigLine, Creator, EndSettingType,
    EndSettings, GatekeeperConfig, HiddenSettings, WhitelistMintMode, WhitelistMintSettings,
};

use super::Client;
use crate::prelude::*;

pub(crate) async fn process(
    client: &Client,
    key: Pubkey,
    candy_machine: CandyMachine,
    config_lines: Option<Vec<(ConfigLine, usize, bool)>>,
) -> Result<()> {
    let cm = DbCandyMachine {
        address: Owned(bs58::encode(key).into_string()),
        authority: Owned(bs58::encode(candy_machine.authority).into_string()),
        wallet: Owned(bs58::encode(candy_machine.wallet).into_string()),
        token_mint: candy_machine
            .token_mint
            .map(|t| Owned(bs58::encode(t).into_string())),
        items_redeemed: candy_machine.items_redeemed.try_into()?,
    };

    client
        .db()
        .run(move |db| {
            insert_into(candy_machines::table)
                .values(&cm)
                .on_conflict(candy_machines::address)
                .do_update()
                .set(&cm)
                .execute(db)
        })
        .await
        .context("failed to insert candy machine")?;

    let mut futures: Vec<std::pin::Pin<Box<dyn Future<Output = Result<()>> + Send>>> = vec![
        Box::pin(process_data(client, key, candy_machine.data.clone())),
        Box::pin(process_creators(client, key, candy_machine.data.creators)),
    ];

    if let Some(config_lines) = config_lines {
        futures.push(Box::pin(process_config_lines(client, key, config_lines)));
    }

    if let Some(es) = candy_machine.data.end_settings {
        futures.push(Box::pin(process_end_settings(client, key, es)));
    };

    if let Some(hs) = candy_machine.data.hidden_settings {
        futures.push(Box::pin(process_hidden_settings(client, key, hs)));
    }

    if let Some(gk) = candy_machine.data.gatekeeper {
        futures.push(Box::pin(process_gatekeeper_config(client, key, gk)));
    }

    if let Some(wlms) = candy_machine.data.whitelist_mint_settings {
        futures.push(Box::pin(process_whitelist_mint_settings(client, key, wlms)));
    }

    join_all(futures).await;

    Ok(())
}

async fn process_config_lines(
    client: &Client,
    key: Pubkey,
    config_lines: Vec<(ConfigLine, usize, bool)>,
) -> Result<()> {
    let mut db_config_lines: Vec<CMConfigLine> = Vec::new();

    for config_line in &config_lines {
        let db_config_line = CMConfigLine {
            candy_machine_address: Owned(bs58::encode(key).into_string()),
            name: Owned(config_line.0.name.trim_matches(char::from(0)).to_owned()),
            uri: Owned(config_line.0.uri.trim_matches(char::from(0)).to_owned()),
            idx: i32::try_from(config_line.1).unwrap_or(-1i32),
            taken: config_line.2,
        };
        db_config_lines.push(db_config_line);
    }

    client
        .db()
        .run(move |db| {
            db.build_transaction().read_write().run(|| {
                for cl in &db_config_lines {
                    insert_into(candy_machine_config_lines::table)
                        .values(cl)
                        .on_conflict((
                            candy_machine_config_lines::candy_machine_address,
                            candy_machine_config_lines::idx,
                        ))
                        .do_update()
                        .set(cl)
                        .execute(db)
                        .context("Failed to insert config line")?;
                }

                Result::<_>::Ok(())
            })
        })
        .await
        .context("failed to insert candy machine config lines")?;

    Ok(())
}

async fn process_data(client: &Client, key: Pubkey, data: CandyMachineData) -> Result<()> {
    let cm_data = CMData {
        candy_machine_address: Owned(bs58::encode(key).into_string()),
        uuid: Owned(data.uuid),
        price: data.price.try_into()?,
        symbol: Owned(data.symbol.trim_end_matches('\0').to_owned()),
        seller_fee_basis_points: data.seller_fee_basis_points.try_into()?,
        max_supply: data.max_supply.try_into()?,
        is_mutable: data.is_mutable,
        retain_authority: data.retain_authority,
        go_live_date: data.go_live_date,
        items_available: data.items_available.try_into()?,
    };

    client
        .db()
        .run(move |db| {
            insert_into(candy_machine_datas::table)
                .values(&cm_data)
                .on_conflict(candy_machine_datas::candy_machine_address)
                .do_update()
                .set(&cm_data)
                .execute(db)
        })
        .await
        .context("failed to insert candy machine data")?;
    Ok(())
}

async fn process_creators(client: &Client, key: Pubkey, creators: Vec<Creator>) -> Result<()> {
    for creator in creators {
        let c = CMCreator {
            candy_machine_address: Owned(bs58::encode(key).into_string()),
            creator_address: Owned(bs58::encode(creator.address).into_string()),
            verified: creator.verified,
            share: creator.share.into(),
        };

        client
            .db()
            .run(move |db| {
                insert_into(candy_machine_creators::table)
                    .values(&c)
                    .on_conflict((
                        candy_machine_creators::candy_machine_address,
                        candy_machine_creators::creator_address,
                    ))
                    .do_update()
                    .set(&c)
                    .execute(db)
            })
            .await
            .context("failed to insert creator")?;
    }
    Ok(())
}

async fn process_end_settings(client: &Client, key: Pubkey, es: EndSettings) -> Result<()> {
    let end_setting = CMEndSetting {
        candy_machine_address: Owned(bs58::encode(key).into_string()),
        end_setting_type: match es.end_setting_type {
            EndSettingType::Date => DbEndSettingType::Date,
            EndSettingType::Amount => DbEndSettingType::Amount,
        },
        number: es.number.try_into()?,
    };

    client
        .db()
        .run(move |db| {
            insert_into(candy_machine_end_settings::table)
                .values(&end_setting)
                .on_conflict(candy_machine_end_settings::candy_machine_address)
                .do_update()
                .set(&end_setting)
                .execute(db)
        })
        .await
        .context("failed to insert candy machine end setting")?;
    Ok(())
}

async fn process_hidden_settings(client: &Client, key: Pubkey, hs: HiddenSettings) -> Result<()> {
    let hidden_setting = CMHiddenSetting {
        candy_machine_address: Owned(bs58::encode(key).into_string()),
        name: Owned(hs.name),
        uri: Owned(hs.uri),
        hash: hs.hash.to_vec(),
    };

    client
        .db()
        .run(move |db| {
            insert_into(candy_machine_hidden_settings::table)
                .values(&hidden_setting)
                .on_conflict(candy_machine_hidden_settings::candy_machine_address)
                .do_update()
                .set(&hidden_setting)
                .execute(db)
        })
        .await
        .context("failed to insert hidden setting")?;
    Ok(())
}

async fn process_gatekeeper_config(
    client: &Client,
    key: Pubkey,
    gk: GatekeeperConfig,
) -> Result<()> {
    let gatekeeper = CMGateKeeperConfig {
        candy_machine_address: Owned(bs58::encode(key).into_string()),
        gatekeeper_network: Owned(bs58::encode(gk.gatekeeper_network).into_string()),
        expire_on_use: gk.expire_on_use,
    };

    client
        .db()
        .run(move |db| {
            insert_into(candy_machine_gate_keeper_configs::table)
                .values(&gatekeeper)
                .on_conflict(candy_machine_gate_keeper_configs::candy_machine_address)
                .do_update()
                .set(&gatekeeper)
                .execute(db)
        })
        .await
        .context("failed to insert gate keeper config")?;
    Ok(())
}
async fn process_whitelist_mint_settings(
    client: &Client,
    key: Pubkey,
    wlms: WhitelistMintSettings,
) -> Result<()> {
    let whitelist_mint_setting = CMWhitelistMintSetting {
        candy_machine_address: Owned(bs58::encode(key).into_string()),
        mode: match wlms.mode {
            WhitelistMintMode::BurnEveryTime => DbWhitelistMintMode::BurnEveryTime,
            WhitelistMintMode::NeverBurn => DbWhitelistMintMode::NeverBurn,
        },
        mint: Owned(bs58::encode(wlms.mint).into_string()),
        presale: wlms.presale,
        discount_price: wlms
            .discount_price
            .map(TryInto::try_into)
            .transpose()
            .context("error casting u64 to i64!")?,
    };

    client
        .db()
        .run(move |db| {
            insert_into(candy_machine_whitelist_mint_settings::table)
                .values(&whitelist_mint_setting)
                .on_conflict(candy_machine_whitelist_mint_settings::candy_machine_address)
                .do_update()
                .set(&whitelist_mint_setting)
                .execute(db)
        })
        .await
        .context("failed to insert whitelist mint setting")?;
    Ok(())
}

pub(crate) async fn process_collection_pda(
    client: &Client,
    key: Pubkey,
    collection_pda: CollectionPDA,
) -> Result<()> {
    let row = CMCollectionPDA {
        address: Owned(bs58::encode(key).into_string()),
        mint: Owned(bs58::encode(collection_pda.mint).into_string()),
        candy_machine: Owned(bs58::encode(collection_pda.candy_machine).into_string()),
    };

    client
        .db()
        .run(move |db| {
            insert_into(candy_machine_collection_pdas::table)
                .values(&row)
                .on_conflict(candy_machine_collection_pdas::address)
                .do_update()
                .set(&row)
                .execute(db)
        })
        .await
        .context("failed to insert collection pda")?;
    Ok(())
}
