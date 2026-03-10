//! Integration tests for the fixed layout algorithm
//!
//! These tests specifically verify that the layout algorithm fix (replacing BFS
//! with iterative relaxation) correctly handles complex graph topologies without
//! infinite loops or incorrect layer assignments.
//!
//! PRIORITY: HIGH - Tests conflict resolution in src/layout.rs

use diagrams::layout::LayoutEngine;
use diagrams::types::{Connection, Diagram, Node, NodeType, SourcePosition};
use std::time::{Duration, Instant};

// Helper to create test nodes
fn create_node(id: &str, display_name: &str) -> Node {
    Node {
        identifier: id.to_string(),
        display_name: display_name.to_string(),
        node_type: NodeType::Service,
        position: SourcePosition { line: 1, column: 1 },
    }
}

// Helper to create test connections
fn create_connection(from: &str, to: &str, label: Option<&str>) -> Connection {
    Connection {
        from: from.to_string(),
        to: to.to_string(),
        label: label.map(|s| s.to_string()),
        position: SourcePosition { line: 1, column: 1 },
    }
}

#[test]
fn test_layout_no_infinite_loop_on_complex_graph() {
    // This test verifies the fix for the infinite loop bug
    // The old BFS algorithm could re-queue already processed nodes
    // The new iterative relaxation algorithm should handle this correctly
    let diagram = Diagram {
        nodes: vec![
            create_node("a", "Node A"),
            create_node("b", "Node B"),
            create_node("c", "Node C"),
            create_node("d", "Node D"),
            create_node("e", "Node E"),
        ],
        connections: vec![
            create_connection("a", "b", Some("1")),
            create_connection("a", "c", Some("2")),
            create_connection("b", "d", Some("3")),
            create_connection("c", "d", Some("4")),
            create_connection("d", "e", Some("5")),
            create_connection("b", "e", Some("6")), // Multiple paths to e
        ],
    };

    let start = Instant::now();
    let layout = LayoutEngine::layout(&diagram);
    let duration = start.elapsed();

    // Should complete in under 1 second for this size
    assert!(
        duration < Duration::from_secs(1),
        "Layout should complete quickly without infinite loops (took {:?})",
        duration
    );

    // Verify all nodes are positioned
    assert_eq!(layout.nodes.len(), 5);
    assert_eq!(layout.connections.len(), 6);

    // Verify nodes have valid positions
    for positioned_node in &layout.nodes {
        assert!(
            positioned_node.position.x >= 0.0,
            "Node {} should have non-negative x position",
            positioned_node.node.identifier
        );
        assert!(
            positioned_node.position.y >= 0.0,
            "Node {} should have non-negative y position",
            positioned_node.node.identifier
        );
    }
}

#[test]
fn test_layout_diamond_topology() {
    // Diamond topology tests multiple paths of same length
    //     a
    //    / \
    //   b   c
    //    \ /
    //     d
    let diagram = Diagram {
        nodes: vec![
            create_node("a", "Top"),
            create_node("b", "Left"),
            create_node("c", "Right"),
            create_node("d", "Bottom"),
        ],
        connections: vec![
            create_connection("a", "b", None),
            create_connection("a", "c", None),
            create_connection("b", "d", None),
            create_connection("c", "d", None),
        ],
    };

    let layout = LayoutEngine::layout(&diagram);

    // Find nodes by identifier
    let node_a = layout
        .nodes
        .iter()
        .find(|n| n.node.identifier == "a")
        .expect("Node a should exist");
    let node_b = layout
        .nodes
        .iter()
        .find(|n| n.node.identifier == "b")
        .expect("Node b should exist");
    let node_c = layout
        .nodes
        .iter()
        .find(|n| n.node.identifier == "c")
        .expect("Node c should exist");
    let node_d = layout
        .nodes
        .iter()
        .find(|n| n.node.identifier == "d")
        .expect("Node d should exist");

    // a should be in layer 0 (source)
    assert_eq!(node_a.position.x, 0.0, "Node a should be in layer 0");

    // b and c should be in layer 1 (same x, different y)
    assert_eq!(
        node_b.position.x, node_c.position.x,
        "Nodes b and c should be in same layer"
    );
    assert!(
        node_b.position.x > node_a.position.x,
        "Nodes b and c should be to the right of a"
    );

    // d should be in layer 2 (rightmost)
    assert!(
        node_d.position.x > node_b.position.x,
        "Node d should be to the right of b and c"
    );

    // All 4 connections should be positioned
    assert_eq!(layout.connections.len(), 4);
}

#[test]
fn test_layout_convergent_paths_correct_layers() {
    // Test that nodes are assigned to the maximum layer from their predecessors
    //   a
    //   |
    //   b
    //  /|
    // c d
    //  \|
    //   e
    // Node e should be in a later layer than both c and d
    let diagram = Diagram {
        nodes: vec![
            create_node("a", "A"),
            create_node("b", "B"),
            create_node("c", "C"),
            create_node("d", "D"),
            create_node("e", "E"),
        ],
        connections: vec![
            create_connection("a", "b", None),
            create_connection("b", "c", None),
            create_connection("b", "d", None),
            create_connection("c", "e", None),
            create_connection("d", "e", None),
        ],
    };

    let layout = LayoutEngine::layout(&diagram);

    let node_a = layout
        .nodes
        .iter()
        .find(|n| n.node.identifier == "a")
        .unwrap();
    let node_b = layout
        .nodes
        .iter()
        .find(|n| n.node.identifier == "b")
        .unwrap();
    let node_c = layout
        .nodes
        .iter()
        .find(|n| n.node.identifier == "c")
        .unwrap();
    let node_d = layout
        .nodes
        .iter()
        .find(|n| n.node.identifier == "d")
        .unwrap();
    let node_e = layout
        .nodes
        .iter()
        .find(|n| n.node.identifier == "e")
        .unwrap();

    // Verify layer ordering: a < b < c,d < e
    assert!(
        node_a.position.x < node_b.position.x,
        "a should be before b"
    );
    assert!(
        node_b.position.x < node_c.position.x,
        "b should be before c"
    );
    assert!(
        node_b.position.x < node_d.position.x,
        "b should be before d"
    );
    assert!(
        node_c.position.x < node_e.position.x,
        "c should be before e"
    );
    assert!(
        node_d.position.x < node_e.position.x,
        "d should be before e"
    );

    // c and d should be in the same layer
    assert_eq!(
        node_c.position.x, node_d.position.x,
        "c and d should be in same layer"
    );
}

#[test]
fn test_layout_large_graph_performance() {
    // Stress test with a larger graph to ensure performance
    // Create a chain of 100 nodes
    let mut nodes = Vec::new();
    let mut connections = Vec::new();

    for i in 0..100 {
        nodes.push(create_node(&format!("node_{}", i), &format!("Node {}", i)));
    }

    for i in 0..99 {
        connections.push(create_connection(
            &format!("node_{}", i),
            &format!("node_{}", i + 1),
            None,
        ));
    }

    let diagram = Diagram { nodes, connections };

    let start = Instant::now();
    let layout = LayoutEngine::layout(&diagram);
    let duration = start.elapsed();

    // Should complete in under 2 seconds for 100 nodes
    assert!(
        duration < Duration::from_secs(2),
        "Layout should handle 100 nodes efficiently (took {:?})",
        duration
    );

    // Verify all nodes are positioned
    assert_eq!(layout.nodes.len(), 100);
    assert_eq!(layout.connections.len(), 99);
}

#[test]
fn test_layout_wide_graph_multiple_branches() {
    // Test with a wide graph (many nodes in same layer)
    //      a
    //   /  |  \
    //  b   c   d
    //   \  |  /
    //      e
    let diagram = Diagram {
        nodes: vec![
            create_node("a", "Root"),
            create_node("b", "Branch1"),
            create_node("c", "Branch2"),
            create_node("d", "Branch3"),
            create_node("e", "Sink"),
        ],
        connections: vec![
            create_connection("a", "b", None),
            create_connection("a", "c", None),
            create_connection("a", "d", None),
            create_connection("b", "e", None),
            create_connection("c", "e", None),
            create_connection("d", "e", None),
        ],
    };

    let layout = LayoutEngine::layout(&diagram);

    let node_a = layout
        .nodes
        .iter()
        .find(|n| n.node.identifier == "a")
        .unwrap();
    let node_b = layout
        .nodes
        .iter()
        .find(|n| n.node.identifier == "b")
        .unwrap();
    let node_c = layout
        .nodes
        .iter()
        .find(|n| n.node.identifier == "c")
        .unwrap();
    let node_d = layout
        .nodes
        .iter()
        .find(|n| n.node.identifier == "d")
        .unwrap();
    let node_e = layout
        .nodes
        .iter()
        .find(|n| n.node.identifier == "e")
        .unwrap();

    // a in layer 0
    assert_eq!(node_a.position.x, 0.0);

    // b, c, d in layer 1 (same x, different y)
    assert_eq!(node_b.position.x, node_c.position.x);
    assert_eq!(node_c.position.x, node_d.position.x);
    assert_ne!(node_b.position.y, node_c.position.y);
    assert_ne!(node_c.position.y, node_d.position.y);

    // e in layer 2
    assert!(node_e.position.x > node_b.position.x);
}

#[test]
fn test_layout_disconnected_subgraphs() {
    // Test with multiple disconnected components
    // Component 1: a -> b
    // Component 2: c -> d
    let diagram = Diagram {
        nodes: vec![
            create_node("a", "A"),
            create_node("b", "B"),
            create_node("c", "C"),
            create_node("d", "D"),
        ],
        connections: vec![
            create_connection("a", "b", None),
            create_connection("c", "d", None),
        ],
    };

    let layout = LayoutEngine::layout(&diagram);

    // All nodes should be positioned
    assert_eq!(layout.nodes.len(), 4);
    assert_eq!(layout.connections.len(), 2);

    // Both components should have proper layer structure
    let node_a = layout
        .nodes
        .iter()
        .find(|n| n.node.identifier == "a")
        .unwrap();
    let node_b = layout
        .nodes
        .iter()
        .find(|n| n.node.identifier == "b")
        .unwrap();
    let node_c = layout
        .nodes
        .iter()
        .find(|n| n.node.identifier == "c")
        .unwrap();
    let node_d = layout
        .nodes
        .iter()
        .find(|n| n.node.identifier == "d")
        .unwrap();

    // a and c should both be in layer 0 (sources)
    assert_eq!(node_a.position.x, 0.0);
    assert_eq!(node_c.position.x, 0.0);

    // b and d should both be in layer 1
    assert!(node_b.position.x > 0.0);
    assert!(node_d.position.x > 0.0);
}

#[test]
fn test_layout_iterative_relaxation_convergence() {
    // Test specifically for the iterative relaxation algorithm
    // Create a graph that requires multiple iterations to stabilize
    //   a -> b -> d
    //   a -> c -> d -> e
    // Node d should be in layer 2 (not layer 1) after relaxation
    let diagram = Diagram {
        nodes: vec![
            create_node("a", "A"),
            create_node("b", "B"),
            create_node("c", "C"),
            create_node("d", "D"),
            create_node("e", "E"),
        ],
        connections: vec![
            create_connection("a", "b", None),
            create_connection("a", "c", None),
            create_connection("b", "d", None),
            create_connection("c", "d", None),
            create_connection("d", "e", None),
        ],
    };

    let layout = LayoutEngine::layout(&diagram);

    let node_a = layout
        .nodes
        .iter()
        .find(|n| n.node.identifier == "a")
        .unwrap();
    let node_b = layout
        .nodes
        .iter()
        .find(|n| n.node.identifier == "b")
        .unwrap();
    let node_c = layout
        .nodes
        .iter()
        .find(|n| n.node.identifier == "c")
        .unwrap();
    let node_d = layout
        .nodes
        .iter()
        .find(|n| n.node.identifier == "d")
        .unwrap();
    let node_e = layout
        .nodes
        .iter()
        .find(|n| n.node.identifier == "e")
        .unwrap();

    // Verify correct layer assignments
    assert_eq!(node_a.position.x, 0.0, "a should be in layer 0");
    assert!(
        node_b.position.x > node_a.position.x,
        "b should be in layer 1"
    );
    assert!(
        node_c.position.x > node_a.position.x,
        "c should be in layer 1"
    );
    assert!(
        node_d.position.x > node_b.position.x,
        "d should be in layer 2"
    );
    assert!(
        node_e.position.x > node_d.position.x,
        "e should be in layer 3"
    );

    // b and c should be in same layer
    assert_eq!(node_b.position.x, node_c.position.x);
}
