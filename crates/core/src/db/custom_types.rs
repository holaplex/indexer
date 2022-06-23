//! Includes `WhitelistMintMode` enum and `EndSettingType` enum

use std::io::Write;

use diesel::{
    deserialize::{self, FromSql},
    pg::Pg,
    serialize::{self, Output, ToSql},
    AsExpression, FromSqlRow, SqlType,
};

fn to_bytes<T: std::fmt::Display, W: Write, N: FnOnce(&T) -> bool>(
    val: &T,
    mut out: W,
    is_null: N,
) -> serialize::Result {
    use diesel::serialize::IsNull;

    out.write_fmt(format_args!("{}", val))?;

    Ok(if is_null(val) {
        IsNull::Yes
    } else {
        IsNull::No
    })
}

fn from_bytes<T: std::str::FromStr>(bytes: Option<&[u8]>) -> deserialize::Result<T>
where
    T::Err: Into<Box<dyn std::error::Error + Send + Sync>>,
{
    std::str::from_utf8(diesel::not_none!(bytes))?
        .parse()
        .map_err(Into::into)
}

#[derive(SqlType, Debug, Clone, Copy)]
#[postgres(type_name = "settingtype")]
/// Represents database 'settingtype' type
pub struct SettingType;

#[derive(
    Debug, PartialEq, FromSqlRow, AsExpression, Clone, Copy, strum::EnumString, strum::Display,
)]
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
        to_bytes(self, out, |_| false)
    }
}

impl FromSql<SettingType, Pg> for EndSettingType {
    fn from_sql(bytes: Option<&[u8]>) -> deserialize::Result<Self> {
        from_bytes(bytes)
    }
}

#[derive(SqlType, Debug, Clone, Copy)]
#[postgres(type_name = "mode")]
/// Represents database 'mode' type
pub struct Mode;

#[derive(
    Debug, PartialEq, FromSqlRow, AsExpression, Clone, Copy, strum::EnumString, strum::Display,
)]
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
        to_bytes(self, out, |_| false)
    }
}

impl FromSql<Mode, Pg> for WhitelistMintMode {
    fn from_sql(bytes: Option<&[u8]>) -> deserialize::Result<Self> {
        from_bytes(bytes)
    }
}

#[derive(SqlType, Debug, Clone, Copy)]
#[postgres(type_name = "token_standard")]
/// Represents database `token_standard` type
pub struct TokenStandard;

#[derive(
    Debug, PartialEq, FromSqlRow, AsExpression, Clone, Copy, strum::EnumString, strum::Display,
)]
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
        to_bytes(self, out, |_| false)
    }
}

impl FromSql<TokenStandard, Pg> for TokenStandardEnum {
    fn from_sql(bytes: Option<&[u8]>) -> deserialize::Result<Self> {
        from_bytes(bytes)
    }
}

/// An offer event lifecycle
#[derive(SqlType, Debug, Clone, Copy)]
#[postgres(type_name = "offereventlifecycle")]
/// Represents database `offereventlifecycle` type
pub struct OfferEventLifecycle;

#[derive(
    Debug, PartialEq, FromSqlRow, AsExpression, Clone, Copy, strum::EnumString, strum::Display,
)]
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
        to_bytes(self, out, |_| false)
    }
}

impl FromSql<OfferEventLifecycle, Pg> for OfferEventLifecycleEnum {
    fn from_sql(bytes: Option<&[u8]>) -> deserialize::Result<Self> {
        from_bytes(bytes)
    }
}

/// A listing event lifecycle
#[derive(SqlType, Debug, Clone, Copy)]
#[postgres(type_name = "listingeventlifecycle")]
/// Represents database `listingeventlifecycle` type
pub struct ListingEventLifecycle;

#[derive(
    Debug, PartialEq, FromSqlRow, AsExpression, Clone, Copy, strum::EnumString, strum::Display,
)]
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
        to_bytes(self, out, |_| false)
    }
}

impl FromSql<ListingEventLifecycle, Pg> for ListingEventLifecycleEnum {
    fn from_sql(bytes: Option<&[u8]>) -> deserialize::Result<Self> {
        from_bytes(bytes)
    }
}
