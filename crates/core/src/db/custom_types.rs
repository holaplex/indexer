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

/// SPL Governance account type
#[derive(SqlType, Debug, Clone, Copy)]
#[postgres(type_name = "governanceaccounttype")]
/// Represents database `governanceaccounttype` enum
pub struct GovernanceAccountType;

#[derive(
    Debug, PartialEq, FromSqlRow, AsExpression, Clone, Copy, strum::EnumString, strum::Display,
)]
#[allow(missing_docs)]
#[sql_type = "GovernanceAccountType"]
pub enum GovernanceAccountTypeEnum {
    Uninitialized,
    RealmV1,
    TokenOwnerRecordV1,
    GovernanceV1,
    ProgramGovernanceV1,
    ProposalV1,
    SignatoryRecordV1,
    VoteRecordV1,
    ProposalInstructionV1,
    MintGovernanceV1,
    TokenGovernanceV1,
    RealmConfig,
    VoteRecordV2,
    ProposalTransactionV2,
    ProposalV2,
    ProgramMetadata,
    RealmV2,
    TokenOwnerRecordV2,
    GovernanceV2,
    ProgramGovernanceV2,
    MintGovernanceV2,
    TokenGovernanceV2,
    SignatoryRecordV2,
}

impl ToSql<GovernanceAccountType, Pg> for GovernanceAccountTypeEnum {
    fn to_sql<W: Write>(&self, out: &mut Output<W, Pg>) -> serialize::Result {
        to_bytes(self, out, |_| false)
    }
}

impl FromSql<GovernanceAccountType, Pg> for GovernanceAccountTypeEnum {
    fn from_sql(bytes: Option<&[u8]>) -> deserialize::Result<Self> {
        from_bytes(bytes)
    }
}

/// `VoteThreshold` type
#[derive(SqlType, Debug, Clone, Copy)]
#[postgres(type_name = "votethresholdtype")]
/// Represents database `votethresholdtype` enum
pub struct VoteThresholdType;

#[derive(
    Debug, PartialEq, FromSqlRow, AsExpression, Clone, Copy, strum::EnumString, strum::Display,
)]
#[allow(missing_docs)]
#[sql_type = "VoteThresholdType"]
pub enum VoteThresholdEnum {
    YesVote,
    Quorum,
}

impl ToSql<VoteThresholdType, Pg> for VoteThresholdEnum {
    fn to_sql<W: Write>(&self, out: &mut Output<W, Pg>) -> serialize::Result {
        to_bytes(self, out, |_| false)
    }
}

impl FromSql<VoteThresholdType, Pg> for VoteThresholdEnum {
    fn from_sql(bytes: Option<&[u8]>) -> deserialize::Result<Self> {
        from_bytes(bytes)
    }
}

/// `VoteWeightSource` type
#[derive(SqlType, Debug, Clone, Copy)]
/// Represents database `voteweightsourcce` enum
#[postgres(type_name = "votetipping")]
pub struct VoteTipping;

#[derive(
    Debug, PartialEq, FromSqlRow, AsExpression, Clone, Copy, strum::EnumString, strum::Display,
)]
#[sql_type = "VoteTipping"]
#[allow(missing_docs)]
pub enum VoteTippingEnum {
    Strict,
    Early,
    Disabled,
}

impl ToSql<VoteTipping, Pg> for VoteTippingEnum {
    fn to_sql<W: Write>(&self, out: &mut Output<W, Pg>) -> serialize::Result {
        to_bytes(self, out, |_| false)
    }
}

impl FromSql<VoteTipping, Pg> for VoteTippingEnum {
    fn from_sql(bytes: Option<&[u8]>) -> deserialize::Result<Self> {
        from_bytes(bytes)
    }
}

/// `MintMaxVote` type
#[derive(SqlType, Debug, Clone, Copy)]
/// Represents database `mintmaxvotetype` enum
#[postgres(type_name = "mintmaxvotetype")]
pub struct MintMaxVoteType;

#[derive(
    Debug, PartialEq, FromSqlRow, AsExpression, Clone, Copy, strum::EnumString, strum::Display,
)]
#[sql_type = "MintMaxVoteType"]
#[allow(missing_docs)]
pub enum MintMaxVoteEnum {
    SupplyFraction,
    Absolute,
}

impl ToSql<MintMaxVoteType, Pg> for MintMaxVoteEnum {
    fn to_sql<W: Write>(&self, out: &mut Output<W, Pg>) -> serialize::Result {
        to_bytes(self, out, |_| false)
    }
}

impl FromSql<MintMaxVoteType, Pg> for MintMaxVoteEnum {
    fn from_sql(bytes: Option<&[u8]>) -> deserialize::Result<Self> {
        from_bytes(bytes)
    }
}

/// `VoteRecordV2 Vote` type
#[derive(SqlType, Debug, Clone, Copy)]
/// Represents database `vote_record_v2_vote` enum
#[postgres(type_name = "vote_record_v2_vote")]
pub struct VoteRecordV2Vote;

#[derive(
    Debug, PartialEq, FromSqlRow, AsExpression, Clone, Copy, strum::EnumString, strum::Display,
)]
#[sql_type = "VoteRecordV2Vote"]
#[allow(missing_docs)]
pub enum VoteRecordV2VoteEnum {
    Approve,
    Deny,
    Abstain,
    Veto,
}

impl ToSql<VoteRecordV2Vote, Pg> for VoteRecordV2VoteEnum {
    fn to_sql<W: Write>(&self, out: &mut Output<W, Pg>) -> serialize::Result {
        to_bytes(self, out, |_| false)
    }
}

impl FromSql<VoteRecordV2Vote, Pg> for VoteRecordV2VoteEnum {
    fn from_sql(bytes: Option<&[u8]>) -> deserialize::Result<Self> {
        from_bytes(bytes)
    }
}

/// Direction for ordering SQL query results by the "ORDER BY" variable(s)
#[derive(Debug, Clone, Copy, strum::EnumString, strum::Display)]
pub enum OrderDirection {
    /// order results descending
    #[strum(serialize = "DESC")]
    Desc,

    /// order results ascending
    #[strum(serialize = "ASC")]
    Asc,
}

/// Direction for sorting SQL query results by the "SORT BY" variable(s)
#[derive(Debug, Clone, Copy, strum::EnumString, strum::Display)]
pub enum NftSort {
    /// sort results by Price
    #[strum(serialize = "Price")]
    Price,

    /// sort results by ListedAt
    #[strum(serialize = "ListedAt")]
    ListedAt,
}

/// `ProposalV2State`
#[derive(SqlType, Debug, Clone, Copy)]
/// Represents database `proposalstate` enum
#[postgres(type_name = "proposalstate")]
pub struct ProposalState;

#[derive(
    Debug, PartialEq, FromSqlRow, AsExpression, Clone, Copy, strum::EnumString, strum::Display,
)]
#[sql_type = "ProposalState"]
#[allow(missing_docs)]
pub enum ProposalStateEnum {
    Draft,
    SigningOff,
    Voting,
    Succeeded,
    Executing,
    Completed,
    Cancelled,
    Defeated,
    ExecutingWithErrors,
}

impl ToSql<ProposalState, Pg> for ProposalStateEnum {
    fn to_sql<W: Write>(&self, out: &mut Output<W, Pg>) -> serialize::Result {
        to_bytes(self, out, |_| false)
    }
}

impl FromSql<ProposalState, Pg> for ProposalStateEnum {
    fn from_sql(bytes: Option<&[u8]>) -> deserialize::Result<Self> {
        from_bytes(bytes)
    }
}

/// `InstructionExecutionFlags`
#[derive(SqlType, Debug, Clone, Copy)]
/// Represents database `instructionexecutionflags` enum
#[postgres(type_name = "instructionexecutionflags")]
pub struct InstructionExecutionFlags;

#[derive(
    Debug, PartialEq, FromSqlRow, AsExpression, Clone, Copy, strum::EnumString, strum::Display,
)]
#[sql_type = "InstructionExecutionFlags"]
#[allow(missing_docs)]
pub enum InstructionExecutionFlagsEnum {
    None,
    Ordered,
    UseTransaction,
}

impl ToSql<InstructionExecutionFlags, Pg> for InstructionExecutionFlagsEnum {
    fn to_sql<W: Write>(&self, out: &mut Output<W, Pg>) -> serialize::Result {
        to_bytes(self, out, |_| false)
    }
}

impl FromSql<InstructionExecutionFlags, Pg> for InstructionExecutionFlagsEnum {
    fn from_sql(bytes: Option<&[u8]>) -> deserialize::Result<Self> {
        from_bytes(bytes)
    }
}

/// `ProposalV2VoteType`
#[derive(SqlType, Debug, Clone, Copy)]
/// Represents database `votetype` enum
#[postgres(type_name = "proposalvotetype")]
pub struct ProposalVoteType;

#[derive(
    Debug, PartialEq, FromSqlRow, AsExpression, Clone, Copy, strum::EnumString, strum::Display,
)]
#[sql_type = "ProposalVoteType"]
#[allow(missing_docs)]
pub enum ProposalVoteTypeEnum {
    SingleChoice,
    MultiChoice,
}

impl ToSql<ProposalVoteType, Pg> for ProposalVoteTypeEnum {
    fn to_sql<W: Write>(&self, out: &mut Output<W, Pg>) -> serialize::Result {
        to_bytes(self, out, |_| false)
    }
}

impl FromSql<ProposalVoteType, Pg> for ProposalVoteTypeEnum {
    fn from_sql(bytes: Option<&[u8]>) -> deserialize::Result<Self> {
        from_bytes(bytes)
    }
}

/// `ProposalV2 OptionVoteResult`
#[derive(SqlType, Debug, Clone, Copy)]
/// Represents database `optionvoteresult` enum
#[postgres(type_name = "optionvoteresult")]
pub struct OptionVoteResult;

#[derive(
    Debug, PartialEq, FromSqlRow, AsExpression, Clone, Copy, strum::EnumString, strum::Display,
)]
#[sql_type = "OptionVoteResult"]
#[allow(missing_docs)]
pub enum OptionVoteResultEnum {
    None,
    Succeeded,
    Defeated,
}

impl ToSql<OptionVoteResult, Pg> for OptionVoteResultEnum {
    fn to_sql<W: Write>(&self, out: &mut Output<W, Pg>) -> serialize::Result {
        to_bytes(self, out, |_| false)
    }
}

impl FromSql<OptionVoteResult, Pg> for OptionVoteResultEnum {
    fn from_sql(bytes: Option<&[u8]>) -> deserialize::Result<Self> {
        from_bytes(bytes)
    }
}

/// `SPL Governance ProposalTransactionV2` execution status
#[derive(SqlType, Debug, Clone, Copy)]
/// Represents database `transactionexecutionstatus` enum
#[postgres(type_name = "transactionexecutionstatus")]
pub struct TransactionExecutionStatus;

#[derive(
    Debug, PartialEq, FromSqlRow, AsExpression, Clone, Copy, strum::EnumString, strum::Display,
)]
#[sql_type = "TransactionExecutionStatus"]
#[allow(missing_docs)]
pub enum TransactionExecutionStatusEnum {
    None,
    Success,
    Error,
}

impl ToSql<TransactionExecutionStatus, Pg> for TransactionExecutionStatusEnum {
    fn to_sql<W: Write>(&self, out: &mut Output<W, Pg>) -> serialize::Result {
        to_bytes(self, out, |_| false)
    }
}

impl FromSql<TransactionExecutionStatus, Pg> for TransactionExecutionStatusEnum {
    fn from_sql(bytes: Option<&[u8]>) -> deserialize::Result<Self> {
        from_bytes(bytes)
    }
}

/// `SPL Governance VoteRecordV1` vote weight type
#[derive(SqlType, Debug, Clone, Copy)]
/// Represents database `voteweightv1` enum
#[postgres(type_name = "voteweightv1")]
pub struct VoteWeightV1;

#[derive(
    Debug, PartialEq, FromSqlRow, AsExpression, Clone, Copy, strum::EnumString, strum::Display,
)]
#[sql_type = "VoteWeightV1"]
#[allow(missing_docs)]
pub enum VoteWeightV1Enum {
    Yes,
    No,
}

impl ToSql<VoteWeightV1, Pg> for VoteWeightV1Enum {
    fn to_sql<W: Write>(&self, out: &mut Output<W, Pg>) -> serialize::Result {
        to_bytes(self, out, |_| false)
    }
}

impl FromSql<VoteWeightV1, Pg> for VoteWeightV1Enum {
    fn from_sql(bytes: Option<&[u8]>) -> deserialize::Result<Self> {
        from_bytes(bytes)
    }
}

/// Direction for sorting Collections query results by the "SORT BY" variable(s)
#[derive(Debug, Clone, Copy, strum::EnumString, strum::Display)]
pub enum CollectionSort {
    /// sort results by floor price
    FloorPrice,

    /// sort results by 1 day volume
    OneDayVolume,

    /// sort results by 7 days volume
    SevenDayVolume,

    /// sort results by 30 days volume]
    ThirtyDayVolume,

    /// sort results by 1 day sales count
    OneDaySalesCount,

    /// sort results by 7 days sales count
    SevenDaySalesCount,

    /// sort results by 30 days sales count
    ThirtyDaySalesCount,

    /// sort results by 1 day marketcap
    OneDayMarketcap,

    /// sort results by 7 day marketcap
    SevenDayMarketcap,

    /// sort results by 30 day marketcap
    ThirtyDayMarketcap,
}
