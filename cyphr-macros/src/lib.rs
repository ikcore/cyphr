
//! Procedural macros for cyphr.
//!
//! This crate is not meant to be used directly — use the [`cyphr`] facade
//! crate which re-exports all macros.

extern crate proc_macro;

use proc_macro::TokenStream;

mod node;
mod relation;
mod from_cyphr;
mod cypher;
mod cypher_query;
mod to_cyphr_params;

/// Derive [`CyphrNode`](cyphr_core::traits::CyphrNode) and [`FromCyphrValue`](cyphr_core::traits::FromCyphrValue) for a struct.
///
/// Maps a Neo4j node to a Rust struct. Each struct field is read from a
/// node property via `FromCyphrValue`.
///
/// Also auto-implements `FromCyphrValue` so the struct can be converted
/// directly from `BoltType::Node` without needing `NodeWrapper`.
///
/// # Attributes
///
/// **Struct-level:**
/// - `#[cyphr(label = "...")]` — set the Neo4j label. Defaults to the struct name.
///
/// **Field-level:**
/// - `#[cyphr(prop = "...")]` — override the Neo4j property name (default: field name).
/// - `#[cyphr(id)]` — marker for the identity field (documentation only, no runtime effect).
///
/// # Example
///
/// ```rust,ignore
/// use cyphr::prelude::*;
///
/// #[derive(Debug, CyphrNode)]
/// #[cyphr(label = "User")]
/// struct User {
///     #[cyphr(id)]
///     id: i64,
///     name: String,
///     #[cyphr(prop = "email_address")]
///     email: String,
/// }
/// ```
#[proc_macro_derive(CyphrNode, attributes(cyphr))]
pub fn cyphr_node(input: TokenStream) -> TokenStream {
    node::expand(input)
}

/// Derive [`CyphrRelation`](cyphr_core::traits::CyphrRelation) and [`FromCyphrValue`](cyphr_core::traits::FromCyphrValue) for a struct.
///
/// Maps a Neo4j relationship to a Rust struct. Each struct field is read
/// from a relationship property via `FromCyphrValue`.
///
/// Also auto-implements `FromCyphrValue` so the struct can be converted
/// directly from `BoltType::Relation` without needing `RelationWrapper`.
///
/// # Attributes
///
/// **Struct-level:**
/// - `#[cyphr(type = "...")]` — set the relationship type. Defaults to the struct name.
/// - `#[cyphr(from = "...")]` — optional start-node label (compile-time documentation).
/// - `#[cyphr(to = "...")]` — optional end-node label (compile-time documentation).
///
/// **Field-level:**
/// - `#[cyphr(prop = "...")]` — override the Neo4j property name (default: field name).
///
/// # Example
///
/// ```rust,ignore
/// use cyphr::prelude::*;
///
/// #[derive(Debug, CyphrRelation)]
/// #[cyphr(type = "FOLLOWS", from = "User", to = "User")]
/// struct Follows {
///     since: i64,
///     #[cyphr(prop = "weight")]
///     strength: f64,
/// }
/// ```
#[proc_macro_derive(CyphrRelation, attributes(cyphr))]
pub fn cyphr_relation(input: TokenStream) -> TokenStream {
    relation::expand(input)
}

/// Derive [`FromCyphr`](cyphr_core::traits::FromCyphr) for a struct.
///
/// Maps a `neo4rs::Row` to a Rust struct. Each field is read from the row
/// by its name (the Cypher alias).
///
/// # Attributes
///
/// **Field-level:**
/// - `#[cyphr(flatten)]` — instead of reading a single column, delegates to
///   the inner type's `FromCyphr::from_record()` with the same row. Useful
///   for composing result structs that share columns.
///
/// # Field type behaviour
///
/// - **`Option<T>`** — missing row key or `null` becomes `None`.
/// - **`NodeWrapper<T>`** — wraps a `CyphrNode` type embedded in the row.
/// - **`T: FromCyphrValue`** — any type with a value conversion (including
///   auto-derived `CyphrNode`/`CyphrRelation` types, spatial types, etc.).
///
/// # Example
///
/// ```rust,ignore
/// use cyphr::prelude::*;
///
/// # #[derive(Debug, CyphrNode)]
/// # #[cyphr(label = "User")]
/// # struct User { id: i64, name: String }
/// #[derive(FromCyphr)]
/// struct Metadata {
///     created: String,
/// }
///
/// #[derive(FromCyphr)]
/// struct UserRow {
///     u: NodeWrapper<User>,
///     score: Option<f64>,
///     #[cyphr(flatten)]
///     meta: Metadata,
/// }
/// ```
#[proc_macro_derive(FromCyphr, attributes(cyphr))]
pub fn from_cyphr(input: TokenStream) -> TokenStream {
    from_cyphr::expand(input)
}

/// Converts a Cypher token block into a `&'static str` with normalized whitespace.
///
/// The output is intentionally literal — no rewriting or validation. You can
/// paste the resulting string directly into Neo4j Browser.
///
/// # Example
///
/// ```rust,ignore
/// use cyphr::prelude::*;
///
/// let q: &str = cypher! {
///     MATCH (u:User)-[:FOLLOWS]->(f:User)
///     WHERE u.name = "Alice"
///     RETURN f.name, f.id
/// };
/// ```
#[proc_macro]
pub fn cypher(input: TokenStream) -> TokenStream {
    cypher::expand(input)
}

/// Builds a [`CyphrQuery`](cyphr::query::CyphrQuery) from a Cypher block with auto-bound parameters.
///
/// Any `$ident` in the Cypher text becomes a named parameter. The Rust
/// variable `ident` must be in scope and implement `Into<BoltType>`.
/// Duplicate parameters are bound only once.
///
/// # Example
///
/// ```rust,ignore
/// use cyphr::prelude::*;
///
/// let name = "Alice";
/// let age: i64 = 30;
/// let query = cypher_query! {
///     MATCH (u:User {name: $name})
///     WHERE u.age > $age
///     RETURN u
/// };
/// // Expands to:
/// //   CyphrQuery::new("MATCH (u:User {name:$name}) WHERE u.age > $age RETURN u")
/// //       .param("name", name)
/// //       .param("age", age)
/// ```
#[proc_macro]
pub fn cypher_query(input: TokenStream) -> TokenStream {
    cypher_query::expand(input)
}

/// Derive `ToCyphrParams` for a struct.
///
/// Converts each field into a named query parameter via `IntoCyphrValue`.
/// Use with `CyphrQuery::params_from` for ergonomic bulk parameter binding.
///
/// # Attributes
///
/// **Field-level:**
/// - `#[cyphr(skip)]` or `#[cyphr(id)]` — exclude the field from the parameter map.
/// - `#[cyphr(prop = "...")]` — override the parameter name (default: field name).
///
/// # Example
///
/// ```rust,ignore
/// use cyphr::prelude::*;
///
/// #[derive(ToCyphrParams)]
/// struct CreateUser {
///     #[cyphr(skip)]
///     internal_id: u64,
///     name: String,
///     #[cyphr(prop = "user_age")]
///     age: i64,
/// }
/// ```
#[proc_macro_derive(ToCyphrParams, attributes(cyphr))]
pub fn to_cyphr_params(input: TokenStream) -> TokenStream {
    to_cyphr_params::expand(input)
}
