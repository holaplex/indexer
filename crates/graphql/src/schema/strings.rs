macro_rules! strings {
    () => {};
    ($($id:ident),+) => { strings!($($id,)+); };
    ($id:ident, $($rest:tt)*) => {
        #[repr(transparent)]
        #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
        pub struct $id(String);

        impl std::fmt::Display for $id {
            fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                std::fmt::Display::fmt(&self.0, f)
            }
        }

        impl AsRef<str> for $id {
            fn as_ref(&self) -> &str { &self.0 }
        }

        impl AsMut<str> for $id {
            fn as_mut(&mut self) -> &mut str { &mut self.0 }
        }

        impl<'a> From<std::borrow::Cow<'a, str>> for $id {
            fn from(c: std::borrow::Cow<str>) -> Self { Self(c.into_owned()) }
        }

        impl From<String> for $id {
            fn from(s: String) -> Self { Self(s) }
        }

        impl From<$id> for String {
            fn from(s: $id) -> Self { s.0 }
        }

        impl<T, B> indexer_core::db::serialize::ToSql<T, B> for $id
            where B: indexer_core::db::Backend,
                  String: indexer_core::db::serialize::ToSql<T, B> {
            fn to_sql<W: std::io::Write>(
                &self,
                out: &mut indexer_core::db::serialize::Output<W, B>,
            ) -> indexer_core::db::serialize::Result {
                self.0.to_sql(out)
            }
        }

        strings!($($rest)*);
    };
}

strings![
    AuctionHouseAddress,
    ListingAddress,
    MetadataAddress,
    StorefrontAddress
];
