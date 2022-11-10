use std::str::FromStr;

use indexer_core::bigdecimal::BigDecimal;

use super::prelude::*;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Numeric(pub BigDecimal);

#[graphql_scalar(description = "Numeric data type")]
impl<S: ScalarValue> GraphQLScalar for Numeric {
    fn resolve(&self) -> Value {
        Value::scalar(self.0.to_string())
    }

    fn from_input_value(v: &InputValue) -> Option<Numeric> {
        v.as_string_value()
            .and_then(|v| BigDecimal::from_str(v).ok().map(Numeric))
    }

    fn from_str<'a>(v: ScalarToken<'a>) -> ParseScalarResult<'a, S> {
        <String as ParseScalarValue<S>>::from_str(v)
    }
}

impl From<BigDecimal> for Numeric {
    fn from(v: BigDecimal) -> Self {
        Self(v)
    }
}
