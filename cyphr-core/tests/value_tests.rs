use std::collections::HashMap;
use cyphr_core::traits::{CyphrNode, FromCyphrValue, IntoCyphrValue};
use cyphr_core::value::{Point2D, Point3D, CyphrBytes, CyphrPath};
use cyphr_core::CyphrError;
use neo4rs::BoltType;

#[test]
fn test_from_value_integer() {
    let val = BoltType::Integer(neo4rs::BoltInteger { value: 42 });
    let res = i64::from_value(val).unwrap();
    assert_eq!(res, 42);
}

#[test]
fn test_from_value_string() {
    let val = BoltType::String(neo4rs::BoltString { value: "hello".to_string() });
    let res = String::from_value(val).unwrap();
    assert_eq!(res, "hello");
}

#[test]
fn test_from_value_bool() {
    let val = BoltType::Boolean(neo4rs::BoltBoolean { value: true });
    let res = bool::from_value(val).unwrap();
    assert!(res);
}

#[test]
fn test_from_value_list() {
    let val = BoltType::List(neo4rs::BoltList {
        value: vec![
            BoltType::Integer(neo4rs::BoltInteger { value: 1 }),
            BoltType::Integer(neo4rs::BoltInteger { value: 2 }),
        ],
    });
    let res = Vec::<i64>::from_value(val).unwrap();
    assert_eq!(res, vec![1, 2]);
}

#[test]
fn test_from_value_option() {
    let val = BoltType::Null(neo4rs::BoltNull);
    let res = Option::<i64>::from_value(val).unwrap();
    assert_eq!(res, None);

    let val = BoltType::Integer(neo4rs::BoltInteger { value: 42 });
    let res = Option::<i64>::from_value(val).unwrap();
    assert_eq!(res, Some(42));
}

#[test]
fn test_type_mismatch_error() {
    let val = BoltType::String(neo4rs::BoltString { value: "oops".to_string() });
    let err = i64::from_value(val).unwrap_err();
    match &err {
        CyphrError::TypeMismatch { expected, got, .. } => {
            assert_eq!(expected, "Integer");
            assert_eq!(got, "String");
        }
        other => panic!("expected TypeMismatch, got: {other}"),
    }
}

#[test]
fn test_missing_property_error() {
    let err = CyphrError::missing_property("name", "User");
    let msg = err.to_string();
    assert!(msg.contains("name"));
    assert!(msg.contains("User"));
}

#[test]
fn test_missing_field_error() {
    let err = CyphrError::missing_field("age", "UserResult");
    let msg = err.to_string();
    assert!(msg.contains("age"));
    assert!(msg.contains("UserResult"));
}

#[test]
fn test_hashmap_from_value() {
    let mut map = neo4rs::BoltMap::new();
    map.put(
        neo4rs::BoltString { value: "a".to_string() },
        BoltType::Integer(neo4rs::BoltInteger { value: 1 }),
    );
    map.put(
        neo4rs::BoltString { value: "b".to_string() },
        BoltType::Integer(neo4rs::BoltInteger { value: 2 }),
    );

    let val = BoltType::Map(map);
    let res = HashMap::<String, i64>::from_value(val).unwrap();
    assert_eq!(res.get("a"), Some(&1));
    assert_eq!(res.get("b"), Some(&2));
}

#[test]
fn test_hashmap_type_mismatch() {
    let val = BoltType::Integer(neo4rs::BoltInteger { value: 42 });
    let err = HashMap::<String, i64>::from_value(val).unwrap_err();
    match &err {
        CyphrError::TypeMismatch { expected, got, .. } => {
            assert_eq!(expected, "Map");
            assert_eq!(got, "Integer");
        }
        other => panic!("expected TypeMismatch, got: {other}"),
    }
}

// --- More numeric types ---

#[test]
fn test_from_value_u32() {
    let val = BoltType::Integer(neo4rs::BoltInteger { value: 100 });
    let res = u32::from_value(val).unwrap();
    assert_eq!(res, 100u32);
}

#[test]
fn test_from_value_i16() {
    let val = BoltType::Integer(neo4rs::BoltInteger { value: -300 });
    let res = i16::from_value(val).unwrap();
    assert_eq!(res, -300i16);
}

#[test]
fn test_from_value_u8() {
    let val = BoltType::Integer(neo4rs::BoltInteger { value: 255 });
    let res = u8::from_value(val).unwrap();
    assert_eq!(res, 255u8);
}

// --- Point2D / Point3D ---

#[test]
fn test_point2d_from_value() {
    let val = BoltType::Point2D(neo4rs::BoltPoint2D {
        sr_id: neo4rs::BoltInteger::new(4326),
        x: neo4rs::BoltFloat::new(1.0),
        y: neo4rs::BoltFloat::new(2.0),
    });
    let p = Point2D::from_value(val).unwrap();
    assert_eq!(p.sr_id, 4326);
    assert_eq!(p.x, 1.0);
    assert_eq!(p.y, 2.0);
}

#[test]
fn test_point3d_from_value() {
    let val = BoltType::Point3D(neo4rs::BoltPoint3D {
        sr_id: neo4rs::BoltInteger::new(4979),
        x: neo4rs::BoltFloat::new(1.0),
        y: neo4rs::BoltFloat::new(2.0),
        z: neo4rs::BoltFloat::new(3.0),
    });
    let p = Point3D::from_value(val).unwrap();
    assert_eq!(p.sr_id, 4979);
    assert_eq!(p.x, 1.0);
    assert_eq!(p.y, 2.0);
    assert_eq!(p.z, 3.0);
}

#[test]
fn test_point2d_type_mismatch() {
    let val = BoltType::Integer(neo4rs::BoltInteger { value: 1 });
    let err = Point2D::from_value(val).unwrap_err();
    match &err {
        CyphrError::TypeMismatch { expected, got, .. } => {
            assert_eq!(expected, "Point2D");
            assert_eq!(got, "Integer");
        }
        other => panic!("expected TypeMismatch, got: {other}"),
    }
}

// --- CyphrBytes ---

#[test]
fn test_cyphr_bytes_from_value() {
    let val = BoltType::Bytes(neo4rs::BoltBytes::new(bytes::Bytes::from_static(b"hello")));
    let b = CyphrBytes::from_value(val).unwrap();
    assert_eq!(b.0, b"hello");
}

#[test]
fn test_cyphr_bytes_type_mismatch() {
    let val = BoltType::Integer(neo4rs::BoltInteger { value: 1 });
    let err = CyphrBytes::from_value(val).unwrap_err();
    match &err {
        CyphrError::TypeMismatch { expected, got, .. } => {
            assert_eq!(expected, "Bytes");
            assert_eq!(got, "Integer");
        }
        other => panic!("expected TypeMismatch, got: {other}"),
    }
}

// --- CyphrPath ---

struct TestPathNode {
    name: String,
}

impl CyphrNode for TestPathNode {
    const LABEL: &'static str = "TestPathNode";
    fn from_node(node: &neo4rs::BoltNode) -> Result<Self, CyphrError> {
        let v = cyphr_core::props::node_prop(node, "name")
            .ok_or_else(|| CyphrError::missing_property("name", "TestPathNode"))?;
        Ok(TestPathNode {
            name: String::from_value(v)?,
        })
    }
}

#[test]
fn test_cyphr_path_from_value() {
    let mark = neo4rs::BoltNode::new(
        neo4rs::BoltInteger::new(1),
        vec![neo4rs::BoltType::from("TestPathNode")].into(),
        vec![(neo4rs::BoltString::from("name"), neo4rs::BoltType::from("Mark"))]
            .into_iter()
            .collect(),
    );
    let james = neo4rs::BoltNode::new(
        neo4rs::BoltInteger::new(2),
        vec![neo4rs::BoltType::from("TestPathNode")].into(),
        vec![(neo4rs::BoltString::from("name"), neo4rs::BoltType::from("James"))]
            .into_iter()
            .collect(),
    );
    let friend = neo4rs::BoltUnboundedRelation::new(
        neo4rs::BoltInteger::new(10),
        neo4rs::BoltString::from("KNOWS"),
        vec![(neo4rs::BoltString::from("since"), neo4rs::BoltType::from(2020))]
            .into_iter()
            .collect(),
    );
    let path = neo4rs::BoltPath {
        nodes: vec![neo4rs::BoltType::Node(mark), neo4rs::BoltType::Node(james)].into(),
        rels: vec![neo4rs::BoltType::UnboundedRelation(friend)].into(),
        indices: vec![neo4rs::BoltType::Integer(neo4rs::BoltInteger::new(1)), neo4rs::BoltType::Integer(neo4rs::BoltInteger::new(1))].into(),
    };
    let val = BoltType::Path(path);
    let cp = CyphrPath::<TestPathNode>::from_value(val).unwrap();
    assert_eq!(cp.nodes.len(), 2);
    assert_eq!(cp.nodes[0].name, "Mark");
    assert_eq!(cp.nodes[1].name, "James");
    assert_eq!(cp.rels.len(), 1);
    assert_eq!(cp.indices.len(), 2);
}

// --- Error context chaining ---

#[test]
fn test_error_with_context() {
    let err = CyphrError::type_mismatch("Integer", "String", "age");
    let ctx = err.with_context("User::age");
    let msg = ctx.to_string();
    assert!(msg.contains("User::age"));
    assert!(msg.contains("type mismatch"));
}

#[test]
fn test_error_context_variant() {
    let inner = CyphrError::missing_property("name", "User");
    let outer = inner.with_context("parsing User");
    match &outer {
        CyphrError::Context { context, source } => {
            assert_eq!(context, "parsing User");
            assert!(matches!(source.as_ref(), CyphrError::MissingProperty { .. }));
        }
        other => panic!("expected Context, got: {other}"),
    }
}

// --- IntoCyphrValue ---

#[test]
fn test_into_value_string() {
    let val = String::from("hello").into_value();
    match val {
        BoltType::String(s) => assert_eq!(s.value, "hello"),
        other => panic!("expected String, got: {other:?}"),
    }
}

#[test]
fn test_into_value_i64() {
    let val = 42_i64.into_value();
    match val {
        BoltType::Integer(i) => assert_eq!(i.value, 42),
        other => panic!("expected Integer, got: {other:?}"),
    }
}

#[test]
fn test_into_value_point2d() {
    let p = Point2D { sr_id: 4326, x: 1.0, y: 2.0 };
    let val = p.into_value();
    match val {
        BoltType::Point2D(p) => {
            assert_eq!(p.sr_id.value, 4326);
            assert_eq!(p.x.value, 1.0);
            assert_eq!(p.y.value, 2.0);
        }
        other => panic!("expected Point2D, got: {other:?}"),
    }
}

#[test]
fn test_into_value_point3d() {
    let p = Point3D { sr_id: 4979, x: 1.0, y: 2.0, z: 3.0 };
    let val = p.into_value();
    match val {
        BoltType::Point3D(p) => {
            assert_eq!(p.sr_id.value, 4979);
            assert_eq!(p.x.value, 1.0);
            assert_eq!(p.y.value, 2.0);
            assert_eq!(p.z.value, 3.0);
        }
        other => panic!("expected Point3D, got: {other:?}"),
    }
}

#[test]
fn test_into_value_cyphr_bytes() {
    let b = CyphrBytes(vec![1, 2, 3]);
    let val = b.into_value();
    match val {
        BoltType::Bytes(b) => assert_eq!(&b.value[..], &[1, 2, 3]),
        other => panic!("expected Bytes, got: {other:?}"),
    }
}
