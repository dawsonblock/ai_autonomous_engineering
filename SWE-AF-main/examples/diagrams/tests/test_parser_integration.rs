use diagrams::lexer::Lexer;
use diagrams::parser::Parser;

#[test]
fn test_parse_complete_diagram() {
    let input = r#"node "API Gateway" as api
node "Database" as db [type: database]
api -> db : "SQL query"
"#;

    let mut lexer = Lexer::new(input);
    let tokens = lexer.tokenize().expect("Lexer should succeed");

    let mut parser = Parser::new(tokens);
    let diagram = parser.parse().expect("Parser should succeed");

    assert_eq!(diagram.nodes.len(), 2);
    assert_eq!(diagram.connections.len(), 1);

    assert_eq!(diagram.nodes[0].identifier, "api");
    assert_eq!(diagram.nodes[0].display_name, "API Gateway");

    assert_eq!(diagram.nodes[1].identifier, "db");
    assert_eq!(diagram.nodes[1].display_name, "Database");

    assert_eq!(diagram.connections[0].from, "api");
    assert_eq!(diagram.connections[0].to, "db");
    assert_eq!(diagram.connections[0].label, Some("SQL query".to_string()));
}

#[test]
fn test_parse_with_comments() {
    let input = r#"# This is a comment
node "API" as api
# Another comment
api -> db
"#;

    let mut lexer = Lexer::new(input);
    let tokens = lexer.tokenize().expect("Lexer should succeed");

    let mut parser = Parser::new(tokens);
    let diagram = parser.parse().expect("Parser should succeed");

    assert_eq!(diagram.nodes.len(), 1);
    assert_eq!(diagram.connections.len(), 1);
}

#[test]
fn test_parser_error_on_invalid_syntax() {
    let input = "node invalid syntax";

    let mut lexer = Lexer::new(input);
    let tokens = lexer.tokenize().expect("Lexer should succeed");

    let mut parser = Parser::new(tokens);
    let result = parser.parse();

    assert!(result.is_err());
}

#[test]
fn test_parse_all_node_types() {
    let input = r#"node "Service" as svc [type: service]
node "DB" as db [type: database]
node "External" as ext [type: external]
node "Queue" as q [type: queue]
"#;

    let mut lexer = Lexer::new(input);
    let tokens = lexer.tokenize().expect("Lexer should succeed");

    let mut parser = Parser::new(tokens);
    let diagram = parser.parse().expect("Parser should succeed");

    assert_eq!(diagram.nodes.len(), 4);
    assert_eq!(
        diagram.nodes[0].node_type,
        diagrams::types::NodeType::Service
    );
    assert_eq!(
        diagram.nodes[1].node_type,
        diagrams::types::NodeType::Database
    );
    assert_eq!(
        diagram.nodes[2].node_type,
        diagrams::types::NodeType::External
    );
    assert_eq!(diagram.nodes[3].node_type, diagrams::types::NodeType::Queue);
}

#[test]
fn test_parse_connection_without_label() {
    let input = r#"node "A" as a
node "B" as b
a -> b
"#;

    let mut lexer = Lexer::new(input);
    let tokens = lexer.tokenize().expect("Lexer should succeed");

    let mut parser = Parser::new(tokens);
    let diagram = parser.parse().expect("Parser should succeed");

    assert_eq!(diagram.nodes.len(), 2);
    assert_eq!(diagram.connections.len(), 1);
    assert_eq!(diagram.connections[0].label, None);
}

#[test]
fn test_parse_multiple_connections() {
    let input = r#"node "A" as a
node "B" as b
node "C" as c
a -> b : "first"
b -> c : "second"
a -> c : "direct"
"#;

    let mut lexer = Lexer::new(input);
    let tokens = lexer.tokenize().expect("Lexer should succeed");

    let mut parser = Parser::new(tokens);
    let diagram = parser.parse().expect("Parser should succeed");

    assert_eq!(diagram.nodes.len(), 3);
    assert_eq!(diagram.connections.len(), 3);
    assert_eq!(diagram.connections[0].label, Some("first".to_string()));
    assert_eq!(diagram.connections[1].label, Some("second".to_string()));
    assert_eq!(diagram.connections[2].label, Some("direct".to_string()));
}
