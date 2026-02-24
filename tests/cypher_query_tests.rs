use cyphr::cypher_query;

#[test]
fn test_cypher_query_with_params() {
    let name = "Alice";
    let age: i64 = 30;
    let _q = cypher_query! {
        MATCH (u:User {name: $name, age: $age}) RETURN u
    };
}

#[test]
fn test_cypher_query_no_params() {
    let _q = cypher_query! {
        MATCH (u:User) RETURN u
    };
}

#[test]
fn test_cypher_query_dedup_params() {
    let name = "Alice";
    // $name appears twice but should only generate one .param() call
    let _q = cypher_query! {
        MATCH (u:User {name: $name}) WHERE u.name = $name RETURN u
    };
}
