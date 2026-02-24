#![doc = r#"
A lightweight, Cypher-first mapping layer for Neo4j in Rust.

`cyphr` keeps Cypher literal and copy/pasteable into Neo4j Browser while
adding strong Rust types around nodes, relationships, and query result
projections. Built on [`neo4rs`] 0.8.

# Quick start

## Define nodes and relationships

```rust
use cyphr::prelude::*;

#[derive(Debug, CyphrNode)]
#[cyphr(label = "User")]
struct User {
    id: i64,
    name: String,
}

#[derive(Debug, CyphrRelation)]
#[cyphr(type = "FOLLOWS", from = "User", to = "User")]
struct Follows {
    since: i64,
}
```

Both derives automatically implement `FromCyphrValue`, so you can convert
`BoltType::Node` / `BoltType::Relation` values directly without a manual
impl.

## Map query results

Since `#[derive(CyphrNode)]` auto-implements `FromCyphrValue`, you can use
node types directly as fields in `FromCyphr` structs:

```rust
use cyphr::prelude::*;

# #[derive(Debug, CyphrNode)]
# #[cyphr(label = "User")]
# struct User { id: i64, name: String }
#[derive(FromCyphr)]
struct UserRow {
    u: User,
    score: f64,
}
```

### Flatten nested projections

Use `#[cyphr(flatten)]` to share the same row across multiple `FromCyphr`
structs:

```rust
use cyphr::prelude::*;

#[derive(FromCyphr)]
struct Metadata {
    created: String,
}

#[derive(FromCyphr)]
struct UserRow {
    name: String,
    #[cyphr(flatten)]
    meta: Metadata,
}
```

Both `name` and `created` are read from the same `neo4rs::Row`.

## Write Cypher

```rust
use cyphr::prelude::*;

// String literal — copy/paste into Neo4j Browser:
let q: &str = cypher! { MATCH (u:User) RETURN u };

// Query builder with auto-bound parameters:
let name = "Alice";
let query = cypher_query! {
    MATCH (u:User {name: $name}) RETURN u
};
// Expands to: CyphrQuery::new("...").param("name", name)
```

## Execute queries

```rust,no_run
use cyphr::prelude::*;
use cyphr::query;

# #[derive(Debug, CyphrNode)]
# #[cyphr(label = "User")]
# struct User { id: i64, name: String }
# #[derive(FromCyphr)]
# struct UserRow { u: User }
# async fn example(graph: &neo4rs::Graph) -> Result<(), CyphrError> {
// Exactly one row (error if empty):
let user: UserRow = query::query("MATCH (u:User) RETURN u LIMIT 1")
    .fetch_one(graph)
    .await?;

// Zero or one row:
let maybe_user: Option<UserRow> = query::query("MATCH (u:User {id: 1}) RETURN u")
    .fetch_optional(graph)
    .await?;

// All rows at once:
let users: Vec<UserRow> = query::query("MATCH (u:User) RETURN u")
    .fetch_all(graph)
    .await?;

// Streaming — rows converted one at a time:
let mut stream = query::query("MATCH (u:User) RETURN u")
    .fetch_stream::<UserRow>(graph)
    .await?;
while let Some(result) = stream.next().await {
    let user = result?;
}

// Inside a transaction:
let mut txn = graph.start_txn().await.unwrap();
let users: Vec<UserRow> = query::query("MATCH (u:User) RETURN u")
    .fetch_all_in(&mut txn)
    .await?;
txn.commit().await.unwrap();
# Ok(())
# }
```

## Writing data with `ToCyphrParams`

Use `#[derive(ToCyphrParams)]` to convert a struct into named query
parameters, then bind them with `.params_from()`:

```rust,no_run
use cyphr::prelude::*;
use cyphr::query::CyphrQuery;

#[derive(ToCyphrParams)]
struct CreateUser {
    name: String,
    age: i64,
}

# async fn example(graph: &neo4rs::Graph) -> Result<(), CyphrError> {
let params = CreateUser { name: "Alice".into(), age: 30 };
let query = CyphrQuery::new("CREATE (u:User {name: $name, age: $age})")
    .params_from(params);
# Ok(())
# }
```

Fields marked `#[cyphr(skip)]` or `#[cyphr(id)]` are excluded from
the parameter map. Use `#[cyphr(prop = "...")]` to override the key name.

# Supported value types

`FromCyphrValue` conversions are provided for:

| Neo4j type | Rust type |
|------------|-----------|
| Integer | `i64`, `i32`, `u64`, `u32`, `i16`, `u16`, `i8`, `u8` |
| Float | `f64`, `f32` |
| String | `String` |
| Boolean | `bool` |
| List | `Vec<T>`, `(A, B)`, `(A, B, C)` |
| Map | `HashMap<String, V>` |
| Null | `Option<T>` |
| Node | `NodeWrapper<T>`, or directly via `#[derive(CyphrNode)]` |
| Relationship | `RelationWrapper<T>`, or directly via `#[derive(CyphrRelation)]` |
| Point2D | [`Point2D`] |
| Point3D | [`Point3D`] |
| Bytes | [`CyphrBytes`] |
| Path | [`CyphrPath<N>`] |
| Date | `chrono::NaiveDate` |
| LocalTime | `chrono::NaiveTime` |
| Time | `(chrono::NaiveTime, chrono::FixedOffset)` |
| LocalDateTime | `chrono::NaiveDateTime` |
| DateTime / DateTimeZoneId | `chrono::DateTime<chrono::FixedOffset>` |
| Duration | `std::time::Duration` |

# Error handling

All conversions return [`CyphrError`]. Derive macros automatically wrap
errors with `.with_context()` so messages include the struct, field, and
property name:

```text
User::name (prop 'name'): type mismatch: expected String, got Integer (String)
```

[`neo4rs`]: https://docs.rs/neo4rs
[`Point2D`]: cyphr_core::Point2D
[`Point3D`]: cyphr_core::Point3D
[`CyphrBytes`]: cyphr_core::CyphrBytes
[`CyphrPath<N>`]: cyphr_core::CyphrPath
"#]

pub mod prelude;
pub mod query;
pub mod stream;

pub use cyphr_core as core;
pub use cyphr_macros::{CyphrNode, CyphrRelation, FromCyphr, ToCyphrParams, cypher, cypher_query};

pub use cyphr_core::traits::{CyphrNode as CyphrNodeTrait, CyphrRelation as CyphrRelationTrait, FromCyphr as FromCyphrTrait};
pub use cyphr_core::CyphrError;
