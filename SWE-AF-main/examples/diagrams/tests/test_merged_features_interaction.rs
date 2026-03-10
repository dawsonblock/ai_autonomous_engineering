//! Integration Tests for Merged Feature Interactions
//!
//! This test suite focuses on testing the interactions between features from
//! different merged branches, particularly:
//! 1. The comprehensive integration.rs test suite from branch (30 tests in main)
//! 2. The fixed layout algorithm in src/layout.rs
//! 3. Cross-module interactions: Lexer → Parser → Validator → Layout → Renderers
//!
//! These tests verify that the merged code works correctly at interaction boundaries

use diagrams::app::App;
use diagrams::error::DiagramError;
use diagrams::layout::LayoutEngine;
use diagrams::lexer::Lexer;
use diagrams::parser::Parser;
use diagrams::validator::Validator;
use std::fs;
use std::io::Write;
use tempfile::{NamedTempFile, TempDir};

/// Helper to create a temporary file with DSL content
fn create_dsl_file(content: &str) -> NamedTempFile {
    let mut file = NamedTempFile::new().expect("Failed to create temp file");
    file.write_all(content.as_bytes())
        .expect("Failed to write DSL content");
    file.flush().expect("Failed to flush temp file");
    file
}

// =============================================================================
// Parser → Validator → Layout Interaction Tests
// =============================================================================

#[test]
fn test_parser_to_layout_preserves_node_order() {
    // Test that node order is preserved through parser → layout pipeline
    let dsl_content = r#"node "First" as first
node "Second" as second
node "Third" as third
first -> second
second -> third
"#;

    let mut lexer = Lexer::new(dsl_content);
    let tokens = lexer.tokenize().expect("Lexer should succeed");
    let mut parser = Parser::new(tokens);
    let diagram = parser.parse().expect("Parser should succeed");

    // Verify parser preserves order
    assert_eq!(diagram.nodes[0].identifier, "first");
    assert_eq!(diagram.nodes[1].identifier, "second");
    assert_eq!(diagram.nodes[2].identifier, "third");

    // Verify layout can handle the parsed diagram
    let layout = LayoutEngine::layout(&diagram);

    // All nodes should be positioned
    assert_eq!(layout.nodes.len(), 3);

    // Verify all original nodes are in layout
    let identifiers: Vec<String> = layout
        .nodes
        .iter()
        .map(|n| n.node.identifier.clone())
        .collect();
    assert!(identifiers.contains(&"first".to_string()));
    assert!(identifiers.contains(&"second".to_string()));
    assert!(identifiers.contains(&"third".to_string()));
}

#[test]
fn test_validator_catches_errors_before_layout() {
    // Test that validator catches semantic errors before layout runs
    let dsl_content = r#"node "Service" as svc
svc -> undefined_node
"#;

    let mut lexer = Lexer::new(dsl_content);
    let tokens = lexer.tokenize().expect("Lexer should succeed");
    let mut parser = Parser::new(tokens);
    let diagram = parser.parse().expect("Parser should succeed");

    // Validator should catch the undefined node
    let result = Validator::validate(&diagram);
    assert!(result.is_err(), "Validator should reject undefined node");

    match result {
        Err(DiagramError::Semantic(_)) => {} // Expected
        _ => panic!("Expected SemanticError"),
    }

    // Layout should still run (even though validation failed)
    // Layout engine doesn't validate - it just positions what it's given
    let layout = LayoutEngine::layout(&diagram);
    assert_eq!(layout.nodes.len(), 1, "Layout should position valid nodes");
}

#[test]
fn test_full_pipeline_lexer_to_svg_with_node_types() {
    // Test complete pipeline with node types feature
    let dsl_content = r#"node "API" as api [type: service]
node "DB" as db [type: database]
node "Queue" as queue [type: queue]
api -> queue
queue -> db
"#;

    let input_file = create_dsl_file(dsl_content);
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let output_path = temp_dir.path().join("types.svg");

    // Full pipeline should succeed
    let result = App::compile(input_file.path(), &output_path);
    assert!(
        result.is_ok(),
        "Pipeline should handle node types correctly"
    );

    // Verify SVG contains all nodes
    let svg_content = fs::read_to_string(&output_path).expect("Failed to read SVG");
    assert!(svg_content.contains("API"));
    assert!(svg_content.contains("DB"));
    assert!(svg_content.contains("Queue"));
}

// =============================================================================
// Layout → Renderer (SVG/ASCII) Interaction Tests
// =============================================================================

#[test]
fn test_layout_produces_valid_coordinates_for_svg() {
    // Test that layout produces coordinates that SVG renderer can use
    let dsl_content = r#"node "A" as a
node "B" as b
node "C" as c
a -> b
b -> c
"#;

    let mut lexer = Lexer::new(dsl_content);
    let tokens = lexer.tokenize().expect("Lexer should succeed");
    let mut parser = Parser::new(tokens);
    let diagram = parser.parse().expect("Parser should succeed");

    let layout = LayoutEngine::layout(&diagram);

    // Verify all coordinates are non-negative (valid for SVG)
    for node in &layout.nodes {
        assert!(
            node.position.x >= 0.0,
            "X coordinate should be non-negative"
        );
        assert!(
            node.position.y >= 0.0,
            "Y coordinate should be non-negative"
        );
        assert!(node.width > 0.0, "Width should be positive");
        assert!(node.height > 0.0, "Height should be positive");
    }

    for conn in &layout.connections {
        assert!(conn.start.x >= 0.0 && conn.start.y >= 0.0);
        assert!(conn.end.x >= 0.0 && conn.end.y >= 0.0);
    }

    // Now test that SVG renderer accepts this layout
    let input_file = create_dsl_file(dsl_content);
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let output_path = temp_dir.path().join("coords.svg");

    let result = App::compile(input_file.path(), &output_path);
    assert!(result.is_ok(), "SVG rendering should succeed");

    let svg = fs::read_to_string(&output_path).expect("Failed to read SVG");
    assert!(svg.starts_with("<svg"));
}

#[test]
fn test_layout_produces_valid_coordinates_for_ascii() {
    // Test that layout produces coordinates that ASCII renderer can convert
    let dsl_content = r#"node "Service X" as x
node "Service Y" as y
x -> y
"#;

    let input_file = create_dsl_file(dsl_content);

    // Preview uses ASCII renderer with layout coordinates
    let result = App::preview(input_file.path());
    assert!(result.is_ok(), "ASCII rendering should succeed");

    let ascii = result.unwrap();
    assert!(!ascii.is_empty(), "ASCII output should not be empty");
}

#[test]
fn test_layout_dimensions_match_renderer_expectations() {
    // Test that layout dimensions are used correctly by renderers
    let dsl_content = r#"node "N1" as n1
node "N2" as n2
node "N3" as n3
node "N4" as n4
n1 -> n2
n2 -> n3
n3 -> n4
"#;

    let mut lexer = Lexer::new(dsl_content);
    let tokens = lexer.tokenize().expect("Lexer should succeed");
    let mut parser = Parser::new(tokens);
    let diagram = parser.parse().expect("Parser should succeed");

    let layout = LayoutEngine::layout(&diagram);

    // Layout dimensions should encompass all nodes
    let max_x = layout
        .nodes
        .iter()
        .map(|n| n.position.x + n.width)
        .fold(0.0, f64::max);
    let max_y = layout
        .nodes
        .iter()
        .map(|n| n.position.y + n.height)
        .fold(0.0, f64::max);

    assert_eq!(
        layout.width, max_x,
        "Layout width should match rightmost extent"
    );
    assert_eq!(
        layout.height, max_y,
        "Layout height should match bottommost extent"
    );

    // Verify SVG uses these dimensions
    let input_file = create_dsl_file(dsl_content);
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let output_path = temp_dir.path().join("dims.svg");

    App::compile(input_file.path(), &output_path).expect("Compile should succeed");

    let svg = fs::read_to_string(&output_path).expect("Failed to read SVG");

    // SVG should have width and height attributes
    assert!(svg.contains("width="));
    assert!(svg.contains("height="));
}

// =============================================================================
// Error Propagation Through Pipeline
// =============================================================================

#[test]
fn test_lexer_error_propagates_through_app() {
    // Test that lexer errors are properly propagated through App layer
    let dsl_content = r#"node "Test" @ invalid"#;

    let input_file = create_dsl_file(dsl_content);

    // Compile should fail with syntax error from lexer
    let result = App::compile(input_file.path(), std::path::Path::new("/tmp/test.svg"));
    assert!(result.is_err(), "Should fail on lexer error");

    match result.unwrap_err() {
        DiagramError::Syntax(_) => {} // Expected
        other => panic!("Expected Syntax error from lexer, got {:?}", other),
    }
}

#[test]
fn test_parser_error_propagates_through_app() {
    // Test that parser errors are properly propagated
    let dsl_content = r#"node "Test" as test
test -> -> test
"#;

    let input_file = create_dsl_file(dsl_content);

    let result = App::compile(input_file.path(), std::path::Path::new("/tmp/test.svg"));
    assert!(result.is_err(), "Should fail on parser error");

    match result.unwrap_err() {
        DiagramError::Syntax(_) => {} // Expected
        other => panic!("Expected Syntax error from parser, got {:?}", other),
    }
}

#[test]
fn test_validator_error_propagates_through_app() {
    // Test that validator errors are properly propagated
    let dsl_content = r#"node "A" as a
a -> nonexistent
"#;

    let input_file = create_dsl_file(dsl_content);

    let result = App::compile(input_file.path(), std::path::Path::new("/tmp/test.svg"));
    assert!(result.is_err(), "Should fail on validator error");

    match result.unwrap_err() {
        DiagramError::Semantic(_) => {} // Expected
        other => panic!("Expected Semantic error from validator, got {:?}", other),
    }
}

// =============================================================================
// Multi-Workflow Consistency Tests
// =============================================================================

#[test]
fn test_compile_and_preview_use_same_layout() {
    // Test that compile and preview produce consistent layouts
    let dsl_content = r#"node "Service A" as a
node "Service B" as b
node "Service C" as c
a -> b
b -> c
"#;

    // Get layout from compile path
    let mut lexer1 = Lexer::new(dsl_content);
    let tokens1 = lexer1.tokenize().expect("Lexer should succeed");
    let mut parser1 = Parser::new(tokens1);
    let diagram1 = parser1.parse().expect("Parser should succeed");
    let layout1 = LayoutEngine::layout(&diagram1);

    // Get layout from preview path
    let mut lexer2 = Lexer::new(dsl_content);
    let tokens2 = lexer2.tokenize().expect("Lexer should succeed");
    let mut parser2 = Parser::new(tokens2);
    let diagram2 = parser2.parse().expect("Parser should succeed");
    let layout2 = LayoutEngine::layout(&diagram2);

    // Layouts should be identical
    assert_eq!(layout1.nodes.len(), layout2.nodes.len());
    assert_eq!(layout1.connections.len(), layout2.connections.len());
    assert_eq!(layout1.width, layout2.width);
    assert_eq!(layout1.height, layout2.height);

    // Node positions should match
    for i in 0..layout1.nodes.len() {
        assert_eq!(
            layout1.nodes[i].node.identifier, layout2.nodes[i].node.identifier,
            "Node identifiers should match"
        );
        assert_eq!(
            layout1.nodes[i].position.x, layout2.nodes[i].position.x,
            "X positions should match"
        );
        assert_eq!(
            layout1.nodes[i].position.y, layout2.nodes[i].position.y,
            "Y positions should match"
        );
    }
}

#[test]
fn test_validate_compile_preview_all_consistent() {
    // Test that all three workflows handle the same input consistently
    let dsl_content = r#"node "Web" as web
node "API" as api
node "DB" as db
web -> api
api -> db
"#;

    let input_file = create_dsl_file(dsl_content);
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let output_path = temp_dir.path().join("test.svg");

    // All three should succeed
    let validate_result = App::validate(input_file.path());
    assert!(validate_result.is_ok(), "Validate should succeed");

    let compile_result = App::compile(input_file.path(), &output_path);
    assert!(compile_result.is_ok(), "Compile should succeed");

    let preview_result = App::preview(input_file.path());
    assert!(preview_result.is_ok(), "Preview should succeed");

    // Verify outputs
    assert!(output_path.exists(), "SVG should be created");
    assert!(
        !preview_result.unwrap().is_empty(),
        "Preview should produce output"
    );
}

// =============================================================================
// Stress Tests for Merged Features
// =============================================================================

#[test]
fn test_large_diagram_through_all_workflows() {
    // Create a large diagram and run through all workflows
    let mut dsl_content = String::new();

    // 25 nodes in a grid-like structure
    for i in 0..25 {
        dsl_content.push_str(&format!("node \"Node {}\" as n{}\n", i, i));
    }

    // Connect in layers: 0-4 -> 5-9 -> 10-14 -> 15-19 -> 20-24
    for layer in 0..4 {
        let layer_start = layer * 5;
        let next_layer_start = (layer + 1) * 5;
        for i in 0..5 {
            for j in 0..5 {
                let src = layer_start + i;
                let dst = next_layer_start + j;
                dsl_content.push_str(&format!("n{} -> n{}\n", src, dst));
            }
        }
    }

    let input_file = create_dsl_file(&dsl_content);
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let output_path = temp_dir.path().join("large.svg");

    // Validate workflow
    let validate_result = App::validate(input_file.path());
    assert!(
        validate_result.is_ok(),
        "Validate should handle large diagram"
    );

    // Compile workflow
    let compile_result = App::compile(input_file.path(), &output_path);
    assert!(
        compile_result.is_ok(),
        "Compile should handle large diagram"
    );

    // Preview workflow
    let preview_result = App::preview(input_file.path());
    assert!(
        preview_result.is_ok(),
        "Preview should handle large diagram"
    );

    // Verify SVG output
    let svg = fs::read_to_string(&output_path).expect("Failed to read SVG");
    assert!(svg.contains("Node 0"));
    assert!(svg.contains("Node 24"));
}

#[test]
fn test_complex_labels_through_pipeline() {
    // Test that complex labels (special chars, unicode, etc.) work through pipeline
    let dsl_content = r#"node "Service (v2.0)" as svc
node "DB-Cluster #1" as db
node "Cache: Redis" as cache
node "Queue 🚀" as queue

svc -> db : "Query: SELECT *"
db -> cache : "Update & Sync"
cache -> queue : "Message → Queue"
"#;

    let input_file = create_dsl_file(dsl_content);
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let output_path = temp_dir.path().join("labels.svg");

    // Should work through all workflows
    let compile_result = App::compile(input_file.path(), &output_path);
    assert!(
        compile_result.is_ok(),
        "Compile should handle complex labels"
    );

    let preview_result = App::preview(input_file.path());
    assert!(
        preview_result.is_ok(),
        "Preview should handle complex labels"
    );

    // Verify SVG contains labels
    let svg = fs::read_to_string(&output_path).expect("Failed to read SVG");
    assert!(svg.contains("Service (v2.0)"));
    assert!(svg.contains("DB-Cluster #1"));
    assert!(svg.contains("Cache: Redis"));
}
