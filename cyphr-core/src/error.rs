
//! Error types for cyphr conversions.

use thiserror::Error;

/// Unified error type for all cyphr operations.
///
/// Derive macros automatically wrap conversion errors with [`Context`](CyphrError::Context)
/// via [`with_context`](CyphrError::with_context), producing chained messages like:
///
/// ```text
/// User::age (prop 'age'): type mismatch: expected Integer, got String (i64)
/// ```
#[derive(Error, Debug)]
pub enum CyphrError {
    /// A general mapping error with a freeform message.
    #[error("mapping error: {0}")]
    Mapping(String),

    /// A required property was not found on a node or relationship.
    #[error("missing property '{property}' on {label}")]
    MissingProperty { property: String, label: String },

    /// A required field was not found in a `neo4rs::Row`.
    #[error("missing field '{field}' on {struct_name}")]
    MissingField { field: String, struct_name: String },

    /// A `BoltType` variant did not match the expected Rust type.
    #[error("type mismatch: expected {expected}, got {got} ({context})")]
    TypeMismatch {
        expected: String,
        got: String,
        context: String,
    },

    /// Wraps an inner error with additional context (struct name, field, property).
    ///
    /// Created automatically by derive macros. Can also be created manually
    /// via [`with_context`](CyphrError::with_context).
    #[error("{context}: {source}")]
    Context {
        context: String,
        source: Box<CyphrError>,
    },

    /// A `neo4rs::Error` from the underlying driver.
    #[error("neo4j error: {0}")]
    Neo4j(#[from] neo4rs::Error),
}

impl CyphrError {
    /// Create a [`TypeMismatch`](CyphrError::TypeMismatch) error.
    pub fn type_mismatch(expected: &str, got: &str, context: &str) -> Self {
        CyphrError::TypeMismatch {
            expected: expected.to_owned(),
            got: got.to_owned(),
            context: context.to_owned(),
        }
    }

    /// Create a [`MissingProperty`](CyphrError::MissingProperty) error.
    pub fn missing_property(property: &str, label: &str) -> Self {
        CyphrError::MissingProperty {
            property: property.to_owned(),
            label: label.to_owned(),
        }
    }

    /// Create a [`MissingField`](CyphrError::MissingField) error.
    pub fn missing_field(field: &str, struct_name: &str) -> Self {
        CyphrError::MissingField {
            field: field.to_owned(),
            struct_name: struct_name.to_owned(),
        }
    }

    /// Wrap this error with additional context, producing a [`Context`](CyphrError::Context) variant.
    ///
    /// The derive macros call this automatically to annotate errors with the
    /// struct name, field name, and property key so you can trace exactly
    /// where a conversion failed.
    ///
    /// ```rust
    /// # use cyphr_core::CyphrError;
    /// let err = CyphrError::type_mismatch("Integer", "String", "i64");
    /// let wrapped = err.with_context("User::age (prop 'age')");
    /// assert!(wrapped.to_string().contains("User::age"));
    /// ```
    pub fn with_context(self, ctx: impl Into<String>) -> Self {
        CyphrError::Context {
            context: ctx.into(),
            source: Box::new(self),
        }
    }
}
