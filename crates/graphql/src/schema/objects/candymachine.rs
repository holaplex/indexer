use indexer_core::{
    base64,
    db::custom_types::{EndSettingType, WhitelistMintMode},
};
use objects::wallet::Wallet;
use scalars::{PublicKey, U64};

use super::prelude::*;
use crate::schema::scalars::markers::TokenMint;

#[derive(Debug, Clone)]
pub struct CandyMachine {
    pub address: PublicKey<CandyMachine>,
    pub authority: PublicKey<Wallet>,
    pub wallet: PublicKey<Wallet>,
    pub token_mint: Option<PublicKey<TokenMint>>,
    pub items_redeemed: U64,

    pub uuid: String,
    pub price: U64,
    pub symbol: String,
    pub seller_fee_basis_points: i32,
    pub max_supply: U64,
    pub is_mutable: bool,
    pub retain_authority: bool,
    pub go_live_date: Option<U64>,
    pub items_available: U64,
}

#[graphql_object(Context = AppContext)]
impl CandyMachine {
    pub fn address(&self) -> &PublicKey<CandyMachine> {
        &self.address
    }

    pub fn authority(&self) -> &PublicKey<Wallet> {
        &self.authority
    }

    pub fn wallet(&self) -> &PublicKey<Wallet> {
        &self.wallet
    }

    pub fn token_mint(&self) -> &Option<PublicKey<TokenMint>> {
        &self.token_mint
    }

    pub fn items_redeemed(&self) -> &U64 {
        &self.items_redeemed
    }

    pub fn uuid(&self) -> &String {
        &self.uuid
    }

    pub fn price(&self) -> &U64 {
        &self.price
    }

    pub fn symbol(&self) -> &String {
        &self.symbol
    }

    pub fn seller_fee_basis_points(&self) -> &i32 {
        &self.seller_fee_basis_points
    }

    pub fn max_supply(&self) -> &U64 {
        &self.max_supply
    }

    pub fn is_mutable(&self) -> &bool {
        &self.is_mutable
    }

    pub fn retain_authority(&self) -> &bool {
        &self.retain_authority
    }

    pub fn go_live_date(&self) -> &Option<U64> {
        &self.go_live_date
    }

    pub fn items_available(&self) -> &U64 {
        &self.items_available
    }

    #[graphql(description = "NOTE - this is currently bugged and will only return one creator")]
    pub async fn creators(&self, ctx: &AppContext) -> FieldResult<Vec<CandyMachineCreator>> {
        ctx.candymachine_creator_loader
            .load(self.address.clone())
            .await
            .map_err(Into::into)
    }

    pub async fn collection_pda(
        &self,
        ctx: &AppContext,
    ) -> FieldResult<Option<CandyMachineCollectionPda>> {
        ctx.candymachine_collection_pda_loader
            .load(self.address.clone())
            .await
            .map_err(Into::into)
    }

    #[graphql(description = "NOTE - this is currently bugged and will always be empty")]
    pub async fn config_lines(&self, ctx: &AppContext) -> FieldResult<Vec<CandyMachineConfigLine>> {
        ctx.candymachine_config_line_loader
            .load(self.address.clone())
            .await
            .map_err(Into::into)
    }

    pub async fn end_setting(
        &self,
        ctx: &AppContext,
    ) -> FieldResult<Option<CandyMachineEndSetting>> {
        ctx.candymachine_end_settings_loader
            .load(self.address.clone())
            .await
            .map_err(Into::into)
    }

    pub async fn whitelist_mint_setting(
        &self,
        ctx: &AppContext,
    ) -> FieldResult<Option<CandyMachineWhitelistMintSetting>> {
        ctx.candymachine_whitelist_mint_settings_loader
            .load(self.address.clone())
            .await
            .map_err(Into::into)
    }

    pub async fn hidden_setting(
        &self,
        ctx: &AppContext,
    ) -> FieldResult<Option<CandyMachineHiddenSetting>> {
        ctx.candymachine_hidden_settings_loader
            .load(self.address.clone())
            .await
            .map_err(Into::into)
    }

    pub async fn gate_keeper_config(
        &self,
        ctx: &AppContext,
    ) -> FieldResult<Option<CandyMachineGateKeeperConfig>> {
        ctx.candymachine_gatekeeper_configs_loader
            .load(self.address.clone())
            .await
            .map_err(Into::into)
    }
}

impl<'a, 'b> TryFrom<(models::CandyMachine<'a>, models::CandyMachineData<'b>)> for CandyMachine {
    type Error = std::num::TryFromIntError;

    fn try_from(
        (
            models::CandyMachine {
                address,
                authority,
                wallet,
                token_mint,
                items_redeemed,
            },
            models::CandyMachineData {
                uuid,
                price,
                symbol,
                seller_fee_basis_points,
                max_supply,
                is_mutable,
                retain_authority,
                go_live_date,
                items_available,
                ..
            },
        ): (models::CandyMachine, models::CandyMachineData),
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            address: address.into(),
            authority: authority.into(),
            wallet: wallet.into(),
            token_mint: token_mint.map(Into::into),
            items_redeemed: items_redeemed.try_into()?,
            uuid: uuid.into_owned(),
            price: price.try_into()?,
            symbol: symbol.into_owned(),
            seller_fee_basis_points: seller_fee_basis_points.try_into()?,
            max_supply: max_supply.try_into()?,
            is_mutable,
            retain_authority,
            go_live_date: go_live_date.map(U64::try_from).transpose()?,
            items_available: items_available.try_into()?,
        })
    }
}

#[derive(Debug, Clone, GraphQLObject)]
pub struct CandyMachineCreator {
    pub candy_machine_address: PublicKey<CandyMachine>,
    pub creator_address: PublicKey<Wallet>,
    pub verified: bool,
    pub share: i32,
}

impl<'a> TryFrom<models::CMCreator<'a>> for CandyMachineCreator {
    type Error = std::num::TryFromIntError;

    fn try_from(
        models::CMCreator {
            candy_machine_address,
            creator_address,
            verified,
            share,
        }: models::CMCreator,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            candy_machine_address: candy_machine_address.into(),
            creator_address: creator_address.into(),
            verified,
            share: share.try_into()?,
        })
    }
}

#[derive(Debug, Clone, GraphQLObject)]
pub struct CandyMachineCollectionPda {
    pub candy_machine_address: PublicKey<CandyMachine>,
    pub collection_pda: PublicKey<CandyMachine>,
    pub mint: PublicKey<TokenMint>,
}

impl<'a> From<models::CMCollectionPDA<'a>> for CandyMachineCollectionPda {
    fn from(
        models::CMCollectionPDA {
            address,
            mint,
            candy_machine,
        }: models::CMCollectionPDA,
    ) -> Self {
        Self {
            candy_machine_address: candy_machine.into(),
            collection_pda: address.into(),
            mint: mint.into(),
        }
    }
}

#[derive(Debug, Clone, GraphQLObject)]
pub struct CandyMachineConfigLine {
    pub candy_machine_address: PublicKey<CandyMachine>,
    pub name: String,
    pub uri: String,
    pub idx: i32,
    pub taken: bool,
}

impl<'a> From<models::CMConfigLine<'a>> for CandyMachineConfigLine {
    fn from(
        models::CMConfigLine {
            candy_machine_address,
            name,
            uri,
            idx,
            taken,
        }: models::CMConfigLine,
    ) -> Self {
        Self {
            candy_machine_address: candy_machine_address.into(),
            name: name.into_owned(),
            uri: uri.into_owned(),
            idx,
            taken,
        }
    }
}

#[derive(Debug, Clone, GraphQLObject)]
pub struct CandyMachineEndSetting {
    pub candy_machine_address: PublicKey<CandyMachine>,
    pub end_setting_type: CandyMachineEndSettingType,
    pub number: U64,
}

#[derive(Debug, Clone, juniper::GraphQLEnum)]
pub enum CandyMachineEndSettingType {
    Date,
    Amount,
}

impl From<EndSettingType> for CandyMachineEndSettingType {
    fn from(v: EndSettingType) -> Self {
        match v {
            EndSettingType::Date => CandyMachineEndSettingType::Date,
            EndSettingType::Amount => CandyMachineEndSettingType::Amount,
        }
    }
}

impl<'a> TryFrom<models::CMEndSetting<'a>> for CandyMachineEndSetting {
    type Error = std::num::TryFromIntError;

    fn try_from(
        models::CMEndSetting {
            candy_machine_address,
            end_setting_type,
            number,
        }: models::CMEndSetting,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            candy_machine_address: candy_machine_address.into(),
            end_setting_type: end_setting_type.into(),
            number: number.try_into()?,
        })
    }
}

#[derive(Debug, Clone, GraphQLObject)]
pub struct CandyMachineWhitelistMintSetting {
    pub candy_machine_address: PublicKey<CandyMachine>,
    pub mode: CandyMachineWhitelistMintMode,
    pub mint: PublicKey<TokenMint>,
    pub presale: bool,
    pub discount_price: Option<U64>,
}

#[derive(Debug, Clone, juniper::GraphQLEnum)]
pub enum CandyMachineWhitelistMintMode {
    BurnEveryTime,
    NeverBurn,
}

impl From<WhitelistMintMode> for CandyMachineWhitelistMintMode {
    fn from(v: WhitelistMintMode) -> Self {
        match v {
            WhitelistMintMode::BurnEveryTime => CandyMachineWhitelistMintMode::BurnEveryTime,
            WhitelistMintMode::NeverBurn => CandyMachineWhitelistMintMode::NeverBurn,
        }
    }
}

impl<'a> TryFrom<models::CMWhitelistMintSetting<'a>> for CandyMachineWhitelistMintSetting {
    type Error = std::num::TryFromIntError;

    fn try_from(
        models::CMWhitelistMintSetting {
            candy_machine_address,
            mode,
            mint,
            presale,
            discount_price,
        }: models::CMWhitelistMintSetting,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            candy_machine_address: candy_machine_address.into(),
            mode: mode.into(),
            mint: mint.into(),
            presale,
            discount_price: discount_price.map(U64::try_from).transpose()?,
        })
    }
}

#[derive(Debug, Clone, GraphQLObject)]
pub struct CandyMachineHiddenSetting {
    pub candy_machine_address: PublicKey<CandyMachine>,
    pub name: String,
    pub uri: String,
    #[graphql(description = "lowercase base64 encoded string of the hash bytes")]
    pub hash: String,
}

impl<'a> TryFrom<models::CMHiddenSetting<'a>> for CandyMachineHiddenSetting {
    type Error = std::num::TryFromIntError;

    fn try_from(
        models::CMHiddenSetting {
            candy_machine_address,
            name,
            uri,
            hash,
        }: models::CMHiddenSetting,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            candy_machine_address: candy_machine_address.into(),
            name: name.into_owned(),
            uri: uri.into_owned(),
            hash: base64::encode_config(hash, base64::STANDARD_NO_PAD),
        })
    }
}

#[derive(Debug, Clone, GraphQLObject)]
pub struct CandyMachineGateKeeperConfig {
    pub candy_machine_address: PublicKey<CandyMachine>,
    pub gatekeeper_network: String,
    pub expire_on_use: bool,
}

impl<'a> From<models::CMGateKeeperConfig<'a>> for CandyMachineGateKeeperConfig {
    fn from(
        models::CMGateKeeperConfig {
            candy_machine_address,
            gatekeeper_network,
            expire_on_use,
        }: models::CMGateKeeperConfig,
    ) -> Self {
        Self {
            candy_machine_address: candy_machine_address.into(),
            gatekeeper_network: gatekeeper_network.into_owned(),
            expire_on_use,
        }
    }
}
