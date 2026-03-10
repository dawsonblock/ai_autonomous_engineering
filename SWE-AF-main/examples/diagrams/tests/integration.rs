//! Comprehensive Integration Tests for Diagrams DSL Compiler
//!
//! This test suite provides end-to-end integration testing covering:
//! - Complete compilation workflow (file I/O → lex → parse → validate → layout → render → write)
//! - Preview workflow (file I/O → lex → parse → validate → layout → ASCII render)
//! - Validation workflow (file I/O → lex → parse → validate)
//! - Syntax error handling
//! - Semantic error handling
//! - I/O error handling
//! - Edge cases and boundary conditions

use diagrams::app::App;
use diagrams::error::DiagramError;
use diagrams::layout::LayoutEngine;
use diagrams::lexer::Lexer;
use diagrams::parser::Parser;
use diagrams::validator::Validator;
use std::fs;
use std::io::Write;
use tempfile::{NamedTempFile, TempDir};

// =============================================================================
// Helper Functions
// =============================================================================

/// Helper to create a temporary file with DSL content
fn create_dsl_file(content: &str) -> NamedTempFile {
    let mut file = NamedTempFile::new().expect("Failed to create temp file");
    file.write_all(content.as_bytes())
        .expect("Failed to write DSL content");
    file.flush().expect("Failed to flush temp file");
    file
}

// =============================================================================
// Section 1: Complete Compile Workflow Tests
// =============================================================================

#[test]
fn test_compile_workflow_simple_diagram() {
    // Complete pipeline test: file I/O → lex → parse → validate → layout → render → write
    let dsl_content = r#"node "API Gateway" as api
node "Backend Service" as backend
node "Database" as db

api -> backend : "HTTP/REST"
backend -> db : "SQL queries"
"#;

    let input_file = create_dsl_file(dsl_content);
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let output_path = temp_dir.path().join("diagram.svg");

    // Execute compile
    let result = App::compile(input_file.path(), &output_path);

    // Verify success
    assert!(result.is_ok(), "compile() should succeed for valid DSL");

    // Verify SVG file was created
    assert!(
        output_path.exists(),
        "SVG output file should exist after compilation"
    );

    // Verify SVG content structure
    let svg_content = fs::read_to_string(&output_path).expect("Failed to read SVG output");

    // Check SVG structure
    assert!(
        svg_content.starts_with("<svg"),
        "Output should start with <svg tag"
    );
    assert!(
        svg_content.contains("</svg>"),
        "Output should contain closing </svg> tag"
    );
    assert!(
        svg_content.contains("xmlns"),
        "SVG should have xmlns attribute"
    );

    // Check that node labels are present
    assert!(
        svg_content.contains("API Gateway"),
        "SVG should contain 'API Gateway' label"
    );
    assert!(
        svg_content.contains("Backend Service"),
        "SVG should contain 'Backend Service' label"
    );
    assert!(
        svg_content.contains("Database"),
        "SVG should contain 'Database' label"
    );

    // Check that connection labels are present
    assert!(
        svg_content.contains("HTTP/REST"),
        "SVG should contain 'HTTP/REST' connection label"
    );
    assert!(
        svg_content.contains("SQL queries"),
        "SVG should contain 'SQL queries' connection label"
    );
}

#[test]
fn test_compile_workflow_complex_diagram() {
    // Complex diagram to test full pipeline integration with multiple nodes and connections
    let dsl_content = r#"node "Load Balancer" as lb
node "Web Server 1" as web1
node "Web Server 2" as web2
node "Application Server" as app
node "Database Primary" as db_primary
node "Database Replica" as db_replica

lb -> web1 : "distribute"
lb -> web2 : "distribute"
web1 -> app : "forward"
web2 -> app : "forward"
app -> db_primary : "write"
app -> db_replica : "read"
"#;

    let input_file = create_dsl_file(dsl_content);
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let output_path = temp_dir.path().join("complex.svg");

    // Test compile
    let compile_result = App::compile(input_file.path(), &output_path);
    assert!(
        compile_result.is_ok(),
        "compile() should succeed for complex diagram"
    );

    // Verify SVG contains all nodes and connections
    let svg_content = fs::read_to_string(&output_path).expect("Failed to read SVG");

    let expected_labels = [
        "Load Balancer",
        "Web Server 1",
        "Web Server 2",
        "Application Server",
        "Database Primary",
        "Database Replica",
        "distribute",
        "forward",
        "write",
        "read",
    ];

    for label in &expected_labels {
        assert!(
            svg_content.contains(label),
            "SVG should contain label '{}'",
            label
        );
    }
}

#[test]
fn test_compile_workflow_empty_diagram() {
    // Empty diagram (only whitespace and comments) should compile
    let dsl_content = "# This is just a comment\n\n  \n";

    let input_file = create_dsl_file(dsl_content);
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let output_path = temp_dir.path().join("empty.svg");

    // Execute compile
    let result = App::compile(input_file.path(), &output_path);

    // Verify success
    assert!(result.is_ok(), "compile() should succeed for empty diagram");

    // Verify SVG file was created
    assert!(output_path.exists(), "SVG file should be created");

    let content = fs::read_to_string(&output_path).expect("Failed to read SVG");
    assert!(content.starts_with("<svg"), "Output should be valid SVG");
}

#[test]
fn test_compile_workflow_overwrites_existing_file() {
    let dsl_content = r#"node "New Content" as new"#;
    let input_file = create_dsl_file(dsl_content);

    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let output_path = temp_dir.path().join("output.svg");

    // Write initial content to output path
    fs::write(
        &output_path,
        "This is old content that should be overwritten",
    )
    .expect("Failed to write initial content");

    // Execute compile
    let result = App::compile(input_file.path(), &output_path);

    // Verify success
    assert!(result.is_ok(), "compile() should succeed");

    // Verify file was overwritten with new SVG content
    let content = fs::read_to_string(&output_path).expect("Failed to read output file");

    assert!(
        content.starts_with("<svg"),
        "File should be overwritten with SVG content"
    );
    assert!(
        !content.contains("old content"),
        "Old content should be completely replaced"
    );
    assert!(
        content.contains("New Content"),
        "New SVG should contain new node label"
    );
}

// =============================================================================
// Section 2: Preview Workflow Tests
// =============================================================================

#[test]
fn test_preview_workflow_returns_ascii_art() {
    // Complete preview pipeline: file I/O → lex → parse → validate → layout → ASCII render
    let dsl_content = r#"node "Service A" as svc_a
node "Service B" as svc_b
svc_a -> svc_b : "API call"
"#;

    let input_file = create_dsl_file(dsl_content);

    // Execute preview
    let result = App::preview(input_file.path());

    // Verify success
    assert!(result.is_ok(), "preview() should succeed for valid DSL");

    let ascii_output = result.unwrap();

    // Verify ASCII art contains box-drawing characters
    let has_box_chars = ascii_output.contains('┌')
        || ascii_output.contains('─')
        || ascii_output.contains('│')
        || ascii_output.contains('┐')
        || ascii_output.contains('└')
        || ascii_output.contains('┘');

    assert!(
        has_box_chars,
        "ASCII output should contain box-drawing characters"
    );
}

#[test]
fn test_preview_workflow_complex_diagram() {
    let dsl_content = r#"node "API Gateway" as api
node "Database" as db
node "Cache" as cache
api -> db : "query"
api -> cache : "get"
"#;

    let input_file = create_dsl_file(dsl_content);

    // Execute preview
    let result = App::preview(input_file.path());

    // Verify success
    assert!(result.is_ok(), "preview() should succeed for valid DSL");

    let ascii_output = result.unwrap();

    // Verify output is non-empty
    assert!(!ascii_output.is_empty(), "ASCII output should not be empty");
}

// =============================================================================
// Section 3: Validate Workflow Tests
// =============================================================================

#[test]
fn test_validate_workflow_accepts_valid_dsl() {
    // Validation pipeline: file I/O → lex → parse → validate
    let dsl_content = r#"node "Web Server" as web
node "App Server" as app
node "Cache" as cache

web -> app : "requests"
app -> cache : "lookup"
"#;

    let input_file = create_dsl_file(dsl_content);

    // Execute validate
    let result = App::validate(input_file.path());

    // Verify success
    assert!(result.is_ok(), "validate() should return Ok for valid DSL");
}

#[test]
fn test_validate_workflow_nodes_only() {
    let dsl_content = r#"node "Service A" as svc_a
node "Service B" as svc_b
"#;

    let input_file = create_dsl_file(dsl_content);

    // Execute validate
    let result = App::validate(input_file.path());

    // Verify success
    assert!(
        result.is_ok(),
        "validate() should succeed for nodes-only diagram"
    );
}

#[test]
fn test_validate_workflow_with_node_types() {
    let dsl_content = r#"node "API Gateway" as api [type: service]
node "PostgreSQL" as db [type: database]
api -> db
"#;

    let input_file = create_dsl_file(dsl_content);

    // Execute validate
    let result = App::validate(input_file.path());

    // Verify success
    assert!(
        result.is_ok(),
        "validate() should succeed for diagram with node types"
    );
}

// =============================================================================
// Section 4: Syntax Error Handling Tests
// =============================================================================

#[test]
fn test_syntax_error_invalid_dsl() {
    let dsl_content = "this is not valid DSL syntax at all";

    let input_file = create_dsl_file(dsl_content);

    // Execute validate
    let result = App::validate(input_file.path());

    // Verify failure
    assert!(result.is_err(), "validate() should fail for invalid syntax");

    // Verify correct error type
    match result.unwrap_err() {
        DiagramError::Syntax(_) => {} // Expected
        other => panic!("Expected Syntax error, got {:?}", other),
    }
}

#[test]
fn test_syntax_error_missing_as_keyword() {
    // Missing "as" keyword in node definition
    let dsl_content = r#"node "API Gateway" api"#;

    let mut lexer = Lexer::new(dsl_content);
    let tokens = lexer
        .tokenize()
        .expect("Lexer should tokenize even invalid DSL");

    let mut parser = Parser::new(tokens);
    let result = parser.parse();

    // Should fail with syntax error
    assert!(result.is_err(), "Parser should fail on invalid syntax");
    match result {
        Err(DiagramError::Syntax(err)) => {
            assert!(
                err.message.contains("expected"),
                "Error message should indicate what was expected"
            );
        }
        _ => panic!("Expected SyntaxError"),
    }
}

#[test]
fn test_syntax_error_invalid_character() {
    // Invalid character in DSL
    let dsl_content = r#"node "API" @ api"#;

    let mut lexer = Lexer::new(dsl_content);
    let result = lexer.tokenize();

    assert!(result.is_err(), "Lexer should fail on invalid character");
    match result {
        Err(DiagramError::Syntax(err)) => {
            assert!(err.message.contains("unexpected character"));
        }
        _ => panic!("Expected SyntaxError from lexer"),
    }
}

// =============================================================================
// Section 5: Semantic Error Handling Tests
// =============================================================================

#[test]
fn test_semantic_error_undefined_node() {
    // Syntactically valid but semantically invalid (undefined node reference)
    let dsl_content = r#"node "Service A" as svc_a
svc_a -> undefined_service : "connection to nowhere"
"#;

    let input_file = create_dsl_file(dsl_content);

    // Execute validate
    let result = App::validate(input_file.path());

    // Verify failure
    assert!(
        result.is_err(),
        "validate() should fail for semantic errors"
    );

    // Verify correct error type
    match result.unwrap_err() {
        DiagramError::Semantic(_) => {} // Expected
        other => panic!("Expected Semantic error, got {:?}", other),
    }
}

#[test]
fn test_semantic_error_self_connection() {
    let dsl_content = r#"node "API" as api
api -> api
"#;

    let mut lexer = Lexer::new(dsl_content);
    let tokens = lexer.tokenize().expect("Lexer should succeed");

    let mut parser = Parser::new(tokens);
    let diagram = parser.parse().expect("Parser should succeed");

    let result = Validator::validate(&diagram);
    assert!(result.is_err(), "Validator should fail on self-connection");
    match result {
        Err(DiagramError::Semantic(_)) => {
            // Expected
        }
        _ => panic!("Expected SemanticError for self-connection"),
    }
}

#[test]
fn test_semantic_error_duplicate_node() {
    let dsl_content = r#"node "API 1" as api
node "API 2" as api
"#;

    let mut lexer = Lexer::new(dsl_content);
    let tokens = lexer.tokenize().expect("Lexer should succeed");

    let mut parser = Parser::new(tokens);
    let diagram = parser.parse().expect("Parser should succeed");

    let result = Validator::validate(&diagram);
    assert!(result.is_err(), "Validator should fail on duplicate node");
    match result {
        Err(DiagramError::Semantic(_)) => {
            // Expected
        }
        _ => panic!("Expected SemanticError for duplicate node"),
    }
}

// =============================================================================
// Section 6: I/O Error Handling Tests
// =============================================================================

#[test]
fn test_io_error_nonexistent_input_file() {
    let nonexistent_path = "/this/path/does/not/exist/input.dsl";

    // Execute compile with nonexistent file
    let result = App::compile(nonexistent_path, "/tmp/output.svg");

    // Verify failure
    assert!(
        result.is_err(),
        "compile() should fail for nonexistent input file"
    );

    // Verify correct error type and message
    match result.unwrap_err() {
        DiagramError::Io(e) => {
            assert!(
                e.message.contains("Failed to read file"),
                "Error message should indicate read failure"
            );
            assert!(
                e.message.contains(nonexistent_path),
                "Error message should contain the file path"
            );
        }
        other => panic!("Expected IoError, got {:?}", other),
    }
}

#[test]
fn test_io_error_unwritable_output_path() {
    let dsl_content = r#"node "Test" as test"#;
    let input_file = create_dsl_file(dsl_content);

    // Try to write to a directory that doesn't exist
    let unwritable_path = std::path::PathBuf::from("/nonexistent/directory/output.svg");

    // Execute compile with unwritable output path
    let result = App::compile(input_file.path(), &unwritable_path);

    // Verify failure
    assert!(
        result.is_err(),
        "compile() should fail for unwritable output path"
    );

    // Verify correct error type and message
    match result.unwrap_err() {
        DiagramError::Io(e) => {
            assert!(
                e.message.contains("Failed to write file"),
                "Error message should indicate write failure"
            );
            assert!(
                e.message.contains("/nonexistent/directory/output.svg"),
                "Error message should contain the file path"
            );
        }
        other => panic!("Expected IoError, got {:?}", other),
    }
}

#[test]
fn test_io_error_preview_nonexistent_file() {
    let nonexistent_path = "/no/such/file.dsl";

    // Execute preview with nonexistent file
    let result = App::preview(nonexistent_path);

    // Verify failure
    assert!(
        result.is_err(),
        "preview() should fail for nonexistent input file"
    );

    // Verify correct error type
    match result.unwrap_err() {
        DiagramError::Io(_) => {} // Expected
        other => panic!("Expected IoError, got {:?}", other),
    }
}

// =============================================================================
// Section 7: Edge Case Tests
// =============================================================================

#[test]
fn test_edge_case_single_node() {
    // Diagram with only one node and no connections
    let dsl_content = r#"node "Lonely Service" as lonely"#;

    let input_file = create_dsl_file(dsl_content);
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let output_path = temp_dir.path().join("single.svg");

    // Test compile
    let result = App::compile(input_file.path(), &output_path);
    assert!(result.is_ok(), "compile() should succeed for single node");

    // Verify SVG contains the node
    let svg_content = fs::read_to_string(&output_path).expect("Failed to read SVG");
    assert!(
        svg_content.contains("Lonely Service"),
        "SVG should contain the node label"
    );
}

#[test]
fn test_edge_case_connection_without_label() {
    let dsl_content = r#"node "API" as api
node "DB" as db
api -> db
"#;

    let input_file = create_dsl_file(dsl_content);
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let output_path = temp_dir.path().join("no_label.svg");

    // Test compile
    let result = App::compile(input_file.path(), &output_path);
    assert!(
        result.is_ok(),
        "compile() should succeed for connection without label"
    );

    // Also test through parser to verify connection structure
    let mut lexer = Lexer::new(dsl_content);
    let tokens = lexer.tokenize().expect("Lexer should succeed");
    let mut parser = Parser::new(tokens);
    let diagram = parser.parse().expect("Parser should succeed");

    assert_eq!(diagram.connections.len(), 1);
    assert_eq!(diagram.connections[0].label, None);
}

#[test]
fn test_edge_case_very_long_node_names() {
    // Test with very long node names (boundary condition)
    let long_name = "A".repeat(200);
    let dsl_content = format!(
        r#"node "{}" as long_node
node "Short" as short
long_node -> short
"#,
        long_name
    );

    let input_file = create_dsl_file(&dsl_content);
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let output_path = temp_dir.path().join("long_names.svg");

    // Test compile
    let result = App::compile(input_file.path(), &output_path);
    assert!(
        result.is_ok(),
        "compile() should succeed for very long node names"
    );

    // Verify SVG contains the long name
    let svg_content = fs::read_to_string(&output_path).expect("Failed to read SVG");
    assert!(
        svg_content.contains(&long_name),
        "SVG should contain the long node name"
    );
}

#[test]
fn test_edge_case_special_characters_in_labels() {
    // Test with special characters in labels
    let dsl_content = r#"node "API (v2.0)" as api
node "DB-Cluster #1" as db
node "Cache: Redis" as cache
api -> db : "Query: SELECT *"
db -> cache : "Update & Sync"
"#;

    let input_file = create_dsl_file(dsl_content);
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let output_path = temp_dir.path().join("special_chars.svg");

    // Test compile
    let result = App::compile(input_file.path(), &output_path);
    assert!(
        result.is_ok(),
        "compile() should succeed for labels with special characters"
    );

    // Verify SVG contains the special characters
    let svg_content = fs::read_to_string(&output_path).expect("Failed to read SVG");
    assert!(svg_content.contains("API (v2.0)"));
    assert!(svg_content.contains("DB-Cluster #1"));
    assert!(svg_content.contains("Cache: Redis"));
}

#[test]
fn test_edge_case_many_nodes_stress_test() {
    // Stress test with many nodes (performance and correctness)
    let mut dsl_content = String::new();
    for i in 0..50 {
        dsl_content.push_str(&format!(r#"node "Service {}" as svc_{}"#, i, i));
        dsl_content.push('\n');
    }

    // Add some connections
    for i in 0..49 {
        dsl_content.push_str(&format!("svc_{} -> svc_{}\n", i, i + 1));
    }

    let input_file = create_dsl_file(&dsl_content);
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let output_path = temp_dir.path().join("many_nodes.svg");

    // Test compile
    let result = App::compile(input_file.path(), &output_path);
    assert!(result.is_ok(), "compile() should succeed for many nodes");

    // Verify SVG was created and contains expected content
    let svg_content = fs::read_to_string(&output_path).expect("Failed to read SVG");
    assert!(svg_content.contains("Service 0"));
    assert!(svg_content.contains("Service 49"));
}

#[test]
fn test_edge_case_unicode_characters() {
    // Test with Unicode characters in labels
    let dsl_content = r#"node "データベース" as db
node "API 服务器" as api
node "Cache 🚀" as cache
api -> db : "クエリ"
db -> cache : "更新"
"#;

    let input_file = create_dsl_file(dsl_content);
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let output_path = temp_dir.path().join("unicode.svg");

    // Test compile
    let result = App::compile(input_file.path(), &output_path);
    assert!(
        result.is_ok(),
        "compile() should succeed for Unicode characters"
    );

    // Verify SVG contains Unicode content
    let svg_content = fs::read_to_string(&output_path).expect("Failed to read SVG");
    assert!(svg_content.contains("データベース"));
    assert!(svg_content.contains("API 服务器"));
}

#[test]
fn test_edge_case_multiple_connections_same_nodes() {
    // Test multiple connections between the same pair of nodes
    let dsl_content = r#"node "Service A" as svc_a
node "Service B" as svc_b
svc_a -> svc_b : "HTTP"
svc_a -> svc_b : "WebSocket"
svc_a -> svc_b : "gRPC"
"#;

    let input_file = create_dsl_file(dsl_content);
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let output_path = temp_dir.path().join("multi_conn.svg");

    // Test compile
    let result = App::compile(input_file.path(), &output_path);
    assert!(
        result.is_ok(),
        "compile() should succeed for multiple connections between same nodes"
    );

    // Verify all connection labels are present
    let svg_content = fs::read_to_string(&output_path).expect("Failed to read SVG");
    assert!(svg_content.contains("HTTP"));
    assert!(svg_content.contains("WebSocket"));
    assert!(svg_content.contains("gRPC"));
}

#[test]
fn test_edge_case_whitespace_variations() {
    // Test various whitespace patterns (tabs, multiple spaces, etc.)
    let dsl_content = "node \"API\" as api\n\n\nnode \"DB\" as db\n\t\napi -> db\n  \n";

    let input_file = create_dsl_file(dsl_content);

    // Test validate
    let result = App::validate(input_file.path());
    assert!(
        result.is_ok(),
        "validate() should handle various whitespace patterns"
    );
}

#[test]
fn test_edge_case_comments_everywhere() {
    // Test comments in various positions
    let dsl_content = r#"# Header comment
node "API" as api # inline comment
# Middle comment
node "DB" as db
# Another comment
api -> db : "query" # connection comment
# Trailing comment
"#;

    let input_file = create_dsl_file(dsl_content);
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let output_path = temp_dir.path().join("comments.svg");

    // Test compile
    let result = App::compile(input_file.path(), &output_path);
    assert!(result.is_ok(), "compile() should handle comments correctly");
}

// =============================================================================
// Section 8: Full Pipeline Integration Tests
// =============================================================================

#[test]
fn test_full_pipeline_lex_parse_validate_layout() {
    // Smoke test that verifies the entire parsing and layout pipeline works end-to-end
    // Tests: Lexer → Parser → Validator → Layout
    let dsl = r#"node "API Gateway" as api
node "Database" as db
api -> db : "SQL query"
"#;

    // Step 1: Tokenize with Lexer
    let mut lexer = Lexer::new(dsl);
    let tokens = lexer
        .tokenize()
        .expect("Lexer should successfully tokenize");
    assert!(
        !tokens.is_empty(),
        "Lexer should produce non-empty token stream"
    );

    // Step 2: Parse to Diagram AST with Parser
    let mut parser = Parser::new(tokens);
    let diagram = parser.parse().expect("Parser should successfully parse");
    assert_eq!(
        diagram.nodes.len(),
        2,
        "Diagram should contain exactly 2 nodes"
    );
    assert_eq!(
        diagram.connections.len(),
        1,
        "Diagram should contain exactly 1 connection"
    );

    // Verify node details
    assert_eq!(diagram.nodes[0].identifier, "api");
    assert_eq!(diagram.nodes[0].display_name, "API Gateway");
    assert_eq!(diagram.nodes[1].identifier, "db");
    assert_eq!(diagram.nodes[1].display_name, "Database");

    // Verify connection details
    assert_eq!(diagram.connections[0].from, "api");
    assert_eq!(diagram.connections[0].to, "db");
    assert_eq!(diagram.connections[0].label, Some("SQL query".to_string()));

    // Step 3: Validate with Validator
    let validation_result = Validator::validate(&diagram);
    assert!(
        validation_result.is_ok(),
        "Validator should pass for valid diagram"
    );

    // Step 4: Compute layout with LayoutEngine
    let layout = LayoutEngine::layout(&diagram);

    // Verify LayoutDiagram structure
    assert_eq!(
        layout.nodes.len(),
        2,
        "LayoutDiagram should contain 2 positioned nodes"
    );
    assert_eq!(
        layout.connections.len(),
        1,
        "LayoutDiagram should contain 1 positioned connection"
    );

    // Verify positioned nodes have valid coordinates
    for positioned_node in &layout.nodes {
        assert!(
            positioned_node.position.x >= 0.0,
            "Node x coordinate should be non-negative"
        );
        assert!(
            positioned_node.position.y >= 0.0,
            "Node y coordinate should be non-negative"
        );
        assert!(positioned_node.width > 0.0, "Node width should be positive");
        assert!(
            positioned_node.height > 0.0,
            "Node height should be positive"
        );
    }

    // Verify positioned connection has valid coordinates
    let conn = &layout.connections[0];
    assert!(
        conn.start.x >= 0.0 && conn.start.y >= 0.0,
        "Connection start point should have valid coordinates"
    );
    assert!(
        conn.end.x >= 0.0 && conn.end.y >= 0.0,
        "Connection end point should have valid coordinates"
    );

    // Verify layout has valid dimensions
    assert!(layout.width > 0.0, "Layout width should be positive");
    assert!(layout.height > 0.0, "Layout height should be positive");
}

#[test]
fn test_full_pipeline_empty_input() {
    let dsl = "";

    let mut lexer = Lexer::new(dsl);
    let tokens = lexer
        .tokenize()
        .expect("Lexer should succeed on empty input");

    let mut parser = Parser::new(tokens);
    let diagram = parser
        .parse()
        .expect("Parser should succeed on empty input");

    assert_eq!(diagram.nodes.len(), 0);
    assert_eq!(diagram.connections.len(), 0);

    Validator::validate(&diagram).expect("Validator should succeed on empty diagram");

    let layout = LayoutEngine::layout(&diagram);
    assert_eq!(layout.nodes.len(), 0);
    assert_eq!(layout.connections.len(), 0);
    assert_eq!(layout.width, 0.0);
    assert_eq!(layout.height, 0.0);
}

#[test]
fn test_full_pipeline_all_workflows_consistent() {
    // Verify that compile, validate, and preview all work consistently on the same input
    let dsl_content = r#"node "Web Server" as web
node "App Server" as app
node "Cache" as cache

web -> app : "requests"
app -> cache : "lookup"
"#;

    let input_file = create_dsl_file(dsl_content);
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let output_path = temp_dir.path().join("output.svg");

    // Test compile
    let compile_result = App::compile(input_file.path(), &output_path);
    assert!(
        compile_result.is_ok(),
        "compile() should succeed for complex diagram"
    );

    // Test validate
    let validate_result = App::validate(input_file.path());
    assert!(
        validate_result.is_ok(),
        "validate() should succeed for complex diagram"
    );

    // Test preview
    let preview_result = App::preview(input_file.path());
    assert!(
        preview_result.is_ok(),
        "preview() should succeed for complex diagram"
    );

    // All three workflows should succeed for the same valid input
    assert!(output_path.exists(), "SVG file should be created");
    assert!(
        !preview_result.unwrap().is_empty(),
        "Preview output should not be empty"
    );
}
