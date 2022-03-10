//! Pagination with limit and offset parameters

use diesel::{
    pg::Pg,
    prelude::*,
    query_builder::{AstPass, Query, QueryFragment},
    query_dsl::methods::LoadQuery,
    sql_types::BigInt,
};

use crate::db::Connection;

// Paginate
pub trait Paginate: Sized {
    /// Takes pagination parameters and returns Pagination struct
    fn paginate(self, limit: Option<i64>, offset: Option<i64>) -> Pagination<Self>;
}

impl<T> Paginate for T {
    fn paginate(self, limit: Option<i64>, offset: Option<i64>) -> Pagination<Self> {
        Pagination {
            query: self,
            limit,
            offset,
        }
    }
}

/// Pagination struct holds query and pagination parameters
#[derive(Debug, Clone, Copy, QueryId)]
pub struct Pagination<T> {
    query: T,
    offset: Option<i64>,
    limit: Option<i64>,
}

impl<T> Pagination<T> {
    /// Loads Query results and count
    /// count is the total results to be returned if no pagination parameter is set
    ///
    /// # Errors
    /// returns an error if query results can not be loaded
    pub fn load_with_pagination<U>(self, conn: &Connection) -> QueryResult<(Vec<U>, i64)>
    where
        Self: LoadQuery<Connection, (U, i64)>,
    {
        let results = self.load::<(U, i64)>(conn)?;
        let total = results.get(0).map_or(0, |x| x.1);
        let records = results.into_iter().map(|x| x.0).collect();
        Ok((records, total))
    }
}

impl<T: Query> Query for Pagination<T> {
    type SqlType = (T::SqlType, BigInt);
}

impl<T> RunQueryDsl<PgConnection> for Pagination<T> {}

impl<T> QueryFragment<Pg> for Pagination<T>
where
    T: QueryFragment<Pg>,
{
    fn walk_ast(&self, mut out: AstPass<Pg>) -> QueryResult<()> {
        out.push_sql("SELECT *, COUNT(*) OVER () FROM (");
        self.query.walk_ast(out.reborrow())?;
        out.push_sql(") as t ");
        if let Some(limit) = self.limit {
            out.push_sql("LIMIT ");
            out.push_bind_param::<BigInt, _>(&limit)?;
        }

        if let Some(offset) = self.offset {
            out.push_sql(" OFFSET ");
            out.push_bind_param::<BigInt, _>(&offset)?;
        }

        Ok(())
    }
}
