
//! Core traits for mapping between Neo4j bolt types and Rust structs.

use neo4rs::{BoltType as Value, Row as Record};
use crate::error::CyphrError;

/// Maps a Neo4j node to a Rust struct.
///
/// Automatically implemented by `#[derive(CyphrNode)]`. The derive also
/// generates a [`FromCyphrValue`] impl so the struct can be converted
/// directly from `BoltType::Node`.
///
/// # Example
///
/// ```rust,ignore
/// #[derive(CyphrNode)]
/// #[cyphr(label = "User")]
/// struct User {
///     id: i64,
///     name: String,
/// }
///
/// assert_eq!(User::LABEL, "User");
/// ```
pub trait CyphrNode: Sized {
    /// The Neo4j label (e.g. `"User"`).
    const LABEL: &'static str;

    /// Deserialize a [`BoltNode`](neo4rs::BoltNode) into `Self`.
    fn from_node(node: &neo4rs::BoltNode) -> Result<Self, CyphrError>;
}

/// Maps a Neo4j relationship to a Rust struct.
///
/// Automatically implemented by `#[derive(CyphrRelation)]`. The derive
/// also generates a [`FromCyphrValue`] impl so the struct can be converted
/// directly from `BoltType::Relation`.
///
/// # Example
///
/// ```rust,ignore
/// #[derive(CyphrRelation)]
/// #[cyphr(type = "FOLLOWS", from = "User", to = "User")]
/// struct Follows {
///     since: i64,
/// }
///
/// assert_eq!(Follows::TYPE, "FOLLOWS");
/// ```
pub trait CyphrRelation: Sized {
    /// The Neo4j relationship type (e.g. `"FOLLOWS"`).
    const TYPE: &'static str;

    /// Optional label of the start node, for compile-time documentation.
    const FROM_LABEL: Option<&'static str> = None;
    /// Optional label of the end node, for compile-time documentation.
    const TO_LABEL: Option<&'static str> = None;

    /// Deserialize a [`BoltRelation`](neo4rs::BoltRelation) into `Self`.
    fn from_rel(rel: &neo4rs::BoltRelation) -> Result<Self, CyphrError>;
}

/// Maps a `neo4rs::Row` into a Rust struct by field name.
///
/// Automatically implemented by `#[derive(FromCyphr)]`. Each struct field
/// maps to a column name in the row.
///
/// # Field attributes
///
/// - **`Option<T>`** fields tolerate missing keys (become `None`).
/// - **`#[cyphr(flatten)]`** delegates to the inner type's `FromCyphr` impl,
///   passing the same row. Useful for composing result structs.
///
/// # Example
///
/// ```rust,ignore
/// #[derive(FromCyphr)]
/// struct UserRow {
///     name: String,
///     age: Option<i64>,
///     #[cyphr(flatten)]
///     meta: Metadata,
/// }
/// ```
pub trait FromCyphr: Sized {
    /// Deserialize a [`Row`](neo4rs::Row) into `Self`.
    fn from_record(record: &Record) -> Result<Self, CyphrError>;
}

/// Converts a single `neo4rs::BoltType` value into a Rust type.
///
/// This is the core conversion primitive. Implementations exist for
/// primitives, collections, temporal types, spatial types, and wrapper
/// types. See the [crate-level docs](crate) for a full table.
///
/// `#[derive(CyphrNode)]` and `#[derive(CyphrRelation)]` automatically
/// implement this trait, so node/relation structs can be used directly
/// as field types in `FromCyphr` structs without `NodeWrapper`/`RelationWrapper`.
pub trait FromCyphrValue: Sized {
    /// Convert a [`BoltType`](neo4rs::BoltType) into `Self`.
    fn from_value(value: Value) -> Result<Self, CyphrError>;
}

/// Converts a Rust value into a `neo4rs::BoltType` for use as a query parameter.
///
/// A blanket implementation covers all types that already implement
/// `Into<BoltType>` (e.g. `String`, `i64`, `f64`, `bool`). Custom
/// implementations are provided for cyphr-specific types like [`Point2D`],
/// [`Point3D`], and [`CyphrBytes`].
///
/// [`Point2D`]: crate::Point2D
/// [`Point3D`]: crate::Point3D
/// [`CyphrBytes`]: crate::CyphrBytes
pub trait IntoCyphrValue {
    /// Convert `self` into a [`BoltType`](neo4rs::BoltType).
    fn into_value(self) -> Value;
}

impl<T: Into<Value>> IntoCyphrValue for T {
    fn into_value(self) -> Value {
        self.into()
    }
}

/// Converts a struct into a `HashMap<&str, BoltType>` for bulk query parameters.
///
/// Automatically implemented by `#[derive(ToCyphrParams)]`. Use with
/// `CyphrQuery::params_from` to bind all struct fields as named parameters
/// in a single call.
///
/// # Example
///
/// ```rust,ignore
/// use cyphr::prelude::*;
///
/// #[derive(ToCyphrParams)]
/// struct CreateUser {
///     name: String,
///     age: i64,
/// }
///
/// let params = CreateUser { name: "Alice".into(), age: 30 };
/// let query = cypher_query! {
///     CREATE (u:User {name: $name, age: $age})
/// }.params_from(params);
/// ```
pub trait ToCyphrParams {
    /// Convert `self` into a map of parameter name → value.
    fn to_params(self) -> std::collections::HashMap<String, Value>;
}

/// Newtype wrapper for embedding a [`CyphrNode`] inside a [`FromCyphr`] struct.
///
/// Use this when you need to keep a node type as a field in a row-mapping
/// struct and the node type does not have an auto-derived `FromCyphrValue`
/// impl (e.g. a manually-implemented `CyphrNode`).
///
/// ```rust,ignore
/// #[derive(FromCyphr)]
/// struct UserRow {
///     u: NodeWrapper<User>,
/// }
/// ```
///
/// If `User` was derived with `#[derive(CyphrNode)]`, you can also use
/// `User` directly (without the wrapper) since the derive auto-implements
/// `FromCyphrValue`.
pub struct NodeWrapper<T>(pub T);

/// Newtype wrapper for embedding a [`CyphrRelation`] inside a [`FromCyphr`] struct.
///
/// Analogous to [`NodeWrapper`] but for relationships. See its docs for details.
pub struct RelationWrapper<T>(pub T);
