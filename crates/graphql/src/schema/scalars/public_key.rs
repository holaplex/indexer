use super::prelude::*;
use std::marker::PhantomData;

#[repr(transparent)]
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct PublicKey<T>(String, PhantomData<T>);

#[graphql_scalar(description = "PublicKey")]
impl<T, S: ScalarValue> GraphQLScalar for PublicKey<T> {
    fn resolve(&self) -> Value {
        Value::scalar(self.0.to_string())
    }

    fn from_input_value(v: &InputValue) -> Option<PublicKey<T>> {
        v.as_string_value().and_then(|s| s.parse().ok()).map(Self)
    }

    fn from_str<'a>(value: ScalarToken<'a>) -> ParseScalarResult<'a, S> {
        <String as ParseScalarValue<S>>::from_str(value)
    }
}

impl<T> std::fmt::Display for PublicKey<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        std::fmt::Display::fmt(&self.0, f)
    }
}

impl<T> AsRef<str> for PublicKey<T> {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl<T> AsMut<str> for PublicKey<T> {
    fn as_mut(&mut self) -> &mut str {
        &mut self.0
    }
}

impl<'a, T> From<std::borrow::Cow<'a, str>> for PublicKey<T> {
    fn from(c: std::borrow::Cow<str>) -> Self {
        Self(c.into_owned())
    }
}

impl<T> From<String> for PublicKey<T> {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl<T> From<PublicKey<T>> for String {
    fn from(s: PublicKey<T>) -> Self {
        s.0
    }
}

impl<T, U, B> indexer_core::db::serialize::ToSql<U, B> for PublicKey<T>
where
    B: indexer_core::db::Backend,
    String: indexer_core::db::serialize::ToSql<U, B>,
{
    fn to_sql<W: std::io::Write>(
        &self,
        out: &mut indexer_core::db::serialize::Output<W, B>,
    ) -> indexer_core::db::serialize::Result {
        self.0.to_sql(out)
    }
}
