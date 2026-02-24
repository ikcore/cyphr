
//! Centralized property access for Neo4j nodes and relationships.
//!
//! These functions isolate the `neo4rs` property-access API to a single
//! location so that derive-macro generated code only depends on this module.

use neo4rs::BoltType as Value;

/// Read a property from a [`BoltNode`](neo4rs::BoltNode) by key.
///
/// Returns `None` if the property does not exist on the node.
pub fn node_prop(node: &neo4rs::BoltNode, key: &str) -> Option<Value> {
    node.get::<Value>(key).ok()
}

/// Read a property from a [`BoltRelation`](neo4rs::BoltRelation) by key.
///
/// Returns `None` if the property does not exist on the relationship.
pub fn rel_prop(rel: &neo4rs::BoltRelation, key: &str) -> Option<Value> {
    rel.get::<Value>(key).ok()
}
