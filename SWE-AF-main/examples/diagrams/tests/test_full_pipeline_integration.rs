//! Integration tests for full pipeline interactions
//!
//! These tests verify the end-to-end integration of all components:
//! Lexer → Parser → Validator → Layout → Renderers (SVG & ASCII)
//!
//! PRIORITY: MEDIUM - Tests cross-feature interactions after merge

use diagrams::app::App;
use diagrams::ascii::AsciiRenderer;
use diagrams::layout::LayoutEngine;
use diagrams::lexer::Lexer;
use diagrams::parser::Parser;
use diagrams::svg::SvgRenderer;
use diagrams::validator::Validator;
use std::fs;
use std::io::Write;
use tempfile::{NamedTempFile, TempDir};

// Helper to create a temporary DSL file
fn create_dsl_file(content: &str) -> NamedTempFile {
    let mut file = NamedTempFile::new().expect("Failed to create temp file");
    file.write_all(content.as_bytes())
        .expect("Failed to write DSL content");
    file.flush().expect("Failed to flush temp file");
    file
}

#[test]
fn test_lexer_parser_integration() {
    // Test that Lexer output is correctly consumed by Parser
    let dsl = r#"node "API Gateway" as api
node "Database" as db
api -> db : "query"
"#;

    // Lexer phase
    let mut lexer = Lexer::new(dsl);
    let tokens = lexer.tokenize().expect("Lexer should succeed");

    // Verify tokens are non-empty
    assert!(!tokens.is_empty(), "Lexer should produce tokens");

    // Parser phase consuming lexer output
    let mut parser = Parser::new(tokens);
    let diagram = parser.parse().expect("Parser should succeed");

    // Verify parsed structure
    assert_eq!(diagram.nodes.len(), 2);
    assert_eq!(diagram.connections.len(), 1);
    assert_eq!(diagram.nodes[0].identifier, "api");
    assert_eq!(diagram.nodes[1].identifier, "db");
    assert_eq!(diagram.connections[0].from, "api");
    assert_eq!(diagram.connections[0].to, "db");
}

#[test]
fn test_parser_validator_integration() {
    // Test that Parser output is correctly validated by Validator
    let dsl = r#"node "Service A" as svc_a
node "Service B" as svc_b
svc_a -> svc_b
"#;

    let mut lexer = Lexer::new(dsl);
    let tokens = lexer.tokenize().expect("Lexer should succeed");

    let mut parser = Parser::new(tokens);
    let diagram = parser.parse().expect("Parser should succeed");

    // Validator phase consuming parser output
    let result = Validator::validate(&diagram);
    assert!(result.is_ok(), "Validator should accept valid diagram");
}

#[test]
fn test_validator_layout_integration() {
    // Test that validated diagrams are correctly processed by Layout
    let dsl = r#"node "Web" as web
node "App" as app
node "DB" as db
web -> app
app -> db
"#;

    let mut lexer = Lexer::new(dsl);
    let tokens = lexer.tokenize().expect("Lexer should succeed");
    let mut parser = Parser::new(tokens);
    let diagram = parser.parse().expect("Parser should succeed");

    // Validate
    Validator::validate(&diagram).expect("Validator should succeed");

    // Layout phase consuming validated diagram
    let layout = LayoutEngine::layout(&diagram);

    // Verify layout output
    assert_eq!(layout.nodes.len(), 3);
    assert_eq!(layout.connections.len(), 2);
    assert!(layout.width > 0.0);
    assert!(layout.height > 0.0);
}

#[test]
fn test_layout_svg_renderer_integration() {
    // Test that Layout output is correctly rendered by SvgRenderer
    let dsl = r#"node "API" as api
node "Cache" as cache
api -> cache : "read"
"#;

    let mut lexer = Lexer::new(dsl);
    let tokens = lexer.tokenize().expect("Lexer should succeed");
    let mut parser = Parser::new(tokens);
    let diagram = parser.parse().expect("Parser should succeed");
    Validator::validate(&diagram).expect("Validator should succeed");

    // Layout
    let layout = LayoutEngine::layout(&diagram);

    // SVG Renderer consuming layout
    let svg = SvgRenderer::render(&layout);

    // Verify SVG output structure
    assert!(svg.starts_with("<svg"), "Should start with SVG tag");
    assert!(svg.contains("</svg>"), "Should end with closing SVG tag");
    assert!(svg.contains("API"), "Should contain node label 'API'");
    assert!(svg.contains("Cache"), "Should contain node label 'Cache'");
    assert!(
        svg.contains("read"),
        "Should contain connection label 'read'"
    );
}

#[test]
fn test_layout_ascii_renderer_integration() {
    // Test that Layout output is correctly rendered by AsciiRenderer
    let dsl = r#"node "A" as a
node "B" as b
a -> b
"#;

    let mut lexer = Lexer::new(dsl);
    let tokens = lexer.tokenize().expect("Lexer should succeed");
    let mut parser = Parser::new(tokens);
    let diagram = parser.parse().expect("Parser should succeed");
    Validator::validate(&diagram).expect("Validator should succeed");

    // Layout
    let layout = LayoutEngine::layout(&diagram);

    // ASCII Renderer consuming layout
    let ascii = AsciiRenderer::render(&layout);

    // Verify ASCII output contains box-drawing characters
    let has_box_chars = ascii.contains('┌')
        || ascii.contains('─')
        || ascii.contains('│')
        || ascii.contains('┐')
        || ascii.contains('└')
        || ascii.contains('┘');

    assert!(has_box_chars, "ASCII output should contain box characters");
}

#[test]
fn test_app_compile_full_pipeline() {
    // Test App::compile orchestrating the full pipeline
    let dsl = r#"node "Frontend" as fe
node "Backend" as be
node "Database" as db
fe -> be : "HTTP"
be -> db : "SQL"
"#;

    let input_file = create_dsl_file(dsl);
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let output_path = temp_dir.path().join("output.svg");

    // App::compile orchestrates: read → lex → parse → validate → layout → render → write
    let result = App::compile(input_file.path(), &output_path);

    assert!(result.is_ok(), "App::compile should succeed");
    assert!(output_path.exists(), "Output file should be created");

    let svg_content = fs::read_to_string(&output_path).expect("Failed to read SVG");
    assert!(svg_content.starts_with("<svg"));
    assert!(svg_content.contains("Frontend"));
    assert!(svg_content.contains("Backend"));
    assert!(svg_content.contains("Database"));
    assert!(svg_content.contains("HTTP"));
    assert!(svg_content.contains("SQL"));
}

#[test]
fn test_app_preview_full_pipeline() {
    // Test App::preview orchestrating: read → lex → parse → validate → layout → ASCII
    let dsl = r#"node "Service" as svc
node "DB" as db
svc -> db
"#;

    let input_file = create_dsl_file(dsl);

    let result = App::preview(input_file.path());

    assert!(result.is_ok(), "App::preview should succeed");

    let ascii = result.unwrap();
    assert!(!ascii.is_empty(), "ASCII output should not be empty");
}

#[test]
fn test_app_validate_full_pipeline() {
    // Test App::validate orchestrating: read → lex → parse → validate
    let dsl = r#"node "A" as a
node "B" as b
a -> b
"#;

    let input_file = create_dsl_file(dsl);

    let result = App::validate(input_file.path());

    assert!(result.is_ok(), "App::validate should succeed");
}

#[test]
fn test_error_propagation_through_pipeline() {
    // Test that errors correctly propagate through the pipeline
    // Syntax error should be caught early (lexer/parser)
    let invalid_dsl = "this is not valid DSL syntax";

    let input_file = create_dsl_file(invalid_dsl);
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let output_path = temp_dir.path().join("output.svg");

    let result = App::compile(input_file.path(), &output_path);

    assert!(result.is_err(), "Should fail on invalid syntax");
    assert!(
        !output_path.exists(),
        "No output file should be created on error"
    );
}

#[test]
fn test_semantic_error_propagation() {
    // Test that semantic errors (undefined node reference) propagate correctly
    let dsl = r#"node "A" as a
a -> undefined_node
"#;

    let input_file = create_dsl_file(dsl);
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let output_path = temp_dir.path().join("output.svg");

    let result = App::compile(input_file.path(), &output_path);

    assert!(result.is_err(), "Should fail on undefined node reference");
}

#[test]
fn test_pipeline_consistency_across_workflows() {
    // Verify that compile, preview, and validate all work consistently
    let dsl = r#"node "Web" as web
node "API" as api
web -> api : "REST"
"#;

    let input_file = create_dsl_file(dsl);
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let output_path = temp_dir.path().join("output.svg");

    // All three workflows should succeed for the same valid input
    assert!(
        App::validate(input_file.path()).is_ok(),
        "Validate should succeed"
    );
    assert!(
        App::preview(input_file.path()).is_ok(),
        "Preview should succeed"
    );
    assert!(
        App::compile(input_file.path(), &output_path).is_ok(),
        "Compile should succeed"
    );

    assert!(output_path.exists(), "SVG should be created");
}

#[test]
fn test_complex_topology_through_full_pipeline() {
    // Test complex graph topology through entire pipeline
    let dsl = r#"node "LB" as lb
node "Web1" as web1
node "Web2" as web2
node "App" as app
node "Cache" as cache
node "DB" as db

lb -> web1
lb -> web2
web1 -> app
web2 -> app
app -> cache
app -> db
"#;

    let input_file = create_dsl_file(dsl);
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let output_path = temp_dir.path().join("complex.svg");

    let result = App::compile(input_file.path(), &output_path);

    assert!(result.is_ok(), "Should handle complex topology");

    let svg = fs::read_to_string(&output_path).expect("Failed to read SVG");
    assert!(svg.contains("LB"));
    assert!(svg.contains("Web1"));
    assert!(svg.contains("Web2"));
    assert!(svg.contains("App"));
    assert!(svg.contains("Cache"));
    assert!(svg.contains("DB"));
}

#[test]
fn test_unicode_through_pipeline() {
    // Test Unicode handling through full pipeline
    let dsl = r#"node "データベース" as db
node "API サーバー" as api
api -> db : "クエリ"
"#;

    let input_file = create_dsl_file(dsl);
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let output_path = temp_dir.path().join("unicode.svg");

    let result = App::compile(input_file.path(), &output_path);

    assert!(result.is_ok(), "Should handle Unicode");

    let svg = fs::read_to_string(&output_path).expect("Failed to read SVG");
    assert!(svg.contains("データベース"));
    assert!(svg.contains("API サーバー"));
    assert!(svg.contains("クエリ"));
}

#[test]
fn test_empty_diagram_through_pipeline() {
    // Test empty diagram through all pipeline stages
    let dsl = "# Just a comment\n\n";

    let input_file = create_dsl_file(dsl);
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let output_path = temp_dir.path().join("empty.svg");

    let result = App::compile(input_file.path(), &output_path);

    assert!(result.is_ok(), "Should handle empty diagram");
    assert!(output_path.exists());

    let svg = fs::read_to_string(&output_path).expect("Failed to read SVG");
    assert!(svg.starts_with("<svg"));
}

#[test]
fn test_large_diagram_pipeline_performance() {
    // Test performance with large diagram through full pipeline
    let mut dsl = String::new();

    // Create 50 nodes
    for i in 0..50 {
        dsl.push_str(&format!(r#"node "Node {}" as n{}"#, i, i));
        dsl.push('\n');
    }

    // Create connections
    for i in 0..49 {
        dsl.push_str(&format!("n{} -> n{}\n", i, i + 1));
    }

    let input_file = create_dsl_file(&dsl);
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let output_path = temp_dir.path().join("large.svg");

    let start = std::time::Instant::now();
    let result = App::compile(input_file.path(), &output_path);
    let duration = start.elapsed();

    assert!(result.is_ok(), "Should handle large diagram");
    assert!(
        duration < std::time::Duration::from_secs(5),
        "Should complete in reasonable time (took {:?})",
        duration
    );

    let svg = fs::read_to_string(&output_path).expect("Failed to read SVG");
    assert!(svg.contains("Node 0"));
    assert!(svg.contains("Node 49"));
}
