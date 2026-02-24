
//! Value conversion from `neo4rs::BoltType` into Rust types.
//!
//! This module contains all [`FromCyphrValue`] implementations and the
//! spatial, binary, and path wrapper types.

use std::collections::HashMap;
use crate::error::CyphrError;
use crate::traits::{CyphrNode, CyphrRelation, FromCyphrValue, IntoCyphrValue, NodeWrapper, RelationWrapper};

/// Returns a human-readable name for a [`neo4rs::BoltType`] variant.
///
/// Used in error messages to describe the actual type received when a
/// conversion fails.
pub fn type_name(v: &neo4rs::BoltType) -> &'static str {
    match v {
        neo4rs::BoltType::Null(_) => "Null",
        neo4rs::BoltType::Boolean(_) => "Boolean",
        neo4rs::BoltType::Integer(_) => "Integer",
        neo4rs::BoltType::Float(_) => "Float",
        neo4rs::BoltType::String(_) => "String",
        neo4rs::BoltType::Bytes(_) => "Bytes",
        neo4rs::BoltType::List(_) => "List",
        neo4rs::BoltType::Map(_) => "Map",
        neo4rs::BoltType::Node(_) => "Node",
        neo4rs::BoltType::Relation(_) => "Relationship",
        neo4rs::BoltType::UnboundedRelation(_) => "UnboundedRelationship",
        neo4rs::BoltType::Path(_) => "Path",
        neo4rs::BoltType::Point2D(_) => "Point2D",
        neo4rs::BoltType::Point3D(_) => "Point3D",
        neo4rs::BoltType::Duration(_) => "Duration",
        neo4rs::BoltType::Date(_) => "Date",
        neo4rs::BoltType::Time(_) => "Time",
        neo4rs::BoltType::LocalTime(_) => "LocalTime",
        neo4rs::BoltType::LocalDateTime(_) => "LocalDateTime",
        neo4rs::BoltType::DateTime(_) => "DateTime",
        neo4rs::BoltType::DateTimeZoneId(_) => "DateTimeZoneId",
    }
}

// ---------------------------------------------------------------------------
// Numeric macro
// ---------------------------------------------------------------------------

macro_rules! impl_from_val_num {
    ($t:ty, $pat:ident) => {
        impl FromCyphrValue for $t {
            fn from_value(value: neo4rs::BoltType) -> Result<Self, CyphrError> {
                match value {
                    neo4rs::BoltType::$pat(v) => Ok(v.value as $t),
                    other => Err(CyphrError::type_mismatch(
                        stringify!($pat),
                        type_name(&other),
                        stringify!($t),
                    )),
                }
            }
        }
    };
}

// ---------------------------------------------------------------------------
// Primitives
// ---------------------------------------------------------------------------

impl FromCyphrValue for String {
    fn from_value(value: neo4rs::BoltType) -> Result<Self, CyphrError> {
        match value {
            neo4rs::BoltType::String(s) => Ok(s.to_string()),
            other => Err(CyphrError::type_mismatch("String", type_name(&other), "String")),
        }
    }
}

impl FromCyphrValue for bool {
    fn from_value(value: neo4rs::BoltType) -> Result<Self, CyphrError> {
        match value {
            neo4rs::BoltType::Boolean(b) => Ok(b.value),
            other => Err(CyphrError::type_mismatch("Boolean", type_name(&other), "bool")),
        }
    }
}

// ---------------------------------------------------------------------------
// Numeric types (Integer → signed/unsigned, Float → f64/f32)
// ---------------------------------------------------------------------------

impl_from_val_num!(i64, Integer);
impl_from_val_num!(i32, Integer);
impl_from_val_num!(u64, Integer);
impl_from_val_num!(u32, Integer);
impl_from_val_num!(i16, Integer);
impl_from_val_num!(u16, Integer);
impl_from_val_num!(i8, Integer);
impl_from_val_num!(u8, Integer);
impl_from_val_num!(f64, Float);
impl_from_val_num!(f32, Float);

// ---------------------------------------------------------------------------
// Collections
// ---------------------------------------------------------------------------

impl<T: FromCyphrValue> FromCyphrValue for Vec<T> {
    fn from_value(value: neo4rs::BoltType) -> Result<Self, CyphrError> {
        match value {
            neo4rs::BoltType::List(xs) => xs.value.into_iter().map(T::from_value).collect(),
            other => Err(CyphrError::type_mismatch("List", type_name(&other), "Vec<T>")),
        }
    }
}

/// `Option<T>` is the "loaded vs not loaded" primitive.
/// - Missing record key is handled in `FromCyphr` derive (returns `None`).
/// - Present but `null` maps to `None`.
/// - Otherwise maps to `Some(T)`.
impl<T: FromCyphrValue> FromCyphrValue for Option<T> {
    fn from_value(value: neo4rs::BoltType) -> Result<Self, CyphrError> {
        match value {
            neo4rs::BoltType::Null(_) => Ok(None),
            other => Ok(Some(T::from_value(other)?)),
        }
    }
}

// ---------------------------------------------------------------------------
// Node / Relation wrappers
// ---------------------------------------------------------------------------

impl<T: CyphrNode> FromCyphrValue for NodeWrapper<T> {
    fn from_value(value: neo4rs::BoltType) -> Result<Self, CyphrError> {
        match value {
            neo4rs::BoltType::Node(n) => {
                Ok(NodeWrapper(T::from_node(&n)?))
            }
            other => Err(CyphrError::type_mismatch("Node", type_name(&other), T::LABEL)),
        }
    }
}

impl<T: CyphrRelation> FromCyphrValue for RelationWrapper<T> {
    fn from_value(value: neo4rs::BoltType) -> Result<Self, CyphrError> {
        match value {
            neo4rs::BoltType::Relation(r) => {
                Ok(RelationWrapper(T::from_rel(&r)?))
            }
            other => Err(CyphrError::type_mismatch("Relationship", type_name(&other), T::TYPE)),
        }
    }
}

// ---------------------------------------------------------------------------
// Tuples — for common graph shapes (e.g. list unpacking)
// ---------------------------------------------------------------------------

impl<A: FromCyphrValue, B: FromCyphrValue> FromCyphrValue for (A, B) {
    fn from_value(value: neo4rs::BoltType) -> Result<Self, CyphrError> {
        match value {
            neo4rs::BoltType::List(mut xs) if xs.value.len() == 2 => {
                let b = xs.value.pop().unwrap();
                let a = xs.value.pop().unwrap();
                Ok((A::from_value(a)?, B::from_value(b)?))
            }
            other => Err(CyphrError::type_mismatch("List[2]", type_name(&other), "tuple(A, B)")),
        }
    }
}

impl<A: FromCyphrValue, B: FromCyphrValue, C: FromCyphrValue> FromCyphrValue for (A, B, C) {
    fn from_value(value: neo4rs::BoltType) -> Result<Self, CyphrError> {
        match value {
            neo4rs::BoltType::List(mut xs) if xs.value.len() == 3 => {
                let c = xs.value.pop().unwrap();
                let b = xs.value.pop().unwrap();
                let a = xs.value.pop().unwrap();
                Ok((A::from_value(a)?, B::from_value(b)?, C::from_value(c)?))
            }
            other => Err(CyphrError::type_mismatch("List[3]", type_name(&other), "tuple(A, B, C)")),
        }
    }
}

// ---------------------------------------------------------------------------
// Temporal types (chrono)
// ---------------------------------------------------------------------------

impl FromCyphrValue for chrono::NaiveDate {
    fn from_value(value: neo4rs::BoltType) -> Result<Self, CyphrError> {
        match value {
            neo4rs::BoltType::Date(d) => {
                let date: chrono::NaiveDate = d.try_into().map_err(|e: neo4rs::Error| {
                    CyphrError::Mapping(format!("BoltDate -> NaiveDate: {e}"))
                })?;
                Ok(date)
            }
            other => Err(CyphrError::type_mismatch("Date", type_name(&other), "NaiveDate")),
        }
    }
}

impl FromCyphrValue for chrono::NaiveTime {
    fn from_value(value: neo4rs::BoltType) -> Result<Self, CyphrError> {
        match value {
            neo4rs::BoltType::LocalTime(t) => Ok(t.into()),
            other => Err(CyphrError::type_mismatch("LocalTime", type_name(&other), "NaiveTime")),
        }
    }
}

/// Converts a Neo4j `Time` (time-with-offset) into a `(NaiveTime, FixedOffset)` pair.
impl FromCyphrValue for (chrono::NaiveTime, chrono::FixedOffset) {
    fn from_value(value: neo4rs::BoltType) -> Result<Self, CyphrError> {
        match value {
            neo4rs::BoltType::Time(t) => Ok(t.into()),
            other => Err(CyphrError::type_mismatch("Time", type_name(&other), "(NaiveTime, FixedOffset)")),
        }
    }
}

impl FromCyphrValue for chrono::NaiveDateTime {
    fn from_value(value: neo4rs::BoltType) -> Result<Self, CyphrError> {
        match value {
            neo4rs::BoltType::LocalDateTime(dt) => {
                let ndt: chrono::NaiveDateTime = dt.try_into().map_err(|e: neo4rs::Error| {
                    CyphrError::Mapping(format!("BoltLocalDateTime -> NaiveDateTime: {e}"))
                })?;
                Ok(ndt)
            }
            other => Err(CyphrError::type_mismatch("LocalDateTime", type_name(&other), "NaiveDateTime")),
        }
    }
}

/// Accepts both `DateTime` (fixed offset) and `DateTimeZoneId` (zone name) bolt types.
impl FromCyphrValue for chrono::DateTime<chrono::FixedOffset> {
    fn from_value(value: neo4rs::BoltType) -> Result<Self, CyphrError> {
        match value {
            neo4rs::BoltType::DateTime(dt) => {
                let cdt: chrono::DateTime<chrono::FixedOffset> = dt.try_into().map_err(|e: neo4rs::Error| {
                    CyphrError::Mapping(format!("BoltDateTime -> DateTime<FixedOffset>: {e}"))
                })?;
                Ok(cdt)
            }
            neo4rs::BoltType::DateTimeZoneId(dt) => {
                let cdt: chrono::DateTime<chrono::FixedOffset> = (&dt).try_into().map_err(|e: neo4rs::Error| {
                    CyphrError::Mapping(format!("BoltDateTimeZoneId -> DateTime<FixedOffset>: {e}"))
                })?;
                Ok(cdt)
            }
            other => Err(CyphrError::type_mismatch("DateTime", type_name(&other), "DateTime<FixedOffset>")),
        }
    }
}

impl FromCyphrValue for std::time::Duration {
    fn from_value(value: neo4rs::BoltType) -> Result<Self, CyphrError> {
        match value {
            neo4rs::BoltType::Duration(d) => Ok(d.into()),
            other => Err(CyphrError::type_mismatch("Duration", type_name(&other), "std::time::Duration")),
        }
    }
}

// ---------------------------------------------------------------------------
// Spatial types
// ---------------------------------------------------------------------------

/// A 2-dimensional point from Neo4j's spatial system.
///
/// Converts from `BoltType::Point2D`. The `sr_id` is the
/// [Spatial Reference Identifier](https://en.wikipedia.org/wiki/Spatial_reference_system#Identifier)
/// (e.g. `4326` for WGS 84 geographic, `7203` for cartesian).
///
/// # Example
///
/// ```rust,ignore
/// use cyphr::prelude::*;
///
/// #[derive(FromCyphr)]
/// struct Location {
///     pos: Point2D,
/// }
/// // Cypher: RETURN point({x: 1.0, y: 2.0, crs: 'cartesian'}) AS pos
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct Point2D {
    /// Spatial Reference System Identifier (e.g. 4326 for WGS 84, 7203 for cartesian).
    pub sr_id: i64,
    /// X coordinate (or longitude for geographic CRS).
    pub x: f64,
    /// Y coordinate (or latitude for geographic CRS).
    pub y: f64,
}

/// A 3-dimensional point from Neo4j's spatial system.
///
/// Same as [`Point2D`] with an additional `z` (altitude/height) component.
/// Converts from `BoltType::Point3D`.
///
/// # Example
///
/// ```rust,ignore
/// use cyphr::prelude::*;
///
/// #[derive(FromCyphr)]
/// struct Location3D {
///     pos: Point3D,
/// }
/// // Cypher: RETURN point({x: 1.0, y: 2.0, z: 3.0, crs: 'cartesian-3d'}) AS pos
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct Point3D {
    /// Spatial Reference System Identifier (e.g. 4979 for WGS 84-3D, 9157 for cartesian-3D).
    pub sr_id: i64,
    /// X coordinate (or longitude for geographic CRS).
    pub x: f64,
    /// Y coordinate (or latitude for geographic CRS).
    pub y: f64,
    /// Z coordinate (altitude/height).
    pub z: f64,
}

impl FromCyphrValue for Point2D {
    fn from_value(value: neo4rs::BoltType) -> Result<Self, CyphrError> {
        match value {
            neo4rs::BoltType::Point2D(p) => Ok(Point2D {
                sr_id: p.sr_id.value,
                x: p.x.value,
                y: p.y.value,
            }),
            other => Err(CyphrError::type_mismatch("Point2D", type_name(&other), "Point2D")),
        }
    }
}

impl FromCyphrValue for Point3D {
    fn from_value(value: neo4rs::BoltType) -> Result<Self, CyphrError> {
        match value {
            neo4rs::BoltType::Point3D(p) => Ok(Point3D {
                sr_id: p.sr_id.value,
                x: p.x.value,
                y: p.y.value,
                z: p.z.value,
            }),
            other => Err(CyphrError::type_mismatch("Point3D", type_name(&other), "Point3D")),
        }
    }
}

// ---------------------------------------------------------------------------
// Bytes
// ---------------------------------------------------------------------------

/// Newtype wrapper for raw bytes from Neo4j's `Bytes` type.
///
/// A dedicated type is used instead of `Vec<u8>` because a blanket
/// `FromCyphrValue` impl already exists for `Vec<T: FromCyphrValue>`, and
/// Rust does not support specialization.
///
/// # Example
///
/// ```rust,ignore
/// use cyphr::prelude::*;
///
/// #[derive(FromCyphr)]
/// struct BlobRow {
///     data: CyphrBytes,
/// }
/// // Access the inner bytes:
/// // let raw: &[u8] = &blob_row.data.0;
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CyphrBytes(pub Vec<u8>);

impl FromCyphrValue for CyphrBytes {
    fn from_value(value: neo4rs::BoltType) -> Result<Self, CyphrError> {
        match value {
            neo4rs::BoltType::Bytes(b) => Ok(CyphrBytes(b.value.to_vec())),
            other => Err(CyphrError::type_mismatch("Bytes", type_name(&other), "CyphrBytes")),
        }
    }
}

// ---------------------------------------------------------------------------
// Path
// ---------------------------------------------------------------------------

/// A typed graph path returned by Neo4j `Path` values.
///
/// `N` is the node type (must implement [`CyphrNode`]). Each node in the
/// path is deserialized via `N::from_node()`. Relationships are kept as raw
/// [`neo4rs::BoltUnboundedRelation`] values since paths return unbounded
/// (direction-less) relationships.
///
/// # Fields
///
/// - `nodes` — the ordered list of nodes along the path.
/// - `rels` — the unbounded relationships connecting consecutive nodes.
/// - `indices` — the raw path indices from the bolt protocol.
///
/// # Example
///
/// ```rust,ignore
/// use cyphr::prelude::*;
///
/// #[derive(Debug, CyphrNode)]
/// #[cyphr(label = "Person")]
/// struct Person {
///     name: String,
/// }
///
/// #[derive(FromCyphr)]
/// struct PathRow {
///     p: CyphrPath<Person>,
/// }
/// // Cypher: MATCH p = (a:Person)-[:KNOWS*..3]->(b:Person) RETURN p
/// ```
pub struct CyphrPath<N: CyphrNode> {
    /// Ordered nodes along the path.
    pub nodes: Vec<N>,
    /// Unbounded relationships connecting consecutive nodes.
    pub rels: Vec<neo4rs::BoltUnboundedRelation>,
    /// Raw path indices from the bolt protocol.
    pub indices: Vec<i64>,
}

impl<N: CyphrNode> FromCyphrValue for CyphrPath<N> {
    fn from_value(value: neo4rs::BoltType) -> Result<Self, CyphrError> {
        match value {
            neo4rs::BoltType::Path(p) => {
                let bolt_nodes = p.nodes();
                let mut nodes = Vec::with_capacity(bolt_nodes.len());
                for n in &bolt_nodes {
                    nodes.push(N::from_node(n)?);
                }
                let rels = p.rels();
                let indices = p.indices().into_iter().map(|i| i.value).collect();
                Ok(CyphrPath { nodes, rels, indices })
            }
            other => Err(CyphrError::type_mismatch("Path", type_name(&other), "CyphrPath")),
        }
    }
}

// ---------------------------------------------------------------------------
// HashMap
// ---------------------------------------------------------------------------

/// Converts a Neo4j `Map` into `HashMap<String, V>`.
///
/// Each map entry's key becomes a `String` and the value is converted via
/// `V::from_value()`.
impl<V: FromCyphrValue> FromCyphrValue for HashMap<String, V> {
    fn from_value(value: neo4rs::BoltType) -> Result<Self, CyphrError> {
        match value {
            neo4rs::BoltType::Map(m) => {
                let mut out = HashMap::with_capacity(m.value.len());
                for (k, v) in m.value {
                    out.insert(k.to_string(), V::from_value(v)?);
                }
                Ok(out)
            }
            other => Err(CyphrError::type_mismatch("Map", type_name(&other), "HashMap<String, V>")),
        }
    }
}

// ---------------------------------------------------------------------------
// IntoCyphrValue — manual impls for types without Into<BoltType>
// ---------------------------------------------------------------------------

impl IntoCyphrValue for Point2D {
    fn into_value(self) -> neo4rs::BoltType {
        neo4rs::BoltType::Point2D(neo4rs::BoltPoint2D {
            sr_id: neo4rs::BoltInteger::new(self.sr_id),
            x: neo4rs::BoltFloat::new(self.x),
            y: neo4rs::BoltFloat::new(self.y),
        })
    }
}

impl IntoCyphrValue for Point3D {
    fn into_value(self) -> neo4rs::BoltType {
        neo4rs::BoltType::Point3D(neo4rs::BoltPoint3D {
            sr_id: neo4rs::BoltInteger::new(self.sr_id),
            x: neo4rs::BoltFloat::new(self.x),
            y: neo4rs::BoltFloat::new(self.y),
            z: neo4rs::BoltFloat::new(self.z),
        })
    }
}

impl IntoCyphrValue for CyphrBytes {
    fn into_value(self) -> neo4rs::BoltType {
        neo4rs::BoltType::Bytes(neo4rs::BoltBytes::new(bytes::Bytes::from(self.0)))
    }
}

