//! Convenience re-exports for common cyphr usage.
//!
//! ```rust
//! use cyphr::prelude::*;
//! ```
//!
//! This imports all derive macros (`CyphrNode`, `CyphrRelation`, `FromCyphr`,
//! `ToCyphrParams`), both function-like macros (`cypher!`, `cypher_query!`),
//! the core traits (`CyphrNodeTrait`, `CyphrRelationTrait`, `FromCyphrTrait`,
//! `FromCyphrValue`, `IntoCyphrValue`, `ToCyphrParamsTrait`), the error type,
//! spatial/binary/path wrapper types, and [`CyphrStream`].

pub use crate::{cypher, cypher_query, CyphrNode, CyphrRelation, FromCyphr, ToCyphrParams};
pub use cyphr_core::traits::{
    CyphrNode as CyphrNodeTrait, CyphrRelation as CyphrRelationTrait,
    FromCyphr as FromCyphrTrait, FromCyphrValue,
    IntoCyphrValue, ToCyphrParams as ToCyphrParamsTrait,
};
pub use cyphr_core::CyphrError;
pub use cyphr_core::{Point2D, Point3D, CyphrBytes, CyphrPath};
pub use crate::stream::CyphrStream;
