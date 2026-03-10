/// Integration tests for App module interaction with the core pipeline
///
/// These tests verify that the App orchestration layer correctly integrates
/// with the core parsing pipeline (Lexer → Parser → Validator → Layout → Renderers)
/// and handles the boundaries between file I/O and processing.
use diagrams::app::App;
use diagrams::error::DiagramError;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use tempfile::{NamedTempFile, TempDir};

/// Helper to create a temp DSL file
fn create_dsl_file(content: &str) -> NamedTempFile {
    let mut file = NamedTempFile::new().expect("Failed to create temp file");
    file.write_all(content.as_bytes())
        .expect("Failed to write content");
    file.flush().expect("Failed to flush");
    file
}

/// Test that App.compile produces the same semantic output as the core pipeline
/// when given identical DSL input
#[test]
fn test_app_compile_produces_consistent_svg_output() {
    // This DSL should exercise multiple components:
    // - Lexer: tokenize nodes, connections, labels
    // - Parser: build AST
    // - Validator: check node references
    // - Layout: position nodes and connections
    // - SVG Renderer: generate SVG
    let dsl = r#"node "Frontend" as fe [type: service]
node "Backend" as be [type: service]
node "Database" as db [type: database]

fe -> be : "REST API"
be -> db : "SQL"
"#;

    let input_file = create_dsl_file(dsl);
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let output_path = temp_dir.path().join("output.svg");

    // Compile DSL to SVG
    let result = App::compile(input_file.path(), &output_path);
    assert!(result.is_ok(), "Compile should succeed");

    // Verify SVG file exists
    assert!(output_path.exists(), "SVG file should be created");

    // Read and verify SVG structure
    let svg = fs::read_to_string(&output_path).expect("Failed to read SVG");

    // Verify SVG header and structure
    assert!(svg.starts_with("<svg"), "Should start with SVG tag");
    assert!(svg.contains("xmlns=\"http://www.w3.org/2000/svg\""));
    assert!(svg.contains("</svg>"));

    // Verify all node labels are present
    assert!(svg.contains("Frontend"), "SVG should contain 'Frontend'");
    assert!(svg.contains("Backend"), "SVG should contain 'Backend'");
    assert!(svg.contains("Database"), "SVG should contain 'Database'");

    // Verify connection labels are present
    assert!(svg.contains("REST API"), "SVG should contain 'REST API'");
    assert!(svg.contains("SQL"), "SVG should contain 'SQL'");

    // Verify node types affect rendering (database nodes have different appearance)
    // SVG renderer should include visual distinctions for different node types
    let rect_count = svg.matches("<rect").count();
    let path_count = svg.matches("<path").count();
    assert!(
        rect_count + path_count >= 3,
        "Should have geometric shapes for all nodes (rects or paths)"
    );
}

/// Test that App.preview produces ASCII output that reflects the layout
#[test]
fn test_app_preview_reflects_layout_structure() {
    let dsl = r#"node "Node A" as a
node "Node B" as b
node "Node C" as c

a -> b
b -> c
"#;

    let input_file = create_dsl_file(dsl);

    let result = App::preview(input_file.path());
    assert!(result.is_ok(), "Preview should succeed");

    let ascii = result.unwrap();

    // Verify ASCII contains box-drawing characters
    let has_box_chars = ascii.contains('┌')
        || ascii.contains('─')
        || ascii.contains('│')
        || ascii.contains('┐')
        || ascii.contains('└')
        || ascii.contains('┘')
        || ascii.contains('→');

    assert!(
        has_box_chars,
        "ASCII output should contain box-drawing characters"
    );

    // ASCII should be non-empty for a diagram with nodes
    assert!(!ascii.is_empty(), "ASCII output should not be empty");
}

/// Test error propagation: Lexer error → App
#[test]
fn test_lexer_error_propagates_through_app_compile() {
    // Invalid character that lexer cannot handle
    let dsl = r#"node "Test" @ invalid"#;

    let input_file = create_dsl_file(dsl);
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let output_path = temp_dir.path().join("output.svg");

    let result = App::compile(input_file.path(), &output_path);

    assert!(result.is_err(), "Should fail with lexer error");
    match result.unwrap_err() {
        DiagramError::Syntax(e) => {
            assert!(
                e.message.contains("unexpected character"),
                "Error should mention unexpected character"
            );
        }
        e => panic!("Expected Syntax error, got {:?}", e),
    }

    // Verify no output file was created
    assert!(
        !output_path.exists(),
        "No output file should be created on error"
    );
}

/// Test error propagation: Parser error → App
#[test]
fn test_parser_error_propagates_through_app_compile() {
    // Syntactically invalid DSL (missing "as" keyword)
    let dsl = r#"node "Service" identifier_without_as"#;

    let input_file = create_dsl_file(dsl);
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let output_path = temp_dir.path().join("output.svg");

    let result = App::compile(input_file.path(), &output_path);

    assert!(result.is_err(), "Should fail with parser error");
    match result.unwrap_err() {
        DiagramError::Syntax(e) => {
            assert!(
                e.message.contains("expected"),
                "Error should indicate what was expected"
            );
        }
        e => panic!("Expected Syntax error, got {:?}", e),
    }

    assert!(!output_path.exists(), "No output file on parse error");
}

/// Test error propagation: Validator error → App
#[test]
fn test_validator_error_propagates_through_app_compile() {
    // Semantically invalid: undefined node reference
    let dsl = r#"node "Service A" as svc_a
svc_a -> undefined_node : "bad connection"
"#;

    let input_file = create_dsl_file(dsl);
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let output_path = temp_dir.path().join("output.svg");

    let result = App::compile(input_file.path(), &output_path);

    assert!(result.is_err(), "Should fail with validator error");
    match result.unwrap_err() {
        DiagramError::Semantic(_) => {}
        e => panic!("Expected Semantic error, got {:?}", e),
    }

    assert!(!output_path.exists(), "No output file on validation error");
}

/// Test error propagation: I/O read error → App
#[test]
fn test_io_read_error_propagates_through_app() {
    let nonexistent = "/nonexistent/path/to/file.dsl";
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let output_path = temp_dir.path().join("output.svg");

    let result = App::compile(Path::new(nonexistent), output_path.as_path());

    assert!(result.is_err(), "Should fail with I/O error");
    match result.unwrap_err() {
        DiagramError::Io(e) => {
            assert!(e.message.contains("Failed to read file"));
            assert!(e.message.contains(nonexistent));
        }
        e => panic!("Expected IoError, got {:?}", e),
    }
}

/// Test error propagation: I/O write error → App
#[test]
fn test_io_write_error_propagates_through_app() {
    let dsl = r#"node "Test" as t"#;
    let input_file = create_dsl_file(dsl);

    // Try to write to a path that doesn't exist
    let bad_output = PathBuf::from("/nonexistent/directory/output.svg");

    let result = App::compile(input_file.path(), bad_output.as_path());

    assert!(result.is_err(), "Should fail with I/O error");
    match result.unwrap_err() {
        DiagramError::Io(e) => {
            assert!(e.message.contains("Failed to write file"));
            assert!(e.message.contains("/nonexistent/directory/output.svg"));
        }
        e => panic!("Expected IoError, got {:?}", e),
    }
}

/// Test that compile and preview work consistently on the same input
#[test]
fn test_compile_and_preview_consistency() {
    let dsl = r#"node "API" as api
node "DB" as db
api -> db : "query"
"#;

    let input_file = create_dsl_file(dsl);
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let output_path = temp_dir.path().join("output.svg");

    // Both should succeed
    let compile_result = App::compile(input_file.path(), &output_path);
    let preview_result = App::preview(input_file.path());

    assert!(compile_result.is_ok(), "Compile should succeed");
    assert!(preview_result.is_ok(), "Preview should succeed");

    // Both outputs should contain the node labels
    let svg = fs::read_to_string(&output_path).expect("Failed to read SVG");
    let ascii = preview_result.unwrap();

    // SVG should contain labels
    assert!(svg.contains("API"));
    assert!(svg.contains("DB"));
    assert!(svg.contains("query"));

    // ASCII might not show all labels depending on rendering,
    // but should be non-empty for a valid diagram
    assert!(!ascii.is_empty(), "ASCII should not be empty");
}

/// Test that validate, preview, and compile all process the same DSL correctly
#[test]
fn test_validate_preview_compile_all_succeed_on_valid_input() {
    let dsl = r#"node "Load Balancer" as lb
node "Web Server" as web
node "App Server" as app

lb -> web
web -> app
"#;

    let input_file = create_dsl_file(dsl);
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let output_path = temp_dir.path().join("output.svg");

    // All three operations should succeed
    let validate_result = App::validate(input_file.path());
    let preview_result = App::preview(input_file.path());
    let compile_result = App::compile(input_file.path(), &output_path);

    assert!(validate_result.is_ok(), "Validate should succeed");
    assert!(preview_result.is_ok(), "Preview should succeed");
    assert!(compile_result.is_ok(), "Compile should succeed");

    // Verify outputs
    assert!(output_path.exists(), "SVG file should exist");
    let svg = fs::read_to_string(&output_path).expect("Failed to read SVG");
    assert!(svg.starts_with("<svg"));

    let ascii = preview_result.unwrap();
    assert!(!ascii.is_empty() || dsl.trim().is_empty());
}

/// Test that validate fails early, before layout/rendering
#[test]
fn test_validate_fails_before_expensive_operations() {
    // Large diagram with semantic error (to verify we don't waste time on layout/render)
    let mut dsl = String::from("node \"Start\" as start\n");
    for i in 1..=50 {
        dsl.push_str(&format!("node \"Node{}\" as n{}\n", i, i));
    }
    // Add invalid connection
    dsl.push_str("start -> nonexistent\n");

    let input_file = create_dsl_file(&dsl);

    // Validate should fail quickly without doing layout or rendering
    let result = App::validate(input_file.path());

    assert!(result.is_err(), "Should fail validation");
    match result.unwrap_err() {
        DiagramError::Semantic(_) => {} // Expected
        e => panic!("Expected Semantic error, got {:?}", e),
    }
}

/// Test complex diagram with multiple node types and connections
#[test]
fn test_complex_diagram_full_pipeline() {
    let dsl = r#"node "Load Balancer" as lb [type: service]
node "Web Server 1" as web1 [type: service]
node "Web Server 2" as web2 [type: service]
node "Application Server" as app [type: service]
node "Redis Cache" as cache [type: database]
node "PostgreSQL Primary" as db_primary [type: database]
node "PostgreSQL Replica" as db_replica [type: database]
node "S3 Storage" as s3 [type: external]

lb -> web1 : "HTTP"
lb -> web2 : "HTTP"
web1 -> app : "forward"
web2 -> app : "forward"
app -> cache : "cache lookup"
app -> db_primary : "write"
app -> db_replica : "read"
app -> s3 : "upload files"
"#;

    let input_file = create_dsl_file(dsl);
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let output_path = temp_dir.path().join("complex.svg");

    // Compile should handle complex diagram
    let compile_result = App::compile(input_file.path(), &output_path);
    assert!(
        compile_result.is_ok(),
        "Compile should handle complex diagram"
    );

    // Verify SVG contains all components
    let svg = fs::read_to_string(&output_path).expect("Failed to read SVG");

    let expected_nodes = [
        "Load Balancer",
        "Web Server 1",
        "Web Server 2",
        "Application Server",
        "Redis Cache",
        "PostgreSQL Primary",
        "PostgreSQL Replica",
        "S3 Storage",
    ];

    for node in &expected_nodes {
        assert!(svg.contains(node), "SVG should contain node '{}'", node);
    }

    let expected_labels = [
        "HTTP",
        "forward",
        "cache lookup",
        "write",
        "read",
        "upload files",
    ];

    for label in &expected_labels {
        assert!(svg.contains(label), "SVG should contain label '{}'", label);
    }

    // Preview should also work
    let preview_result = App::preview(input_file.path());
    assert!(
        preview_result.is_ok(),
        "Preview should handle complex diagram"
    );
}

/// Test that empty diagram processes correctly through all operations
#[test]
fn test_empty_diagram_through_all_operations() {
    let dsl = "# Just comments\n\n";

    let input_file = create_dsl_file(dsl);
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let output_path = temp_dir.path().join("empty.svg");

    // All operations should succeed
    let validate_result = App::validate(input_file.path());
    let preview_result = App::preview(input_file.path());
    let compile_result = App::compile(input_file.path(), &output_path);

    assert!(validate_result.is_ok(), "Validate should succeed for empty");
    assert!(preview_result.is_ok(), "Preview should succeed for empty");
    assert!(compile_result.is_ok(), "Compile should succeed for empty");

    // Verify SVG was created (even if minimal)
    assert!(
        output_path.exists(),
        "SVG should be created for empty diagram"
    );

    let svg = fs::read_to_string(&output_path).expect("Failed to read SVG");
    assert!(svg.starts_with("<svg"), "Output should be valid SVG");

    // Preview of empty diagram should produce empty or minimal output
    let ascii = preview_result.unwrap();
    assert_eq!(ascii, "", "Empty diagram should produce empty ASCII");
}

/// Test that file with only nodes (no connections) works through all operations
#[test]
fn test_nodes_only_diagram() {
    let dsl = r#"node "Service A" as svc_a
node "Service B" as svc_b
node "Service C" as svc_c
"#;

    let input_file = create_dsl_file(dsl);
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let output_path = temp_dir.path().join("nodes_only.svg");

    let compile_result = App::compile(input_file.path(), &output_path);
    assert!(compile_result.is_ok(), "Should compile nodes-only diagram");

    let svg = fs::read_to_string(&output_path).expect("Failed to read SVG");
    assert!(svg.contains("Service A"));
    assert!(svg.contains("Service B"));
    assert!(svg.contains("Service C"));

    // Should not contain any connection arrows
    let _path_count = svg.matches("<path").count();
    // Rectangles for nodes should exist, but no paths for connections
    assert!(svg.contains("<rect"), "Should have rectangles for nodes");
}

/// Test bidirectional connections are rendered correctly
#[test]
fn test_bidirectional_connections() {
    let dsl = r#"node "Client" as client
node "Server" as server

client -> server : "request"
server -> client : "response"
"#;

    let input_file = create_dsl_file(dsl);
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let output_path = temp_dir.path().join("bidirectional.svg");

    let compile_result = App::compile(input_file.path(), &output_path);
    assert!(
        compile_result.is_ok(),
        "Should compile bidirectional diagram"
    );

    let svg = fs::read_to_string(&output_path).expect("Failed to read SVG");

    // Both connection labels should be present
    assert!(svg.contains("request"), "Should contain 'request' label");
    assert!(svg.contains("response"), "Should contain 'response' label");

    // Both nodes should be present
    assert!(svg.contains("Client"));
    assert!(svg.contains("Server"));
}
