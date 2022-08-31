use indexer_core::bigdecimal::{BigDecimal, ParseBigDecimalError, ToPrimitive};

use super::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct I64(i64);

#[graphql_scalar(description = "I64")]
impl<S: ScalarValue> GraphQLScalar for I64 {
    fn resolve(&self) -> Value {
        Value::scalar(self.0.to_string())
    }

    fn from_input_value(v: &InputValue) -> Option<Self> {
        v.as_string_value().and_then(|s| s.parse().ok()).map(Self)
    }

    fn from_str<'a>(value: ScalarToken<'a>) -> ParseScalarResult<'a, S> {
        <String as ParseScalarValue<S>>::from_str(value)
    }
}

impl From<i64> for I64 {
    fn from(value: i64) -> Self {
        Self(value)
    }
}

impl TryFrom<u64> for I64 {
    type Error = std::num::TryFromIntError;

    fn try_from(value: u64) -> Result<Self, Self::Error> {
        value.try_into().map(Self)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct U64(u64);

#[graphql_scalar(description = "U64")]
impl<S: ScalarValue> GraphQLScalar for U64 {
    fn resolve(&self) -> Value {
        Value::scalar(self.0.to_string())
    }

    fn from_input_value(v: &InputValue) -> Option<Self> {
        v.as_string_value().and_then(|s| s.parse().ok()).map(Self)
    }

    fn from_str<'a>(value: ScalarToken<'a>) -> ParseScalarResult<'a, S> {
        <String as ParseScalarValue<S>>::from_str(value)
    }
}

impl From<u64> for U64 {
    fn from(value: u64) -> Self {
        Self(value)
    }
}

impl TryFrom<i64> for U64 {
    type Error = std::num::TryFromIntError;

    fn try_from(value: i64) -> Result<Self, Self::Error> {
        value.try_into().map(Self)
    }
}

impl TryFrom<BigDecimal> for U64 {
    type Error = ParseBigDecimalError;

    fn try_from(value: BigDecimal) -> Result<Self, Self::Error> {
        value
            .to_u64()
            .ok_or_else(|| {
                ParseBigDecimalError::Other(String::from("Integer is too large to store."))
            })
            .map(Self)
    }
}
