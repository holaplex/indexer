use std::{
    cmp::Ordering,
    fmt,
    fmt::{Debug, Display, Formatter},
    hash::Hash,
    marker::PhantomData,
};

use super::prelude::*;

#[repr(transparent)]
pub struct PublicKey<T: 'static>(String, PhantomData<&'static T>);

// Safety: PublicKey only contains a string
unsafe impl<T> Send for PublicKey<T> {}
unsafe impl<T> Sync for PublicKey<T> {}

impl<T> PartialEq for PublicKey<T> {
    fn eq(&self, rhs: &Self) -> bool {
        self.0.eq(&rhs.0)
    }
}

impl<T> Eq for PublicKey<T> {}

impl<T> PartialOrd for PublicKey<T> {
    fn partial_cmp(&self, rhs: &Self) -> Option<Ordering> {
        Some(self.0.cmp(&rhs.0))
    }
}

impl<T> Ord for PublicKey<T> {
    fn cmp(&self, rhs: &Self) -> Ordering {
        self.0.cmp(&rhs.0)
    }
}

impl<T> Clone for PublicKey<T> {
    fn clone(&self) -> Self {
        Self(self.0.clone(), PhantomData::default())
    }
}

impl<T> Display for PublicKey<T> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        Display::fmt(&self.0, f)
    }
}

impl<T> Debug for PublicKey<T> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        Debug::fmt(&self.0, f)
    }
}

impl<T> Hash for PublicKey<T> {
    fn hash<H: std::hash::Hasher>(&self, h: &mut H) {
        self.0.hash(h);
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
        Self(c.into_owned(), PhantomData::default())
    }
}

impl<T> From<String> for PublicKey<T> {
    fn from(s: String) -> Self {
        Self(s, PhantomData::default())
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

impl<T, U, B> indexer_core::db::Queryable<U, B> for PublicKey<T>
where
    B: indexer_core::db::Backend,
    String: indexer_core::db::Queryable<U, B>,
{
    type Row = <String as indexer_core::db::Queryable<U, B>>::Row;

    fn build(row: Self::Row) -> Self {
        Self(
            <String as indexer_core::db::Queryable<U, B>>::build(row),
            PhantomData::default(),
        )
    }
}

//// BUG: juniper v0.15 does not support implementing GraphQLScalar on structs
////      with generic parameters.  As a result the expansion of this macro has
////      been manually included in the code and tweaked to compile, and should
////      be converted to #[derive(GraphQLScalar)] once the fix for this bug has
////      been released.

// #[graphql_scalar(description = "PublicKey")]
// impl<T, S: ScalarValue> GraphQLScalar for PublicKey<T> {
//     fn resolve(&self) -> Value {
//         Value::scalar(self.0.to_string())
//     }

//     fn from_input_value(v: &InputValue) -> Option<PublicKey<T>> {
//         v.as_string_value().and_then(|s| s.parse().ok()).map(Self)
//     }

//     fn from_str<'a>(value: ScalarToken<'a>) -> ParseScalarResult<'a, S> {
//         <String as ParseScalarValue<S>>::from_str(value)
//     }
// }

#[automatically_derived]
impl<T, S> ::juniper::marker::IsInputType<S> for PublicKey<T> where S: ::juniper::ScalarValue {}
#[automatically_derived]
impl<T, S> ::juniper::marker::IsOutputType<S> for PublicKey<T> where S: ::juniper::ScalarValue {}
#[automatically_derived]
impl<T, S> ::juniper::GraphQLType<S> for PublicKey<T>
where
    S: ::juniper::ScalarValue,
{
    fn name(_: &Self::TypeInfo) -> Option<&'static str> {
        Some("PublicKey")
    }
    fn meta<'r>(
        info: &Self::TypeInfo,
        registry: &mut ::juniper::Registry<'r, S>,
    ) -> ::juniper::meta::MetaType<'r, S>
    where
        S: 'r,
    {
        registry
            .build_scalar_type::<Self>(info)
            .description("PublicKey")
            .into_meta()
    }
}
#[automatically_derived]
impl<T, S> ::juniper::GraphQLValue<S> for PublicKey<T>
where
    S: ::juniper::ScalarValue,
{
    type Context = ();
    type TypeInfo = ();
    fn type_name<'__i>(&self, info: &'__i Self::TypeInfo) -> Option<&'__i str> {
        <Self as ::juniper::GraphQLType<S>>::name(info)
    }
    fn resolve(
        &self,
        info: &(),
        selection: Option<&[::juniper::Selection<S>]>,
        executor: &::juniper::Executor<Self::Context, S>,
    ) -> ::juniper::ExecutionResult<S> {
        Ok(Value::scalar(self.0.to_string()))
    }
}
#[automatically_derived]
impl<T, S> ::juniper::GraphQLValueAsync<S> for PublicKey<T>
where
    Self: Sync,
    Self::TypeInfo: Sync,
    Self::Context: Sync,
    S: ::juniper::ScalarValue + Send + Sync,
{
    fn resolve_async<'a>(
        &'a self,
        info: &'a Self::TypeInfo,
        selection_set: Option<&'a [::juniper::Selection<S>]>,
        executor: &'a ::juniper::Executor<Self::Context, S>,
    ) -> ::juniper::BoxFuture<'a, ::juniper::ExecutionResult<S>> {
        use ::juniper::futures::future;
        let v = ::juniper::GraphQLValue::resolve(self, info, selection_set, executor);
        Box::pin(future::ready(v))
    }
}
#[automatically_derived]
impl<T, S> ::juniper::ToInputValue<S> for PublicKey<T>
where
    S: ::juniper::ScalarValue,
{
    fn to_input_value(&self) -> ::juniper::InputValue<S> {
        let v = { Value::scalar(self.0.to_string()) };
        ::juniper::ToInputValue::to_input_value(&v)
    }
}
#[automatically_derived]
impl<T, S> ::juniper::FromInputValue<S> for PublicKey<T>
where
    S: ::juniper::ScalarValue,
{
    fn from_input_value(v: &::juniper::InputValue<S>) -> Option<PublicKey<T>> {
        v.as_string_value()
            .and_then(|s| s.parse().ok())
            .map(|s| Self(s, PhantomData::default()))
    }
}
#[automatically_derived]
impl<T, S> ::juniper::ParseScalarValue<S> for PublicKey<T>
where
    S: ::juniper::ScalarValue,
{
    fn from_str(value: ::juniper::parser::ScalarToken) -> ParseScalarResult<S> {
        <String as ParseScalarValue<S>>::from_str(value)
    }
}
