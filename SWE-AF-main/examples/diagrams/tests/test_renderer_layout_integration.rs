/// Integration tests for Renderer ↔ Layout Engine interactions
///
/// These tests verify that both SVG and ASCII renderers correctly interpret
/// the positioned layout data produced by the LayoutEngine and that layout
/// coordinates are properly transformed into output formats.
use diagrams::ascii::AsciiRenderer;
use diagrams::layout::LayoutEngine;
use diagrams::lexer::Lexer;
use diagrams::parser::Parser;
use diagrams::svg::SvgRenderer;
use diagrams::validator::Validator;

/// Helper to parse DSL and generate layout
fn parse_and_layout(dsl: &str) -> diagrams::types::LayoutDiagram {
    let mut lexer = Lexer::new(dsl);
    let tokens = lexer.tokenize().expect("Lexer should succeed");
    let mut parser = Parser::new(tokens);
    let diagram = parser.parse().expect("Parser should succeed");
    Validator::validate(&diagram).expect("Validator should succeed");
    LayoutEngine::layout(&diagram)
}

/// Test that SVG renderer produces valid SVG from layout
#[test]
fn test_svg_renderer_produces_valid_svg_from_layout() {
    let dsl = r#"node "Node A" as a
node "Node B" as b
a -> b : "connection"
"#;

    let layout = parse_and_layout(dsl);

    // Render to SVG
    let svg = SvgRenderer::render(&layout);

    // Verify SVG structure
    assert!(svg.starts_with("<svg"), "Should start with <svg tag");
    assert!(svg.contains("xmlns=\"http://www.w3.org/2000/svg\""));
    assert!(svg.contains("</svg>"), "Should end with </svg> tag");

    // Verify node labels are in SVG
    assert!(svg.contains("Node A"), "SVG should contain 'Node A'");
    assert!(svg.contains("Node B"), "SVG should contain 'Node B'");

    // Verify connection label is in SVG
    assert!(
        svg.contains("connection"),
        "SVG should contain connection label"
    );

    // Verify SVG has geometric elements
    assert!(svg.contains("<rect"), "Should have rectangles for nodes");
    assert!(
        svg.contains("<path") || svg.contains("<line"),
        "Should have paths or lines for connections"
    );
}

/// Test that ASCII renderer produces box characters from layout
#[test]
fn test_ascii_renderer_produces_box_characters_from_layout() {
    let dsl = r#"node "Node A" as a
node "Node B" as b
a -> b
"#;

    let layout = parse_and_layout(dsl);

    // Render to ASCII
    let ascii = AsciiRenderer::render(&layout);

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
}

/// Test that both renderers handle empty layout
#[test]
fn test_renderers_handle_empty_layout() {
    let dsl = "";

    let layout = parse_and_layout(dsl);

    // SVG should still be valid
    let svg = SvgRenderer::render(&layout);
    assert!(
        svg.starts_with("<svg"),
        "Empty layout should produce valid SVG"
    );
    assert!(svg.contains("</svg>"));

    // ASCII should handle empty layout
    let ascii = AsciiRenderer::render(&layout);
    assert_eq!(ascii, "", "Empty layout should produce empty ASCII");
}

/// Test that both renderers handle single node (no connections)
#[test]
fn test_renderers_handle_single_node() {
    let dsl = r#"node "Standalone Node" as standalone"#;

    let layout = parse_and_layout(dsl);

    // SVG should contain the node
    let svg = SvgRenderer::render(&layout);
    assert!(svg.contains("Standalone Node"), "SVG should contain node");
    assert!(svg.contains("<rect"), "Should have rectangle for node");

    // ASCII should render the node
    let ascii = AsciiRenderer::render(&layout);
    assert!(
        !ascii.is_empty(),
        "ASCII should not be empty for single node"
    );
}

/// Test that renderers handle multiple disconnected nodes
#[test]
fn test_renderers_handle_disconnected_nodes() {
    let dsl = r#"node "Node 1" as n1
node "Node 2" as n2
node "Node 3" as n3
"#;

    let layout = parse_and_layout(dsl);

    // Verify layout has correct number of nodes
    assert_eq!(layout.nodes.len(), 3, "Layout should have 3 nodes");
    assert_eq!(
        layout.connections.len(),
        0,
        "Layout should have no connections"
    );

    // SVG should contain all nodes
    let svg = SvgRenderer::render(&layout);
    assert!(svg.contains("Node 1"), "SVG should contain Node 1");
    assert!(svg.contains("Node 2"), "SVG should contain Node 2");
    assert!(svg.contains("Node 3"), "SVG should contain Node 3");

    // ASCII should render all nodes
    let ascii = AsciiRenderer::render(&layout);
    assert!(!ascii.is_empty(), "ASCII should not be empty");
}

/// Test that layout dimensions are reflected in SVG viewBox
#[test]
fn test_svg_viewbox_matches_layout_dimensions() {
    let dsl = r#"node "Node A" as a
node "Node B" as b
node "Node C" as c
a -> b
b -> c
"#;

    let layout = parse_and_layout(dsl);

    // Layout should have positive dimensions
    assert!(layout.width > 0.0, "Layout width should be positive");
    assert!(layout.height > 0.0, "Layout height should be positive");

    // Render to SVG
    let svg = SvgRenderer::render(&layout);

    // SVG should have viewBox attribute
    assert!(svg.contains("viewBox"), "SVG should have viewBox attribute");

    // ViewBox should contain reasonable dimensions (adds 20.0 padding)
    let expected_width = layout.width + 20.0;
    assert!(
        svg.contains(&format!("{:.0}", expected_width))
            || svg.contains(&format!("{}", expected_width as i32)),
        "ViewBox should reflect layout width with padding"
    );
}

/// Test that positioned node coordinates are used in SVG
#[test]
fn test_svg_uses_positioned_node_coordinates() {
    let dsl = r#"node "Test Node" as test"#;

    let layout = parse_and_layout(dsl);

    // Get positioned node coordinates
    assert_eq!(layout.nodes.len(), 1, "Should have one positioned node");
    let positioned_node = &layout.nodes[0];

    // Coordinates should be non-negative
    assert!(positioned_node.position.x >= 0.0);
    assert!(positioned_node.position.y >= 0.0);

    // Render to SVG
    let svg = SvgRenderer::render(&layout);

    // SVG should contain rect element with coordinates
    assert!(svg.contains("<rect"), "Should have rect element");

    // The rect should have x and y attributes
    assert!(svg.contains("x="), "Rect should have x coordinate");
    assert!(svg.contains("y="), "Rect should have y coordinate");
}

/// Test that connection coordinates are used in SVG
#[test]
fn test_svg_uses_connection_coordinates() {
    let dsl = r#"node "From" as from
node "To" as to
from -> to : "edge"
"#;

    let layout = parse_and_layout(dsl);

    // Verify positioned connection
    assert_eq!(
        layout.connections.len(),
        1,
        "Should have one positioned connection"
    );
    let conn = &layout.connections[0];

    // Connection should have valid coordinates
    assert!(conn.start.x >= 0.0);
    assert!(conn.start.y >= 0.0);
    assert!(conn.end.x >= 0.0);
    assert!(conn.end.y >= 0.0);

    // Render to SVG
    let svg = SvgRenderer::render(&layout);

    // SVG should contain path or line for connection
    assert!(
        svg.contains("<path") || svg.contains("<line"),
        "Should have path or line for connection"
    );

    // SVG should contain connection label
    assert!(svg.contains("edge"), "Should contain connection label");
}

/// Test that different node types are rendered with distinct styles
#[test]
fn test_svg_renders_different_node_types_distinctly() {
    let dsl = r#"node "Service Node" as svc [type: service]
node "Database Node" as db [type: database]
node "External Node" as ext [type: external]
"#;

    let layout = parse_and_layout(dsl);

    // All nodes should be positioned
    assert_eq!(layout.nodes.len(), 3);

    // Render to SVG
    let svg = SvgRenderer::render(&layout);

    // All node labels should be present
    assert!(svg.contains("Service Node"));
    assert!(svg.contains("Database Node"));
    assert!(svg.contains("External Node"));
    // Service and External nodes use <rect>, Database nodes use <path>
    let rect_count = svg.matches("<rect").count();
    let path_count = svg.matches("<path").count();
    assert_eq!(
        rect_count, 2,
        "Should have 2 rectangles for service/external nodes"
    );
    assert!(
        path_count >= 1,
        "Should have at least 1 path for database node"
    );

    // Different node types should have different fill colors
    assert!(
        svg.contains("fill="),
        "Nodes should have fill colors defined"
    );
}

/// Test that layout with multiple connections renders all connections
#[test]
fn test_svg_renders_all_connections() {
    let dsl = r#"node "Hub" as hub
node "Spoke 1" as s1
node "Spoke 2" as s2
node "Spoke 3" as s3

hub -> s1
hub -> s2
hub -> s3
"#;

    let layout = parse_and_layout(dsl);

    // Verify layout has all connections
    assert_eq!(layout.connections.len(), 3, "Should have 3 connections");

    // Render to SVG
    let svg = SvgRenderer::render(&layout);

    // Count connection elements (paths or lines)
    let path_count = svg.matches("<path").count();
    let line_count = svg.matches("<line").count();
    let total_connections = path_count + line_count;

    assert!(
        total_connections >= 3,
        "Should have at least 3 connection elements"
    );
}

/// Test that ASCII renderer scales layout to terminal-friendly dimensions
#[test]
fn test_ascii_scales_layout_appropriately() {
    let dsl = r#"node "A" as a
node "B" as b
node "C" as c
a -> b
b -> c
"#;

    let layout = parse_and_layout(dsl);

    // Layout uses arbitrary coordinate space
    assert!(layout.width > 0.0);
    assert!(layout.height > 0.0);

    // ASCII should scale to reasonable terminal dimensions
    let ascii = AsciiRenderer::render(&layout);

    // ASCII output should not be excessively large
    let line_count = ascii.lines().count();
    assert!(
        line_count < 200,
        "ASCII should scale to reasonable line count"
    );

    // Each line should not be excessively wide
    for line in ascii.lines() {
        assert!(
            line.len() < 300,
            "ASCII lines should be reasonable width: {}",
            line.len()
        );
    }
}

/// Test that connection labels are positioned near their connections
#[test]
fn test_connection_labels_positioned_correctly() {
    let dsl = r#"node "Source" as src
node "Target" as tgt
src -> tgt : "labeled edge"
"#;

    let layout = parse_and_layout(dsl);

    let svg = SvgRenderer::render(&layout);

    // Connection label should be present in SVG
    assert!(
        svg.contains("labeled edge"),
        "SVG should contain connection label"
    );

    // Label should be in a text element
    assert!(
        svg.contains("<text"),
        "SVG should have text elements for labels"
    );

    // The text element for the label should have coordinates
    // (verifying it's positioned, not just floating)
    let text_section = svg
        .split("<text")
        .find(|s| s.contains("labeled edge"))
        .expect("Should find text element with label");

    assert!(
        text_section.contains("x=") && text_section.contains("y="),
        "Label text should have coordinates"
    );
}

/// Test that layout with complex branching renders correctly
#[test]
fn test_complex_branching_layout_renders() {
    let dsl = r#"node "Root" as root
node "Branch A" as a
node "Branch B" as b
node "Leaf A1" as a1
node "Leaf A2" as a2
node "Leaf B1" as b1

root -> a
root -> b
a -> a1
a -> a2
b -> b1
"#;

    let layout = parse_and_layout(dsl);

    // Verify layout structure
    assert_eq!(layout.nodes.len(), 6);
    assert_eq!(layout.connections.len(), 5);

    // Verify all nodes have valid positions
    for node in &layout.nodes {
        assert!(node.position.x >= 0.0, "Node x should be non-negative");
        assert!(node.position.y >= 0.0, "Node y should be non-negative");
        assert!(node.width > 0.0, "Node width should be positive");
        assert!(node.height > 0.0, "Node height should be positive");
    }

    // Verify all connections have valid coordinates
    for conn in &layout.connections {
        assert!(conn.start.x >= 0.0);
        assert!(conn.start.y >= 0.0);
        assert!(conn.end.x >= 0.0);
        assert!(conn.end.y >= 0.0);
    }

    // SVG should render all nodes
    let svg = SvgRenderer::render(&layout);
    assert!(svg.contains("Root"));
    assert!(svg.contains("Branch A"));
    assert!(svg.contains("Branch B"));
    assert!(svg.contains("Leaf A1"));
    assert!(svg.contains("Leaf A2"));
    assert!(svg.contains("Leaf B1"));

    // ASCII should also render
    let ascii = AsciiRenderer::render(&layout);
    assert!(!ascii.is_empty(), "ASCII should render complex layout");
}

/// Test that layout with connection without label renders correctly
#[test]
fn test_connection_without_label_renders() {
    let dsl = r#"node "A" as a
node "B" as b
a -> b
"#;

    let layout = parse_and_layout(dsl);

    // Verify connection exists
    assert_eq!(layout.connections.len(), 1);
    let conn = &layout.connections[0];
    assert!(
        conn.connection.label.is_none(),
        "Connection should have no label"
    );

    // SVG should render connection without label
    let svg = SvgRenderer::render(&layout);

    // Should have path or line for connection
    assert!(
        svg.contains("<path") || svg.contains("<line"),
        "Should have connection element"
    );

    // Should have nodes
    assert!(svg.contains("A") || svg.contains("B"));
}

/// Test that SVG escapes special characters in labels
#[test]
fn test_svg_escapes_special_characters() {
    let dsl = r#"node "Node with <special> & \"chars\"" as node1
node "Normal" as node2
node1 -> node2 : "Label with <xml> & entities"
"#;

    let layout = parse_and_layout(dsl);

    let svg = SvgRenderer::render(&layout);

    // SVG should escape special characters
    // < should become &lt;, > should become &gt;, & should become &amp;
    assert!(
        svg.contains("&lt;") || !svg.contains("<special>"),
        "SVG should escape < characters in content"
    );

    // The SVG should still be valid XML
    assert!(svg.starts_with("<svg"));
    assert!(svg.contains("</svg>"));
}

/// Test renderer consistency: same layout produces same output
#[test]
fn test_renderer_deterministic_output() {
    let dsl = r#"node "Node A" as a
node "Node B" as b
a -> b : "connection"
"#;

    let layout = parse_and_layout(dsl);

    // Render twice
    let svg1 = SvgRenderer::render(&layout);
    let svg2 = SvgRenderer::render(&layout);

    // Should produce identical output
    assert_eq!(
        svg1, svg2,
        "SVG renderer should produce deterministic output"
    );

    let ascii1 = AsciiRenderer::render(&layout);
    let ascii2 = AsciiRenderer::render(&layout);

    assert_eq!(
        ascii1, ascii2,
        "ASCII renderer should produce deterministic output"
    );
}

/// Test that large diagrams render without errors
#[test]
fn test_large_diagram_renders_successfully() {
    // Build a large diagram programmatically
    let mut dsl = String::new();

    // Create 20 nodes
    for i in 0..20 {
        dsl.push_str(&format!("node \"Node {}\" as n{}\n", i, i));
    }

    // Create a chain of connections
    for i in 0..19 {
        dsl.push_str(&format!("n{} -> n{}\n", i, i + 1));
    }

    let layout = parse_and_layout(&dsl);

    // Verify layout has all nodes and connections
    assert_eq!(layout.nodes.len(), 20);
    assert_eq!(layout.connections.len(), 19);

    // Both renderers should handle large diagrams
    let svg = SvgRenderer::render(&layout);
    assert!(
        svg.starts_with("<svg"),
        "SVG renderer should handle large diagram"
    );
    assert!(svg.contains("Node 0"));
    assert!(svg.contains("Node 19"));

    let ascii = AsciiRenderer::render(&layout);
    assert!(
        !ascii.is_empty(),
        "ASCII renderer should handle large diagram"
    );
}
