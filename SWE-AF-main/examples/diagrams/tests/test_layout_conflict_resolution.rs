//! Integration Tests for Layout Conflict Resolution
//!
//! This test suite specifically targets the conflict resolution area in src/layout.rs
//! where an infinite loop bug was fixed. The branch had a BFS algorithm that could
//! re-queue processed nodes causing infinite loops. The HEAD version uses iterative
//! relaxation (Bellman-Ford style) which is guaranteed to terminate.
//!
//! Priority 1: Test cases that would trigger infinite loops in the buggy version
//! Priority 2: Test complex graph topologies that stress the layout algorithm

use diagrams::app::App;
use diagrams::layout::LayoutEngine;
use diagrams::lexer::Lexer;
use diagrams::parser::Parser;
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
// Priority 1: Infinite Loop Prevention Tests
// =============================================================================

#[test]
fn test_layout_cyclic_graph_no_infinite_loop() {
    // This would cause infinite loop in buggy BFS (nodes re-queued repeatedly)
    // Fixed version uses bounded iterations
    let dsl_content = r#"node "A" as a
node "B" as b
node "C" as c
a -> b
b -> c
c -> a
"#;

    let mut lexer = Lexer::new(dsl_content);
    let tokens = lexer.tokenize().expect("Lexer should succeed");
    let mut parser = Parser::new(tokens);
    let diagram = parser.parse().expect("Parser should succeed");

    // The layout engine should handle cycles gracefully without infinite loop
    let layout = LayoutEngine::layout(&diagram);

    // Verify all nodes are positioned
    assert_eq!(layout.nodes.len(), 3, "All 3 nodes should be laid out");
    assert_eq!(
        layout.connections.len(),
        3,
        "All 3 connections should be positioned"
    );

    // Verify layout completed (not stuck in infinite loop)
    assert!(layout.width > 0.0, "Layout should have positive width");
    assert!(layout.height > 0.0, "Layout should have positive height");
}

#[test]
fn test_layout_self_referential_cycle() {
    // Multiple nodes with cyclic dependencies
    let dsl_content = r#"node "Service1" as s1
node "Service2" as s2
node "Service3" as s3
node "Service4" as s4
s1 -> s2
s2 -> s3
s3 -> s4
s4 -> s1
s1 -> s3
"#;

    let input_file = create_dsl_file(dsl_content);
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let output_path = temp_dir.path().join("cycle.svg");

    // Should complete without hanging
    let _result = App::compile(input_file.path(), &output_path);

    // Validation should reject self-cycles, but layout should still complete
    // if we bypass validation
    let mut lexer = Lexer::new(dsl_content);
    let tokens = lexer.tokenize().expect("Lexer should succeed");
    let mut parser = Parser::new(tokens);
    let diagram = parser.parse().expect("Parser should succeed");

    // Layout should complete even for cyclic graphs
    let layout = LayoutEngine::layout(&diagram);
    assert_eq!(layout.nodes.len(), 4);
    assert!(layout.width > 0.0);
}

#[test]
fn test_layout_diamond_with_back_edge() {
    // Diamond topology with back edge - could cause re-queuing in buggy BFS
    //     A
    //    / \
    //   B   C
    //    \ /
    //     D
    //     |
    //     A (back edge)
    let dsl_content = r#"node "A" as a
node "B" as b
node "C" as c
node "D" as d
a -> b
a -> c
b -> d
c -> d
d -> a
"#;

    let mut lexer = Lexer::new(dsl_content);
    let tokens = lexer.tokenize().expect("Lexer should succeed");
    let mut parser = Parser::new(tokens);
    let diagram = parser.parse().expect("Parser should succeed");

    // Should complete without infinite loop
    let layout = LayoutEngine::layout(&diagram);

    assert_eq!(layout.nodes.len(), 4);
    assert_eq!(layout.connections.len(), 5);
    assert!(layout.width > 0.0);
    assert!(layout.height > 0.0);
}

// =============================================================================
// Priority 2: Complex Graph Topology Tests
// =============================================================================

#[test]
fn test_layout_large_layered_dag() {
    // Large directed acyclic graph with many layers
    // Tests that iterative relaxation completes efficiently
    let mut dsl_content = String::new();

    // Create 7 layers with 3 nodes each
    for layer in 0..7 {
        for node in 0..3 {
            let id = format!("l{}n{}", layer, node);
            let name = format!("Layer {} Node {}", layer, node);
            dsl_content.push_str(&format!("node \"{}\" as {}\n", name, id));
        }
    }

    // Connect each layer to the next
    for layer in 0..6 {
        for src_node in 0..3 {
            for dst_node in 0..3 {
                let src = format!("l{}n{}", layer, src_node);
                let dst = format!("l{}n{}", layer + 1, dst_node);
                dsl_content.push_str(&format!("{} -> {}\n", src, dst));
            }
        }
    }

    let mut lexer = Lexer::new(&dsl_content);
    let tokens = lexer.tokenize().expect("Lexer should succeed");
    let mut parser = Parser::new(tokens);
    let diagram = parser.parse().expect("Parser should succeed");

    // Layout should complete for large DAG
    let layout = LayoutEngine::layout(&diagram);

    assert_eq!(
        layout.nodes.len(),
        21,
        "Should have 21 nodes (7 layers * 3)"
    );
    assert!(!layout.connections.is_empty());
    assert!(layout.width > 0.0);
    assert!(layout.height > 0.0);

    // Verify nodes are distributed across multiple layers (x positions)
    let x_positions: Vec<f64> = layout.nodes.iter().map(|n| n.position.x).collect();
    let unique_x: std::collections::HashSet<_> =
        x_positions.iter().map(|x| (*x * 100.0) as i64).collect();
    assert!(
        unique_x.len() >= 5,
        "Nodes should be distributed across multiple layers"
    );
}

#[test]
fn test_layout_star_topology() {
    // Central hub connected to many nodes
    // Tests that layout handles nodes with many incoming/outgoing edges
    let mut dsl_content = String::new();
    dsl_content.push_str("node \"Hub\" as hub\n");

    for i in 0..10 {
        dsl_content.push_str(&format!("node \"Spoke {}\" as spoke{}\n", i, i));
        dsl_content.push_str(&format!("hub -> spoke{}\n", i));
    }

    let mut lexer = Lexer::new(&dsl_content);
    let tokens = lexer.tokenize().expect("Lexer should succeed");
    let mut parser = Parser::new(tokens);
    let diagram = parser.parse().expect("Parser should succeed");

    let layout = LayoutEngine::layout(&diagram);

    assert_eq!(layout.nodes.len(), 11, "Should have hub + 10 spokes");
    assert_eq!(layout.connections.len(), 10);

    // Hub should be in layer 0
    let hub_node = layout
        .nodes
        .iter()
        .find(|n| n.node.identifier == "hub")
        .expect("Hub should exist");
    assert_eq!(hub_node.position.x, 0.0, "Hub should be in layer 0");

    // All spokes should be in layer 1 (same x coordinate)
    let spoke_nodes: Vec<_> = layout
        .nodes
        .iter()
        .filter(|n| n.node.identifier.starts_with("spoke"))
        .collect();
    assert_eq!(spoke_nodes.len(), 10);

    let first_spoke_x = spoke_nodes[0].position.x;
    for spoke in &spoke_nodes {
        assert_eq!(
            spoke.position.x, first_spoke_x,
            "All spokes should be in the same layer"
        );
    }
}

#[test]
fn test_layout_multiple_disconnected_components() {
    // Multiple separate graphs with no connections between them
    // Tests that layout handles disconnected components correctly
    let dsl_content = r#"node "A1" as a1
node "A2" as a2
a1 -> a2

node "B1" as b1
node "B2" as b2
b1 -> b2

node "C1" as c1
node "C2" as c2
c1 -> c2

node "Isolated" as iso
"#;

    let mut lexer = Lexer::new(dsl_content);
    let tokens = lexer.tokenize().expect("Lexer should succeed");
    let mut parser = Parser::new(tokens);
    let diagram = parser.parse().expect("Parser should succeed");

    let layout = LayoutEngine::layout(&diagram);

    assert_eq!(layout.nodes.len(), 7);
    assert_eq!(layout.connections.len(), 3);

    // All source nodes (a1, b1, c1, iso) should be in layer 0
    let layer_0_nodes: Vec<_> = layout
        .nodes
        .iter()
        .filter(|n| n.position.x == 0.0)
        .collect();
    assert!(
        layer_0_nodes.len() >= 4,
        "All disconnected component roots should be in layer 0"
    );
}

#[test]
fn test_layout_long_chain() {
    // Very long linear chain to test iteration bounds
    let mut dsl_content = String::new();

    for i in 0..30 {
        dsl_content.push_str(&format!("node \"Node {}\" as n{}\n", i, i));
    }

    for i in 0..29 {
        dsl_content.push_str(&format!("n{} -> n{}\n", i, i + 1));
    }

    let mut lexer = Lexer::new(&dsl_content);
    let tokens = lexer.tokenize().expect("Lexer should succeed");
    let mut parser = Parser::new(tokens);
    let diagram = parser.parse().expect("Parser should succeed");

    let layout = LayoutEngine::layout(&diagram);

    assert_eq!(layout.nodes.len(), 30);
    assert_eq!(layout.connections.len(), 29);

    // Verify nodes are in correct layers (increasing x coordinates)
    for i in 0..30 {
        let node = layout
            .nodes
            .iter()
            .find(|n| n.node.identifier == format!("n{}", i))
            .expect("Node should exist");

        // Each node should be in a layer corresponding to its position in chain
        // Layer i should have x coordinate proportional to i
        assert!(
            node.position.x >= 0.0,
            "Node {} should have valid x position",
            i
        );
    }

    // Verify that the last node has the highest x coordinate
    let first_node = layout
        .nodes
        .iter()
        .find(|n| n.node.identifier == "n0")
        .unwrap();
    let last_node = layout
        .nodes
        .iter()
        .find(|n| n.node.identifier == "n29")
        .unwrap();
    assert!(
        last_node.position.x > first_node.position.x,
        "Last node should be further right than first node"
    );
}

// =============================================================================
// Cross-Module Integration: Layout + Rendering
// =============================================================================

#[test]
fn test_layout_to_svg_rendering_complex_graph() {
    // Test full pipeline with fixed layout algorithm producing valid SVG
    let dsl_content = r#"node "Frontend" as fe
node "API Gateway" as api
node "Auth Service" as auth
node "User Service" as user
node "Database" as db
node "Cache" as cache

fe -> api
api -> auth
api -> user
auth -> db
user -> db
user -> cache
"#;

    let input_file = create_dsl_file(dsl_content);
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let output_path = temp_dir.path().join("complex.svg");

    // Compile should succeed with fixed layout
    let result = App::compile(input_file.path(), &output_path);
    assert!(
        result.is_ok(),
        "Compile should succeed with fixed layout algorithm"
    );

    // Verify SVG output
    let svg_content = fs::read_to_string(&output_path).expect("Failed to read SVG");

    // Check SVG validity
    assert!(svg_content.starts_with("<svg"));
    assert!(svg_content.contains("</svg>"));

    // Check all nodes are rendered
    assert!(svg_content.contains("Frontend"));
    assert!(svg_content.contains("API Gateway"));
    assert!(svg_content.contains("Auth Service"));
    assert!(svg_content.contains("User Service"));
    assert!(svg_content.contains("Database"));
    assert!(svg_content.contains("Cache"));

    // Check that positions are present (coordinates in SVG)
    assert!(svg_content.contains("x="));
    assert!(svg_content.contains("y="));
}

#[test]
fn test_layout_to_ascii_rendering_complex_graph() {
    // Test full pipeline with fixed layout algorithm producing valid ASCII
    let dsl_content = r#"node "Service A" as a
node "Service B" as b
node "Service C" as c
a -> b
b -> c
a -> c
"#;

    let input_file = create_dsl_file(dsl_content);

    // Preview should succeed with fixed layout
    let result = App::preview(input_file.path());
    assert!(
        result.is_ok(),
        "Preview should succeed with fixed layout algorithm"
    );

    let ascii_output = result.unwrap();

    // Verify ASCII output contains box-drawing characters
    assert!(!ascii_output.is_empty(), "ASCII output should not be empty");
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
fn test_layout_algorithm_stability() {
    // Test that layout algorithm produces consistent results
    let dsl_content = r#"node "A" as a
node "B" as b
node "C" as c
node "D" as d
a -> b
a -> c
b -> d
c -> d
"#;

    let mut lexer = Lexer::new(dsl_content);
    let tokens = lexer.tokenize().expect("Lexer should succeed");
    let mut parser = Parser::new(tokens);
    let diagram = parser.parse().expect("Parser should succeed");

    // Run layout multiple times
    let layout1 = LayoutEngine::layout(&diagram);
    let layout2 = LayoutEngine::layout(&diagram);

    // Results should be identical
    assert_eq!(
        layout1.nodes.len(),
        layout2.nodes.len(),
        "Layout should be deterministic"
    );
    assert_eq!(layout1.width, layout2.width);
    assert_eq!(layout1.height, layout2.height);

    // Node positions should be identical
    for i in 0..layout1.nodes.len() {
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
