use super::prelude::*;

#[derive(Debug, Clone, Copy)]
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
