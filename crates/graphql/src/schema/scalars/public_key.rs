use super::prelude::*;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct PublicKey(String);

#[graphql_scalar(description = "PublicKey")]
impl<S> GraphQLScalar for PublicKey
where
    S: ScalarValue,
{
    fn resolve(&self) -> Value {
        Value::scalar(self.0.to_string())
    }

    fn from_input_value(v: &InputValue) -> Option<PublicKey> {
        v.as_string_value().and_then(|s| s.parse().ok()).map(Self)
    }

    fn from_str<'a>(value: ScalarToken<'a>) -> ParseScalarResult<'a, S> {
        <String as ParseScalarValue<S>>::from_str(value)
    }
}

impl std::fmt::Display for PublicKey {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        std::fmt::Display::fmt(&self.0, f)
    }
}

impl AsRef<str> for PublicKey {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl AsMut<str> for PublicKey {
    fn as_mut(&mut self) -> &mut str {
        &mut self.0
    }
}

impl<'a> From<std::borrow::Cow<'a, str>> for PublicKey {
    fn from(c: std::borrow::Cow<str>) -> Self {
        Self(c.into_owned())
    }
}

impl From<String> for PublicKey {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl From<PublicKey> for String {
    fn from(s: PublicKey) -> Self {
        s.0
    }
}

impl<T, B> indexer_core::db::serialize::ToSql<T, B> for PublicKey
where
    B: indexer_core::db::Backend,
    String: indexer_core::db::serialize::ToSql<T, B>,
{
    fn to_sql<W: std::io::Write>(
        &self,
        out: &mut indexer_core::db::serialize::Output<W, B>,
    ) -> indexer_core::db::serialize::Result {
        self.0.to_sql(out)
    }
}
