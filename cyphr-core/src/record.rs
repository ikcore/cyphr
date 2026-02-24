
//! Centralized row/record access for Neo4j query results.
//!
//! These functions isolate the `neo4rs::Row` API to a single location so
//! that derive-macro generated code only depends on this module.

use neo4rs::{BoltType as Value, Row as Record};

/// Read a value from a [`Row`](neo4rs::Row) by column name.
///
/// Returns `None` if the column does not exist in the row.
pub fn get_value(record: &Record, key: &str) -> Option<Value> {
    record.get(key).ok()
}

/// Check whether a column exists in the row.
pub fn has_key(record: &Record, key: &str) -> bool {
    get_value(record, key).is_some()
}
