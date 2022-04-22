//! Includes `WhitelistMintMode` enum and `EndSettingType` enum

use std::{fmt, io::Write};

use diesel::{
    deserialize::{self, FromSql},
    not_none,
    pg::Pg,
    serialize::{self, IsNull, Output, ToSql},
    AsExpression, FromSqlRow, SqlType,
};

#[derive(SqlType, Debug, Clone, Copy)]
#[postgres(type_name = "settingtype")]
/// Represents database 'settingtype' type
pub struct SettingType;

#[derive(Debug, PartialEq, FromSqlRow, AsExpression, Clone, Copy)]
#[sql_type = "SettingType"]
/// `EndSettingType` enum in `EndSettings`
pub enum EndSettingType {
    /// EndSettingtype
    /// Date : Enable the use of a date to stop the mint
    Date,
    /// Amount: Enable stopping the mint after a specific amount is minted
    Amount,
}

impl ToSql<SettingType, Pg> for EndSettingType {
    fn to_sql<W: Write>(&self, out: &mut Output<W, Pg>) -> serialize::Result {
        match *self {
            EndSettingType::Date => out.write_all(b"Date")?,
            EndSettingType::Amount => out.write_all(b"Amount")?,
        }
        Ok(IsNull::No)
    }
}

impl FromSql<SettingType, Pg> for EndSettingType {
    fn from_sql(bytes: Option<&[u8]>) -> deserialize::Result<Self> {
        match not_none!(bytes) {
            b"date" => Ok(EndSettingType::Date),
            b"amount" => Ok(EndSettingType::Amount),
            _ => Err("Unrecognized enum variant".into()),
        }
    }
}

#[derive(SqlType, Debug, Clone, Copy)]
#[postgres(type_name = "mode")]
/// Represents database 'mode' type
pub struct Mode;

#[derive(Debug, PartialEq, FromSqlRow, AsExpression, Clone, Copy)]
#[sql_type = "Mode"]
/// `WhitelistMintMode` enum in `WhitelistSettings`
pub enum WhitelistMintMode {
    /// Whitelist token is burned after the mint
    BurnEveryTime,
    /// Whitelist token is returned to holder
    NeverBurn,
}

impl ToSql<Mode, Pg> for WhitelistMintMode {
    fn to_sql<W: Write>(&self, out: &mut Output<W, Pg>) -> serialize::Result {
        match *self {
            WhitelistMintMode::BurnEveryTime => out.write_all(b"BurnEveryTime")?,
            WhitelistMintMode::NeverBurn => out.write_all(b"NeverBurn")?,
        }
        Ok(IsNull::No)
    }
}

impl FromSql<Mode, Pg> for WhitelistMintMode {
    fn from_sql(bytes: Option<&[u8]>) -> deserialize::Result<Self> {
        match not_none!(bytes) {
            b"BurnEveryTime" => Ok(WhitelistMintMode::BurnEveryTime),
            b"NeverBurn" => Ok(WhitelistMintMode::NeverBurn),
            _ => Err("Unrecognized enum variant".into()),
        }
    }
}

#[derive(SqlType, Debug, Clone, Copy)]
#[postgres(type_name = "token_standard")]
/// Represents database `token_standard` type
pub struct TokenStandard;

#[derive(Debug, PartialEq, FromSqlRow, AsExpression, Clone, Copy)]
#[sql_type = "TokenStandard"]
/// `TokenStandard` enum in `Metadata` struct
pub enum TokenStandardEnum {
    /// This is a master edition
    NonFungible,
    /// A token with metadata that can also have attributes, sometimes called Semi Fungible
    FungibleAsset,
    /// A token with simple metadata
    Fungible,
    /// This is a limited edition
    NonFungibleEdition,
}

impl ToSql<TokenStandard, Pg> for TokenStandardEnum {
    fn to_sql<W: Write>(&self, out: &mut Output<W, Pg>) -> serialize::Result {
        match *self {
            TokenStandardEnum::NonFungible => out.write_all(b"NonFungible")?,
            TokenStandardEnum::FungibleAsset => out.write_all(b"FungibleAsset")?,
            TokenStandardEnum::Fungible => out.write_all(b"Fungible")?,
            TokenStandardEnum::NonFungibleEdition => out.write_all(b"NonFungibleEdition")?,
        }
        Ok(IsNull::No)
    }
}

impl FromSql<TokenStandard, Pg> for TokenStandardEnum {
    fn from_sql(bytes: Option<&[u8]>) -> deserialize::Result<Self> {
        match not_none!(bytes) {
            b"NonFungible" => Ok(TokenStandardEnum::NonFungible),
            b"FungibleAsset" => Ok(TokenStandardEnum::FungibleAsset),
            b"Fungible" => Ok(TokenStandardEnum::Fungible),
            b"NonFungibleEdition" => Ok(TokenStandardEnum::NonFungibleEdition),
            _ => Err("invalid enum entry".into()),
        }
    }
}

/// An offer event lifecycle
#[derive(SqlType, Debug, Clone, Copy)]
#[postgres(type_name = "offereventlifecycle")]
/// Represents database `offereventlifecycle` type
pub struct OfferEventLifecycle;

#[derive(Debug, PartialEq, FromSqlRow, AsExpression, Clone, Copy)]
#[sql_type = "OfferEventLifecycle"]
/// `OfferEventLifecycle` enum in `OfferEvents` struct
pub enum OfferEventLifecycleEnum {
    /// An offer was made on NFT
    Created,
    /// An offer was cancelled on NFT
    Cancelled,
}

impl ToSql<OfferEventLifecycle, Pg> for OfferEventLifecycleEnum {
    fn to_sql<W: Write>(&self, out: &mut Output<W, Pg>) -> serialize::Result {
        match *self {
            OfferEventLifecycleEnum::Created => out.write_all(b"Created")?,
            OfferEventLifecycleEnum::Cancelled => out.write_all(b"Cancelled")?,
        }
        Ok(IsNull::No)
    }
}

impl FromSql<OfferEventLifecycle, Pg> for OfferEventLifecycleEnum {
    fn from_sql(bytes: Option<&[u8]>) -> deserialize::Result<Self> {
        match not_none!(bytes) {
            b"Created" => Ok(OfferEventLifecycleEnum::Created),
            b"Cancelled" => Ok(OfferEventLifecycleEnum::Cancelled),
            _ => Err("invalid enum entry".into()),
        }
    }
}

impl fmt::Display for OfferEventLifecycleEnum {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            OfferEventLifecycleEnum::Created => write!(f, "Created"),
            OfferEventLifecycleEnum::Cancelled => write!(f, "Cancelled"),
        }
    }
}

/// A listing event lifecycle
#[derive(SqlType, Debug, Clone, Copy)]
#[postgres(type_name = "listingeventlifecycle")]
/// Represents database `listingeventlifecycle` type
pub struct ListingEventLifecycle;

#[derive(Debug, PartialEq, FromSqlRow, AsExpression, Clone, Copy)]
#[sql_type = "ListingEventLifecycle"]
/// `OfferEventLifecycle` enum in `OfferEvents` struct
pub enum ListingEventLifecycleEnum {
    /// A listing was created
    Created,
    /// A listing was cancelled
    Cancelled,
}

impl ToSql<ListingEventLifecycle, Pg> for ListingEventLifecycleEnum {
    fn to_sql<W: Write>(&self, out: &mut Output<W, Pg>) -> serialize::Result {
        match *self {
            ListingEventLifecycleEnum::Created => out.write_all(b"Created")?,
            ListingEventLifecycleEnum::Cancelled => out.write_all(b"Cancelled")?,
        }
        Ok(IsNull::No)
    }
}

impl FromSql<ListingEventLifecycle, Pg> for ListingEventLifecycleEnum {
    fn from_sql(bytes: Option<&[u8]>) -> deserialize::Result<Self> {
        match not_none!(bytes) {
            b"Created" => Ok(ListingEventLifecycleEnum::Created),
            b"Cancelled" => Ok(ListingEventLifecycleEnum::Cancelled),
            _ => Err("invalid enum entry".into()),
        }
    }
}

impl fmt::Display for ListingEventLifecycleEnum {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ListingEventLifecycleEnum::Created => write!(f, "Created"),
            ListingEventLifecycleEnum::Cancelled => write!(f, "Cancelled"),
        }
    }
}
