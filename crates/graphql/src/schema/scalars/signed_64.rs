use super::prelude::*;

#[derive(Debug, Clone, Copy)]
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
