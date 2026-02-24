use cyphr_macros::{CyphrNode, CyphrRelation, FromCyphr, ToCyphrParams, cypher};
use cyphr_core::traits::{CyphrNode, CyphrRelation, FromCyphrValue, FromCyphr, NodeWrapper, ToCyphrParams};
use cyphr_core::error::CyphrError;
use neo4rs::{BoltType, BoltList, Row};

#[derive(Debug, CyphrNode)]
#[cyphr(label = "User")]
#[allow(dead_code)]
struct User {
    id: i64,
    name: String,
}

#[derive(Debug, CyphrRelation)]
#[cyphr(type = "FRIEND")]
#[allow(dead_code)]
struct Friend {
    since: i64,
}

#[derive(FromCyphr)]
#[allow(dead_code)]
struct UserResult {
    u: NodeWrapper<User>,
}

#[test]
fn test_cypher_macro() {
    let query = cypher! {
        MATCH (u:User) RETURN u
    };
    // quote! might strip some spaces
    assert!(query.contains("MATCH") && query.contains("(u:User)") && query.contains("RETURN u"));
}

#[test]
fn test_node_trait_impl() {
    assert_eq!(User::LABEL, "User");
}

#[test]
fn test_relation_trait_impl() {
    assert_eq!(Friend::TYPE, "FRIEND");
}

// --- Auto-derived FromCyphrValue for CyphrNode ---

#[test]
fn test_user_from_value_node() {
    let node = neo4rs::BoltNode::new(
        neo4rs::BoltInteger::new(1),
        vec![BoltType::from("User")].into(),
        vec![
            (neo4rs::BoltString::from("id"), BoltType::from(42)),
            (neo4rs::BoltString::from("name"), BoltType::from("Alice")),
        ]
        .into_iter()
        .collect(),
    );
    let val = BoltType::Node(node);
    let user = User::from_value(val).unwrap();
    assert_eq!(user.id, 42);
    assert_eq!(user.name, "Alice");
}

#[test]
fn test_user_from_value_wrong_type() {
    let val = BoltType::Integer(neo4rs::BoltInteger { value: 1 });
    let err = User::from_value(val).unwrap_err();
    match &err {
        CyphrError::TypeMismatch { expected, got, .. } => {
            assert_eq!(expected, "Node");
            assert_eq!(got, "Integer");
        }
        other => panic!("expected TypeMismatch, got: {other}"),
    }
}

// --- Auto-derived FromCyphrValue for CyphrRelation ---

#[test]
fn test_friend_from_value_relation() {
    let rel = neo4rs::BoltRelation {
        id: neo4rs::BoltInteger::new(10),
        start_node_id: neo4rs::BoltInteger::new(1),
        end_node_id: neo4rs::BoltInteger::new(2),
        typ: neo4rs::BoltString::from("FRIEND"),
        properties: vec![
            (neo4rs::BoltString::from("since"), BoltType::from(2020)),
        ]
        .into_iter()
        .collect(),
    };
    let val = BoltType::Relation(rel);
    let friend = Friend::from_value(val).unwrap();
    assert_eq!(friend.since, 2020);
}

#[test]
fn test_friend_from_value_wrong_type() {
    let val = BoltType::String(neo4rs::BoltString::from("oops"));
    let err = Friend::from_value(val).unwrap_err();
    match &err {
        CyphrError::TypeMismatch { expected, got, .. } => {
            assert_eq!(expected, "Relationship");
            assert_eq!(got, "String");
        }
        other => panic!("expected TypeMismatch, got: {other}"),
    }
}

// --- Flatten ---

#[derive(FromCyphr)]
#[allow(dead_code)]
struct InnerResult {
    name: String,
}

#[derive(FromCyphr)]
#[allow(dead_code)]
struct OuterResult {
    age: i64,
    #[cyphr(flatten)]
    inner: InnerResult,
}

#[test]
fn test_flatten_from_record() {
    let fields = BoltList::from(vec![BoltType::from("name"), BoltType::from("age")]);
    let data = BoltList::from(vec![BoltType::from("Alice"), BoltType::from(30)]);
    let row = Row::new(fields, data);

    let outer = OuterResult::from_record(&row).unwrap();
    assert_eq!(outer.age, 30);
    assert_eq!(outer.inner.name, "Alice");
}

// --- ToCyphrParams ---

#[derive(ToCyphrParams)]
#[allow(dead_code)]
struct CreateUser {
    #[cyphr(id)]
    internal_id: i64,
    name: String,
    age: i64,
}

#[test]
fn test_to_cyphr_params_basic() {
    let params = CreateUser { internal_id: 999, name: "Alice".into(), age: 30 };
    let map = params.to_params();
    // id field should be skipped
    assert!(!map.contains_key("internal_id"));
    assert!(map.contains_key("name"));
    assert!(map.contains_key("age"));
    match &map["name"] {
        BoltType::String(s) => assert_eq!(s.value, "Alice"),
        other => panic!("expected String, got: {other:?}"),
    }
    match &map["age"] {
        BoltType::Integer(i) => assert_eq!(i.value, 30),
        other => panic!("expected Integer, got: {other:?}"),
    }
}

#[derive(ToCyphrParams)]
#[allow(dead_code)]
struct UpdateUser {
    #[cyphr(skip)]
    _ignored: bool,
    #[cyphr(prop = "user_name")]
    name: String,
}

#[test]
fn test_to_cyphr_params_prop_override() {
    let params = UpdateUser { _ignored: true, name: "Bob".into() };
    let map = params.to_params();
    assert!(!map.contains_key("_ignored"));
    assert!(!map.contains_key("name"));
    assert!(map.contains_key("user_name"));
    match &map["user_name"] {
        BoltType::String(s) => assert_eq!(s.value, "Bob"),
        other => panic!("expected String, got: {other:?}"),
    }
}
