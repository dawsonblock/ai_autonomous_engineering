use crate::types::{
    LayoutDiagram, NodeType, PositionedConnection, PositionedNode, SVG_FONT_SIZE, SVG_STROKE_WIDTH,
};

/// SVG renderer for generating production-quality vector graphics.
///
/// Renders positioned nodes and connections as SVG elements with
/// appropriate styling. Different node types are rendered with
/// distinct shapes (rectangles for services, cylinders for databases).
pub struct SvgRenderer;

impl SvgRenderer {
    /// Render layout to SVG string.
    ///
    /// Generates a complete, well-formed SVG document suitable for
    /// embedding in web pages or viewing in any SVG-compatible tool.
    ///
    /// # Arguments
    ///
    /// * `layout` - The layout with positioned nodes and connections
    ///
    /// # Returns
    ///
    /// A string containing the complete SVG document.
    pub fn render(layout: &LayoutDiagram) -> String {
        let mut svg = String::new();

        // SVG header with viewBox
        svg.push_str(&Self::render_header(layout.width, layout.height));

        // Define arrow marker for connections
        svg.push_str(&Self::render_defs());

        // Render all connections (draw before nodes so arrows are behind)
        for conn in &layout.connections {
            svg.push_str(&Self::render_connection(conn));
        }

        // Render all nodes
        for node in &layout.nodes {
            svg.push_str(&Self::render_node(node));
        }

        svg.push_str("</svg>\n");
        svg
    }

    fn render_header(width: f64, height: f64) -> String {
        format!(
            "<svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 {:.0} {:.0}\" width=\"{:.0}\" height=\"{:.0}\">\n",
            width + 20.0,
            height + 20.0,
            width + 20.0,
            height + 20.0
        )
    }

    fn render_defs() -> String {
        // Define arrowhead marker
        "  <defs>\n\
    <marker id=\"arrowhead\" markerWidth=\"10\" markerHeight=\"10\" refX=\"9\" refY=\"3\" orient=\"auto\">\n\
      <polygon points=\"0 0, 10 3, 0 6\" fill=\"#333\" />\n\
    </marker>\n\
  </defs>\n"
            .to_string()
    }

    fn render_node(node: &PositionedNode) -> String {
        let shape = Self::render_node_shape(node);
        let text = Self::render_node_text(node);
        format!("  <g>\n{}{}</g>\n", shape, text)
    }

    fn render_node_shape(node: &PositionedNode) -> String {
        match node.node.node_type {
            NodeType::Service | NodeType::External | NodeType::Queue => {
                // Rectangle with rounded corners
                format!(
                    "    <rect x=\"{:.1}\" y=\"{:.1}\" width=\"{:.1}\" height=\"{:.1}\" fill=\"#E3F2FD\" stroke=\"#1976D2\" stroke-width=\"{:.1}\" rx=\"5\" />\n",
                    node.position.x, node.position.y, node.width, node.height, SVG_STROKE_WIDTH
                )
            }
            NodeType::Database => {
                // Cylinder (approximated with path)
                Self::render_cylinder(node)
            }
        }
    }

    fn render_cylinder(node: &PositionedNode) -> String {
        // SVG path for cylinder shape
        let x = node.position.x;
        let y = node.position.y;
        let w = node.width;
        let h = node.height;
        let eh = h * 0.1; // ellipse height

        format!(
            "    <path d=\"M {:.1},{:.1} L {:.1},{:.1} Q {:.1},{:.1} {:.1},{:.1} Q {:.1},{:.1} {:.1},{:.1} L {:.1},{:.1} Q {:.1},{:.1} {:.1},{:.1} Q {:.1},{:.1} {:.1},{:.1} Z M {:.1},{:.1} Q {:.1},{:.1} {:.1},{:.1} Q {:.1},{:.1} {:.1},{:.1}\" fill=\"#FFF3E0\" stroke=\"#E65100\" stroke-width=\"{:.1}\" />\n",
            x,
            y + eh,
            x,
            y + h - eh,
            x + w / 2.0,
            y + h - eh + eh / 2.0,
            x + w,
            y + h - eh,
            x + w - w / 2.0,
            y + h - eh + eh / 2.0,
            x + w,
            y + h - eh,
            x + w,
            y + eh,
            x + w - w / 2.0,
            y + eh + eh / 2.0,
            x + w,
            y + eh,
            x + w / 2.0,
            y + eh - eh / 2.0,
            x,
            y + eh,
            x,
            y + eh,
            x + w / 2.0,
            y + eh + eh / 2.0,
            x + w,
            y + eh,
            x + w - w / 2.0,
            y + eh - eh / 2.0,
            x + w,
            y + eh,
            SVG_STROKE_WIDTH
        )
    }

    fn render_node_text(node: &PositionedNode) -> String {
        let cx = node.position.x + node.width / 2.0;
        let cy = node.position.y + node.height / 2.0;
        format!(
            "    <text x=\"{:.1}\" y=\"{:.1}\" text-anchor=\"middle\" dominant-baseline=\"middle\" font-size=\"{:.0}\" font-family=\"Arial, sans-serif\">{}</text>\n",
            cx,
            cy,
            SVG_FONT_SIZE,
            Self::escape_xml(&node.node.display_name)
        )
    }

    fn render_connection(conn: &PositionedConnection) -> String {
        let line = format!(
            "    <line x1=\"{:.1}\" y1=\"{:.1}\" x2=\"{:.1}\" y2=\"{:.1}\" stroke=\"#333\" stroke-width=\"{:.1}\" marker-end=\"url(#arrowhead)\" />\n",
            conn.start.x, conn.start.y, conn.end.x, conn.end.y, SVG_STROKE_WIDTH
        );

        let label = if let Some(ref text) = conn.connection.label {
            let mid_x = (conn.start.x + conn.end.x) / 2.0;
            let mid_y = (conn.start.y + conn.end.y) / 2.0 - 5.0; // Offset above line
            format!(
                "    <text x=\"{:.1}\" y=\"{:.1}\" text-anchor=\"middle\" font-size=\"{:.0}\" font-family=\"Arial, sans-serif\" fill=\"#666\">{}</text>\n",
                mid_x,
                mid_y,
                SVG_FONT_SIZE - 2.0,
                Self::escape_xml(text)
            )
        } else {
            String::new()
        };

        format!("  <g>\n{}{}</g>\n", line, label)
    }

    fn escape_xml(s: &str) -> String {
        s.replace('&', "&amp;")
            .replace('<', "&lt;")
            .replace('>', "&gt;")
            .replace('"', "&quot;")
            .replace('\'', "&apos;")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{
        Connection, LayoutDiagram, Node, NodeType, Point, PositionedConnection, PositionedNode,
        SourcePosition, DEFAULT_NODE_HEIGHT, DEFAULT_NODE_WIDTH,
    };

    fn create_node(id: &str, display_name: &str, node_type: NodeType) -> Node {
        Node {
            identifier: id.to_string(),
            display_name: display_name.to_string(),
            node_type,
            position: SourcePosition { line: 1, column: 1 },
        }
    }

    fn create_positioned_node(
        node: Node,
        x: f64,
        y: f64,
        width: f64,
        height: f64,
    ) -> PositionedNode {
        PositionedNode {
            node,
            position: Point { x, y },
            width,
            height,
        }
    }

    fn create_connection(from: &str, to: &str, label: Option<&str>) -> Connection {
        Connection {
            from: from.to_string(),
            to: to.to_string(),
            label: label.map(|s| s.to_string()),
            position: SourcePosition { line: 1, column: 1 },
        }
    }

    fn create_positioned_connection(
        connection: Connection,
        start_x: f64,
        start_y: f64,
        end_x: f64,
        end_y: f64,
    ) -> PositionedConnection {
        PositionedConnection {
            connection,
            start: Point {
                x: start_x,
                y: start_y,
            },
            end: Point { x: end_x, y: end_y },
        }
    }

    #[test]
    fn test_render_empty_diagram() {
        let layout = LayoutDiagram {
            nodes: vec![],
            connections: vec![],
            width: 0.0,
            height: 0.0,
        };

        let svg = SvgRenderer::render(&layout);

        // Check for valid SVG structure
        assert!(svg.starts_with("<svg"));
        assert!(svg.ends_with("</svg>\n"));
        assert!(svg.contains("xmlns=\"http://www.w3.org/2000/svg\""));
        assert!(svg.contains("viewBox="));
        assert!(svg.contains("<defs>"));
        assert!(svg.contains("id=\"arrowhead\""));
    }

    #[test]
    fn test_render_service_node() {
        let node = create_node("api", "API Service", NodeType::Service);
        let positioned_node =
            create_positioned_node(node, 10.0, 20.0, DEFAULT_NODE_WIDTH, DEFAULT_NODE_HEIGHT);

        let layout = LayoutDiagram {
            nodes: vec![positioned_node],
            connections: vec![],
            width: DEFAULT_NODE_WIDTH + 10.0,
            height: DEFAULT_NODE_HEIGHT + 20.0,
        };

        let svg = SvgRenderer::render(&layout);

        // Should contain a rectangle for service node
        assert!(svg.contains("<rect"));
        assert!(svg.contains("x=\"10.0\""));
        assert!(svg.contains("y=\"20.0\""));
        assert!(svg.contains("rx=\"5\"")); // rounded corners
        assert!(svg.contains("API Service"));
    }

    #[test]
    fn test_render_database_node() {
        let node = create_node("db", "PostgreSQL", NodeType::Database);
        let positioned_node =
            create_positioned_node(node, 0.0, 0.0, DEFAULT_NODE_WIDTH, DEFAULT_NODE_HEIGHT);

        let layout = LayoutDiagram {
            nodes: vec![positioned_node],
            connections: vec![],
            width: DEFAULT_NODE_WIDTH,
            height: DEFAULT_NODE_HEIGHT,
        };

        let svg = SvgRenderer::render(&layout);

        // Should contain a path for database cylinder
        assert!(svg.contains("<path"));
        assert!(svg.contains("d=\"M "));
        assert!(svg.contains("fill=\"#FFF3E0\""));
        assert!(svg.contains("stroke=\"#E65100\""));
        assert!(svg.contains("PostgreSQL"));
    }

    #[test]
    fn test_render_connection_with_label() {
        let node1 = create_node("a", "Service A", NodeType::Service);
        let node2 = create_node("b", "Service B", NodeType::Service);
        let positioned_node1 =
            create_positioned_node(node1, 0.0, 0.0, DEFAULT_NODE_WIDTH, DEFAULT_NODE_HEIGHT);
        let positioned_node2 =
            create_positioned_node(node2, 200.0, 0.0, DEFAULT_NODE_WIDTH, DEFAULT_NODE_HEIGHT);

        let connection = create_connection("a", "b", Some("HTTP Request"));
        let positioned_connection = create_positioned_connection(
            connection,
            DEFAULT_NODE_WIDTH / 2.0,
            DEFAULT_NODE_HEIGHT / 2.0,
            200.0 + DEFAULT_NODE_WIDTH / 2.0,
            DEFAULT_NODE_HEIGHT / 2.0,
        );

        let layout = LayoutDiagram {
            nodes: vec![positioned_node1, positioned_node2],
            connections: vec![positioned_connection],
            width: 200.0 + DEFAULT_NODE_WIDTH,
            height: DEFAULT_NODE_HEIGHT,
        };

        let svg = SvgRenderer::render(&layout);

        // Should contain a line with arrowhead marker
        assert!(svg.contains("<line"));
        assert!(svg.contains("marker-end=\"url(#arrowhead)\""));
        assert!(svg.contains("HTTP Request"));
    }

    #[test]
    fn test_render_connection_without_label() {
        let node1 = create_node("a", "Service A", NodeType::Service);
        let node2 = create_node("b", "Service B", NodeType::Service);
        let positioned_node1 =
            create_positioned_node(node1, 0.0, 0.0, DEFAULT_NODE_WIDTH, DEFAULT_NODE_HEIGHT);
        let positioned_node2 =
            create_positioned_node(node2, 200.0, 0.0, DEFAULT_NODE_WIDTH, DEFAULT_NODE_HEIGHT);

        let connection = create_connection("a", "b", None);
        let positioned_connection = create_positioned_connection(
            connection,
            DEFAULT_NODE_WIDTH / 2.0,
            DEFAULT_NODE_HEIGHT / 2.0,
            200.0 + DEFAULT_NODE_WIDTH / 2.0,
            DEFAULT_NODE_HEIGHT / 2.0,
        );

        let layout = LayoutDiagram {
            nodes: vec![positioned_node1, positioned_node2],
            connections: vec![positioned_connection],
            width: 200.0 + DEFAULT_NODE_WIDTH,
            height: DEFAULT_NODE_HEIGHT,
        };

        let svg = SvgRenderer::render(&layout);

        // Should contain a line with arrowhead marker
        assert!(svg.contains("<line"));
        assert!(svg.contains("marker-end=\"url(#arrowhead)\""));

        // Count text elements - should only have 2 (for the two node labels)
        let text_count = svg.matches("<text").count();
        assert_eq!(
            text_count, 2,
            "Should have exactly 2 text elements (node labels only)"
        );
    }

    #[test]
    fn test_xml_escaping() {
        let node = create_node(
            "test",
            "Test & <Node> \"with\" 'special' chars",
            NodeType::Service,
        );
        let positioned_node =
            create_positioned_node(node, 0.0, 0.0, DEFAULT_NODE_WIDTH, DEFAULT_NODE_HEIGHT);

        let layout = LayoutDiagram {
            nodes: vec![positioned_node],
            connections: vec![],
            width: DEFAULT_NODE_WIDTH,
            height: DEFAULT_NODE_HEIGHT,
        };

        let svg = SvgRenderer::render(&layout);

        // Special characters should be escaped
        assert!(svg.contains("&amp;"));
        assert!(svg.contains("&lt;"));
        assert!(svg.contains("&gt;"));
        assert!(svg.contains("&quot;"));
        assert!(svg.contains("&apos;"));

        // Should not contain unescaped special characters in text content
        // The raw & should be escaped
        assert!(!svg.contains("Test & <Node>"));
    }

    #[test]
    fn test_viewbox_calculation() {
        let layout = LayoutDiagram {
            nodes: vec![],
            connections: vec![],
            width: 100.0,
            height: 200.0,
        };

        let svg = SvgRenderer::render(&layout);

        // ViewBox should add 20.0 to both width and height for padding
        assert!(svg.contains("viewBox=\"0 0 120 220\""));
        assert!(svg.contains("width=\"120\""));
        assert!(svg.contains("height=\"220\""));
    }

    #[test]
    fn test_render_complete_diagram() {
        // Create a complete diagram with multiple nodes and connections
        let node1 = create_node("api", "API Gateway", NodeType::Service);
        let node2 = create_node("db", "Database", NodeType::Database);
        let node3 = create_node("queue", "Message Queue", NodeType::Queue);

        let positioned_node1 =
            create_positioned_node(node1, 0.0, 0.0, DEFAULT_NODE_WIDTH, DEFAULT_NODE_HEIGHT);
        let positioned_node2 =
            create_positioned_node(node2, 200.0, 0.0, DEFAULT_NODE_WIDTH, DEFAULT_NODE_HEIGHT);
        let positioned_node3 =
            create_positioned_node(node3, 100.0, 160.0, DEFAULT_NODE_WIDTH, DEFAULT_NODE_HEIGHT);

        let connection1 = create_connection("api", "db", Some("SQL Query"));
        let connection2 = create_connection("api", "queue", None);

        let positioned_connection1 = create_positioned_connection(
            connection1,
            DEFAULT_NODE_WIDTH / 2.0,
            DEFAULT_NODE_HEIGHT / 2.0,
            200.0 + DEFAULT_NODE_WIDTH / 2.0,
            DEFAULT_NODE_HEIGHT / 2.0,
        );

        let positioned_connection2 = create_positioned_connection(
            connection2,
            DEFAULT_NODE_WIDTH / 2.0,
            DEFAULT_NODE_HEIGHT / 2.0,
            100.0 + DEFAULT_NODE_WIDTH / 2.0,
            160.0 + DEFAULT_NODE_HEIGHT / 2.0,
        );

        let layout = LayoutDiagram {
            nodes: vec![positioned_node1, positioned_node2, positioned_node3],
            connections: vec![positioned_connection1, positioned_connection2],
            width: 320.0,
            height: 220.0,
        };

        let svg = SvgRenderer::render(&layout);

        // Basic structure checks
        assert!(svg.starts_with("<svg"));
        assert!(svg.ends_with("</svg>\n"));
        assert!(svg.contains("xmlns=\"http://www.w3.org/2000/svg\""));

        // Check all nodes are present
        assert!(svg.contains("API Gateway"));
        assert!(svg.contains("Database"));
        assert!(svg.contains("Message Queue"));

        // Check for rect (service nodes)
        assert!(svg.matches("<rect").count() >= 2);

        // Check for path (database node)
        assert!(svg.contains("<path"));

        // Check connections
        assert!(svg.matches("<line").count() >= 2);
        assert!(svg.contains("SQL Query"));
    }
}
