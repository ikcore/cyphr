
//! Streaming query results with automatic type mapping.

use std::pin::Pin;
use std::marker::PhantomData;
use futures::stream::Stream;
use cyphr_core::traits::FromCyphr;
use cyphr_core::error::CyphrError;

/// A typed stream of query results mapped via [`FromCyphr`].
///
/// Created by [`CyphrQuery::fetch_stream`](crate::query::CyphrQuery::fetch_stream).
/// Each call to [`next()`](Self::next) pulls the next row from the database
/// and maps it to `T`.
///
/// # Example
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
///     println!("{}", user.name);
/// }
/// # Ok(())
/// # }
/// ```
pub struct CyphrStream<T> {
    inner: Pin<Box<dyn Stream<Item = Result<neo4rs::Row, neo4rs::Error>> + Send>>,
    _marker: PhantomData<T>,
}

impl<T: FromCyphr> CyphrStream<T> {
    pub(crate) fn new(inner: Pin<Box<dyn Stream<Item = Result<neo4rs::Row, neo4rs::Error>> + Send>>) -> Self {
        Self { inner, _marker: PhantomData }
    }

    /// Pull the next row from the stream and map it to `T`.
    ///
    /// Returns `None` when the stream is exhausted.
    pub async fn next(&mut self) -> Option<Result<T, CyphrError>> {
        use futures::StreamExt;
        match self.inner.next().await {
            None => None,
            Some(Err(e)) => Some(Err(CyphrError::Neo4j(e))),
            Some(Ok(row)) => Some(T::from_record(&row)),
        }
    }
}
