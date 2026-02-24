
use neo4rs::{Graph, Query, Txn, BoltType as Value};
use cyphr_core::traits::{FromCyphr, ToCyphrParams};
use cyphr_core::error::CyphrError;
use crate::stream::CyphrStream;

/// A typed query wrapper around [`neo4rs::Query`].
///
/// Provides a builder-style `.param()` API and typed fetch helpers that
/// automatically map rows via [`FromCyphr`].
///
/// # Examples
///
/// ```rust,no_run
/// # use cyphr::query::CyphrQuery;
/// # async fn example(graph: &neo4rs::Graph) -> Result<(), cyphr::CyphrError> {
/// let query = CyphrQuery::new("MATCH (u:User {name: $name}) RETURN u")
///     .param("name", "Alice");
/// # Ok(())
/// # }
/// ```
///
/// Or use the [`cypher_query!`](crate::cypher_query) macro for parameter
/// binding at compile time:
///
/// ```rust,ignore
/// let name = "Alice";
/// let query = cypher_query! { MATCH (u:User {name: $name}) RETURN u };
/// ```
pub struct CyphrQuery {
    inner: Query,
}

impl CyphrQuery {
    /// Create a new query from a Cypher string.
    pub fn new(query: impl Into<String>) -> Self {
        let q: String = query.into();
        Self { inner: neo4rs::query(&q) }
    }

    /// Bind a named parameter. Accepts any type that converts to `BoltType`.
    ///
    /// ```rust,no_run
    /// # use cyphr::query::CyphrQuery;
    /// let q = CyphrQuery::new("MATCH (u:User {age: $age}) RETURN u")
    ///     .param("age", 30_i64);
    /// ```
    pub fn param(mut self, key: impl Into<String>, value: impl Into<Value>) -> Self {
        let k: String = key.into();
        self.inner = self.inner.param(&k, value.into());
        self
    }

    /// Bind all fields from a [`ToCyphrParams`] struct as named parameters.
    ///
    /// This is a bulk alternative to calling [`.param()`](Self::param)
    /// for each field individually.
    ///
    /// ```rust,no_run
    /// # use cyphr::query::CyphrQuery;
    /// # #[derive(cyphr::ToCyphrParams)]
    /// # struct CreateUser { name: String, age: i64 }
    /// let params = CreateUser { name: "Alice".into(), age: 30 };
    /// let q = CyphrQuery::new("CREATE (u:User {name: $name, age: $age})")
    ///     .params_from(params);
    /// ```
    pub fn params_from(mut self, source: impl ToCyphrParams) -> Self {
        for (k, v) in source.to_params() {
            self.inner = self.inner.param(&k, v);
        }
        self
    }

    /// Execute against a [`Graph`] and return exactly one row, mapped to `T`.
    ///
    /// Returns [`CyphrError::MissingField`] if the result set is empty.
    pub async fn fetch_one<T: FromCyphr>(self, graph: &Graph) -> Result<T, CyphrError> {
        let mut stream = graph.execute(self.inner).await?;
        let row = stream.next().await?
            .ok_or_else(|| CyphrError::missing_field("row", "fetch_one"))?;
        T::from_record(&row)
    }

    /// Execute against a [`Graph`] and collect all rows into `Vec<T>`.
    pub async fn fetch_all<T: FromCyphr>(self, graph: &Graph) -> Result<Vec<T>, CyphrError> {
        let mut stream = graph.execute(self.inner).await?;
        let mut out = Vec::new();
        while let Some(row) = stream.next().await? {
            out.push(T::from_record(&row)?);
        }
        Ok(out)
    }

    /// Execute against a [`Graph`] and return zero or one row, mapped to `T`.
    ///
    /// Returns `Ok(None)` if the result set is empty, `Ok(Some(T))` if
    /// exactly one row was found.
    ///
    /// ```rust,no_run
    /// # use cyphr::query::CyphrQuery;
    /// # use cyphr::CyphrError;
    /// # #[derive(cyphr::FromCyphr)] struct UserRow { name: String }
    /// # async fn example(graph: &neo4rs::Graph) -> Result<(), CyphrError> {
    /// let user: Option<UserRow> = CyphrQuery::new("MATCH (u:User {id: $id}) RETURN u.name AS name")
    ///     .param("id", 1_i64)
    ///     .fetch_optional(graph)
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn fetch_optional<T: FromCyphr>(self, graph: &Graph) -> Result<Option<T>, CyphrError> {
        let mut stream = graph.execute(self.inner).await?;
        match stream.next().await? {
            Some(row) => Ok(Some(T::from_record(&row)?)),
            None => Ok(None),
        }
    }

    /// Execute within a [`Txn`] and return zero or one row, mapped to `T`.
    ///
    /// Like [`fetch_optional`](Self::fetch_optional) but runs inside an existing
    /// transaction.
    pub async fn fetch_optional_in<T: FromCyphr>(self, txn: &mut Txn) -> Result<Option<T>, CyphrError> {
        let mut stream = txn.execute(self.inner).await?;
        match stream.next(txn.handle()).await? {
            Some(row) => Ok(Some(T::from_record(&row)?)),
            None => Ok(None),
        }
    }

    /// Execute within a [`Txn`] and return exactly one row, mapped to `T`.
    ///
    /// Like [`fetch_one`](Self::fetch_one) but runs inside an existing
    /// transaction. The stream is driven through `txn.handle()`.
    ///
    /// ```rust,no_run
    /// # use cyphr::query::CyphrQuery;
    /// # use cyphr::CyphrError;
    /// # async fn example(graph: &neo4rs::Graph) -> Result<(), CyphrError> {
    /// # #[derive(cyphr::FromCyphr)] struct UserRow { name: String }
    /// let mut txn = graph.start_txn().await.unwrap();
    /// let row: UserRow = CyphrQuery::new("MATCH (u:User) RETURN u.name AS name LIMIT 1")
    ///     .fetch_one_in(&mut txn)
    ///     .await?;
    /// txn.commit().await.unwrap();
    /// # Ok(())
    /// # }
    /// ```
    pub async fn fetch_one_in<T: FromCyphr>(self, txn: &mut Txn) -> Result<T, CyphrError> {
        let mut stream = txn.execute(self.inner).await?;
        let row = stream.next(txn.handle()).await?
            .ok_or_else(|| CyphrError::missing_field("row", "fetch_one_in"))?;
        T::from_record(&row)
    }

    /// Execute within a [`Txn`] and collect all rows into `Vec<T>`.
    ///
    /// Like [`fetch_all`](Self::fetch_all) but runs inside an existing
    /// transaction.
    pub async fn fetch_all_in<T: FromCyphr>(self, txn: &mut Txn) -> Result<Vec<T>, CyphrError> {
        let mut stream = txn.execute(self.inner).await?;
        let mut out = Vec::new();
        while let Some(row) = stream.next(txn.handle()).await? {
            out.push(T::from_record(&row)?);
        }
        Ok(out)
    }

    /// Execute against a [`Graph`] and return a streaming iterator of `T`.
    ///
    /// Unlike [`fetch_all`](Self::fetch_all), rows are converted one at a
    /// time as they arrive from the server, keeping memory usage constant.
    ///
    /// Only available for [`Graph`] connections, not transactions (use
    /// [`fetch_all_in`](Self::fetch_all_in) for transactions).
    ///
    /// ```rust,no_run
    /// # use cyphr::query::CyphrQuery;
    /// # use cyphr::CyphrError;
    /// # #[derive(cyphr::FromCyphr)] struct UserRow { name: String }
    /// # async fn example(graph: &neo4rs::Graph) -> Result<(), CyphrError> {
    /// let mut stream = CyphrQuery::new("MATCH (u:User) RETURN u.name AS name")
    ///     .fetch_stream::<UserRow>(graph)
    ///     .await?;
    ///
    /// while let Some(result) = stream.next().await {
    ///     let user = result?;
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn fetch_stream<T: FromCyphr>(self, graph: &Graph) -> Result<CyphrStream<T>, CyphrError> {
        use futures::TryStreamExt;
        let detached = graph.execute(self.inner).await?;
        let stream = detached.into_stream().into_stream();
        Ok(CyphrStream::new(Box::pin(stream)))
    }
}

/// Convenience constructor — equivalent to [`CyphrQuery::new`].
///
/// ```rust,no_run
/// # use cyphr::query;
/// let q = query::query("MATCH (u:User) RETURN u");
/// ```
pub fn query(q: impl Into<String>) -> CyphrQuery {
    CyphrQuery::new(q)
}
