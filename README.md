# cyphr

A lightweight, Cypher-first mapping layer for Neo4j in Rust. Not an ORM — it preserves raw Cypher while adding strong Rust types around nodes, relationships, and query result projections. Built on [neo4rs](https://docs.rs/neo4rs) 0.8.

## Installation

```toml
[dependencies]
cyphr = { path = "." }  # or from crates.io when published
neo4rs = "0.8"
```

## Quick start

### Define nodes and relationships

```rust
use cyphr::prelude::*;

#[derive(Debug, CyphrNode)]
#[cyphr(label = "User")]
struct User {
    #[cyphr(id)]
    id: i64,
    name: String,
    #[cyphr(prop = "email_address")]
    email: String,
}

#[derive(Debug, CyphrRelation)]
#[cyphr(type = "FOLLOWS", from = "User", to = "User")]
struct Follows {
    since: i64,
}
```

Both derives automatically implement `FromCyphrValue`, so node/relation types can be used directly as fields in query result structs.

### Map query results

```rust
use cyphr::prelude::*;

#[derive(FromCyphr)]
struct UserRow {
    u: User,          // auto-converted from BoltType::Node
    score: f64,
    tags: Option<Vec<String>>,
    #[cyphr(flatten)]  // shares the same Row
    meta: Metadata,
}
```

## Write Cypher

```rust
use cyphr::prelude::*;

// String literal -- copy/paste into Neo4j Browser:
let q: &str = cypher! { MATCH (u:User) RETURN u };

// Query builder with auto-bound parameters:
let name = "Alice";
let age: i64 = 30;
let query = cypher_query! {
    MATCH (u:User {name: $name})
    WHERE u.age > $age
    RETURN u
};
// Expands to: CyphrQuery::new("...").param("name", name).param("age", age)
```

## Execute queries

```rust,no_run
use cyphr::prelude::*;
use cyphr::query;

// Exactly one row (error if empty):
let user: UserRow = query::query("MATCH (u:User) RETURN u LIMIT 1")
    .fetch_one(&graph).await?;

// Zero or one row:
let maybe: Option<UserRow> = query::query("MATCH (u:User {id: 1}) RETURN u")
    .fetch_optional(&graph).await?;

// All rows at once:
let users: Vec<UserRow> = query::query("MATCH (u:User) RETURN u")
    .fetch_all(&graph).await?;

// Streaming -- rows converted one at a time:
let mut stream = query::query("MATCH (u:User) RETURN u")
    .fetch_stream::<UserRow>(&graph).await?;
while let Some(result) = stream.next().await {
    let user = result?;
}
```

## Writing data with `ToCyphrParams`

Convert a struct into named query parameters with `#[derive(ToCyphrParams)]`:

```rust,no_run
use cyphr::prelude::*;
use cyphr::query::CyphrQuery;

#[derive(ToCyphrParams)]
struct CreateUser {
    name: String,
    age: i64,
}

let params = CreateUser { name: "Alice".into(), age: 30 };
CyphrQuery::new("CREATE (u:User {name: $name, age: $age})")
    .params_from(params);
```

Fields marked `#[cyphr(skip)]` or `#[cyphr(id)]` are excluded. Use `#[cyphr(prop = "...")]` to override the parameter key name.

## Transaction support

All `fetch_*` methods have `_in` variants that execute within a transaction:

```rust,no_run
let mut txn = graph.start_txn().await?;
let user: UserRow = query::query("MATCH (u:User) RETURN u LIMIT 1")
    .fetch_one_in(&mut txn).await?;
txn.commit().await?;
```

## Supported value types

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
| Point2D | `Point2D` |
| Point3D | `Point3D` |
| Bytes | `CyphrBytes` |
| Path | `CyphrPath<N>` |
| Date | `chrono::NaiveDate` |
| LocalTime | `chrono::NaiveTime` |
| Time | `(chrono::NaiveTime, chrono::FixedOffset)` |
| LocalDateTime | `chrono::NaiveDateTime` |
| DateTime / DateTimeZoneId | `chrono::DateTime<chrono::FixedOffset>` |
| Duration | `std::time::Duration` |

## Error handling

All conversions return `CyphrError`. Derive macros automatically wrap errors with context so you can trace exactly where a conversion failed:

```
User::name (prop 'name'): type mismatch: expected String, got Integer (String)
```

## License

AGPLv3
