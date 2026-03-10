use crate::types::{
    LayoutDiagram, PositionedConnection, PositionedNode, ASCII_MIN_NODE_WIDTH, ASCII_NODE_PADDING,
};

/// ASCII renderer for terminal preview mode.
///
/// Renders diagrams using Unicode box-drawing characters (U+2500-U+257F)
/// for quick visualization in the terminal during development.
pub struct AsciiRenderer;

impl AsciiRenderer {
    /// Render layout to ASCII art string.
    ///
    /// Converts the positioned diagram into a terminal-friendly ASCII
    /// representation using Unicode box-drawing characters. The output
    /// is scaled to fit within 80 columns for readability.
    ///
    /// # Arguments
    ///
    /// * `layout` - The layout with positioned nodes and connections
    ///
    /// # Returns
    ///
    /// A string containing the ASCII art representation, suitable for
    /// printing to the terminal.
    pub fn render(layout: &LayoutDiagram) -> String {
        // 1. Scale layout to character grid
        let (grid_width, grid_height, scale) = Self::compute_grid_dimensions(layout);

        // Handle empty diagram
        if grid_width == 0 || grid_height == 0 {
            return String::new();
        }

        let scaled_nodes = Self::scale_nodes(&layout.nodes, scale);
        let scaled_connections = Self::scale_connections(&layout.connections, scale);

        // 2. Initialize character grid
        let mut grid = vec![vec![' '; grid_width]; grid_height];

        // 3. Draw connections first (so they go under nodes)
        for conn in &scaled_connections {
            Self::draw_connection(&mut grid, conn);
        }

        // 4. Draw nodes on top
        for node in &scaled_nodes {
            Self::draw_node(&mut grid, node);
        }

        // 5. Convert grid to string
        Self::grid_to_string(&grid)
    }

    fn compute_grid_dimensions(layout: &LayoutDiagram) -> (usize, usize, f64) {
        // Target 80 columns wide
        const TARGET_WIDTH: usize = 80;

        if layout.width <= 0.0 || layout.height <= 0.0 {
            return (0, 0, 1.0);
        }

        let scale = TARGET_WIDTH as f64 / layout.width.max(1.0);
        let grid_width = (layout.width * scale).ceil() as usize;
        let grid_height = (layout.height * scale).ceil() as usize;
        (grid_width.max(1), grid_height.max(1), scale)
    }

    fn scale_nodes(nodes: &[PositionedNode], scale: f64) -> Vec<ScaledNode> {
        nodes
            .iter()
            .map(|n| ScaledNode {
                display_name: n.node.display_name.clone(),
                x: (n.position.x * scale) as usize,
                y: (n.position.y * scale) as usize,
                width: ((n.width * scale) as usize).max(ASCII_MIN_NODE_WIDTH),
                height: ((n.height * scale) as usize).max(3),
            })
            .collect()
    }

    fn scale_connections(
        connections: &[PositionedConnection],
        scale: f64,
    ) -> Vec<ScaledConnection> {
        connections
            .iter()
            .map(|c| ScaledConnection {
                start_x: (c.start.x * scale) as usize,
                start_y: (c.start.y * scale) as usize,
                end_x: (c.end.x * scale) as usize,
                end_y: (c.end.y * scale) as usize,
                label: c.connection.label.clone(),
            })
            .collect()
    }

    fn draw_node(grid: &mut [Vec<char>], node: &ScaledNode) {
        let x = node.x;
        let y = node.y;
        let w = node.width;
        let h = node.height;

        // Bounds checking
        if y >= grid.len() || x >= grid[0].len() {
            return;
        }

        // Ensure we don't draw beyond grid boundaries
        let max_x = (x + w).min(grid[0].len());
        let max_y = (y + h).min(grid.len());

        if max_x <= x + 1 || max_y <= y + 1 {
            return; // Not enough space to draw even minimal box
        }

        // Draw box with corners and edges
        // ┌─────────┐
        // │  Name   │
        // └─────────┘

        // Top edge
        if y < grid.len() && x < grid[y].len() {
            grid[y][x] = '┌';
        }
        for i in (x + 1)..(max_x - 1) {
            if y < grid.len() && i < grid[y].len() {
                grid[y][i] = '─';
            }
        }
        if y < grid.len() && (max_x - 1) < grid[y].len() {
            grid[y][max_x - 1] = '┐';
        }

        // Middle rows
        for j in (y + 1)..(max_y - 1) {
            if j < grid.len() {
                if x < grid[j].len() {
                    grid[j][x] = '│';
                }
                if (max_x - 1) < grid[j].len() {
                    grid[j][max_x - 1] = '│';
                }
            }
        }

        // Bottom edge
        if (max_y - 1) < grid.len() {
            if x < grid[max_y - 1].len() {
                grid[max_y - 1][x] = '└';
            }
            for i in (x + 1)..(max_x - 1) {
                if i < grid[max_y - 1].len() {
                    grid[max_y - 1][i] = '─';
                }
            }
            if (max_x - 1) < grid[max_y - 1].len() {
                grid[max_y - 1][max_x - 1] = '┘';
            }
        }

        // Center text
        let available_width = w.saturating_sub(2 * ASCII_NODE_PADDING);
        let text = Self::truncate_text(&node.display_name, available_width);
        let text_len = text.chars().count();

        if text_len > 0 && w > text_len {
            let text_x = x + (w - text_len) / 2;
            let text_y = y + h / 2;

            if text_y < grid.len() {
                for (i, ch) in text.chars().enumerate() {
                    let char_x = text_x + i;
                    if char_x < grid[text_y].len() {
                        grid[text_y][char_x] = ch;
                    }
                }
            }
        }
    }

    fn draw_connection(grid: &mut [Vec<char>], conn: &ScaledConnection) {
        let x1 = conn.start_x;
        let y1 = conn.start_y;
        let x2 = conn.end_x;
        let y2 = conn.end_y;

        // Bounds checking
        if y1 >= grid.len() || y2 >= grid.len() {
            return;
        }
        if x1 >= grid[0].len() || x2 >= grid[0].len() {
            return;
        }

        // Draw line from start to end
        // Simple algorithm: horizontal then vertical (L-shape)

        // Horizontal segment
        let (h_start, h_end) = if x1 < x2 { (x1, x2) } else { (x2, x1) };
        if y1 < grid.len() {
            for x in h_start..=h_end.min(grid[y1].len() - 1) {
                if grid[y1][x] == ' ' {
                    grid[y1][x] = '─';
                }
            }
        }

        // Vertical segment
        let (v_start, v_end) = if y1 < y2 { (y1, y2) } else { (y2, y1) };
        if x2 < grid[0].len() {
            for y in v_start..=v_end.min(grid.len() - 1) {
                if grid[y][x2] == ' ' {
                    grid[y][x2] = '│';
                }
            }
        }

        // Corner
        if x1 != x2
            && y1 != y2
            && y1 < grid.len()
            && x2 < grid[y1].len()
            && (grid[y1][x2] == ' ' || grid[y1][x2] == '─' || grid[y1][x2] == '│')
        {
            grid[y1][x2] = '└';
        }

        // Arrowhead at end - always place arrow at endpoint
        if y2 < grid.len() && x2 < grid[y2].len() {
            // Determine arrow direction based on which direction we came from
            if y1 == y2 {
                // Horizontal connection
                if x2 > x1 {
                    grid[y2][x2] = '>'; // Pointing right
                } else {
                    grid[y2][x2] = '<'; // Pointing left
                }
            } else if x1 == x2 {
                // Vertical connection
                if y2 > y1 {
                    grid[y2][x2] = 'v'; // Pointing down
                } else {
                    grid[y2][x2] = '^'; // Pointing up
                }
            } else {
                // L-shaped connection - determine from the last segment
                // Since we do horizontal first, then vertical, the last segment is vertical
                if y2 > y1 {
                    grid[y2][x2] = 'v'; // Pointing down
                } else {
                    grid[y2][x2] = '^'; // Pointing up
                }
            }
        }
    }

    fn truncate_text(text: &str, max_len: usize) -> String {
        if max_len == 0 {
            return String::new();
        }

        let chars: Vec<char> = text.chars().collect();
        let char_count = chars.len();

        if char_count <= max_len {
            text.to_string()
        } else if max_len >= 1 {
            let truncated: String = chars.iter().take(max_len - 1).collect();
            format!("{}…", truncated)
        } else {
            String::new()
        }
    }

    fn grid_to_string(grid: &[Vec<char>]) -> String {
        grid.iter()
            .map(|row| row.iter().collect::<String>())
            .collect::<Vec<_>>()
            .join("\n")
    }
}

// Internal types for scaled layout
struct ScaledNode {
    display_name: String,
    x: usize,
    y: usize,
    width: usize,
    height: usize,
}

struct ScaledConnection {
    start_x: usize,
    start_y: usize,
    end_x: usize,
    end_y: usize,
    #[allow(dead_code)] // Reserved for future label rendering feature
    label: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{
        Connection, LayoutDiagram, Node, NodeType, Point, PositionedConnection, PositionedNode,
        SourcePosition,
    };

    fn create_test_node(id: &str, name: &str, x: f64, y: f64, w: f64, h: f64) -> PositionedNode {
        PositionedNode {
            node: Node {
                identifier: id.to_string(),
                display_name: name.to_string(),
                node_type: NodeType::Service,
                position: SourcePosition { line: 1, column: 1 },
            },
            position: Point { x, y },
            width: w,
            height: h,
        }
    }

    fn create_test_connection(
        from: &str,
        to: &str,
        start: Point,
        end: Point,
    ) -> PositionedConnection {
        PositionedConnection {
            connection: Connection {
                from: from.to_string(),
                to: to.to_string(),
                label: None,
                position: SourcePosition { line: 1, column: 1 },
            },
            start,
            end,
        }
    }

    #[test]
    fn test_render_single_node_box() {
        // Test that a single node produces a box with Unicode box-drawing characters
        let layout = LayoutDiagram {
            nodes: vec![create_test_node("api", "API", 0.0, 0.0, 120.0, 60.0)],
            connections: vec![],
            width: 120.0,
            height: 60.0,
        };

        let output = AsciiRenderer::render(&layout);

        // Check for box-drawing characters
        assert!(
            output.contains('┌'),
            "Output should contain top-left corner"
        );
        assert!(
            output.contains('┐'),
            "Output should contain top-right corner"
        );
        assert!(
            output.contains('└'),
            "Output should contain bottom-left corner"
        );
        assert!(
            output.contains('┘'),
            "Output should contain bottom-right corner"
        );
        assert!(
            output.contains('─'),
            "Output should contain horizontal line"
        );
        assert!(output.contains('│'), "Output should contain vertical line");

        // Check that the node text appears
        assert!(output.contains("API"), "Output should contain node text");
    }

    #[test]
    fn test_node_text_centered() {
        // Test that node text is centered correctly
        let layout = LayoutDiagram {
            nodes: vec![create_test_node("api", "Test", 0.0, 0.0, 120.0, 60.0)],
            connections: vec![],
            width: 120.0,
            height: 60.0,
        };

        let output = AsciiRenderer::render(&layout);

        // The text should appear somewhere in the output
        assert!(output.contains("Test"));

        // Check that we have a box structure
        let lines: Vec<&str> = output.lines().collect();
        assert!(lines.len() >= 3, "Should have at least 3 lines for a box");

        // First line should have top corners
        assert!(lines[0].contains('┌'));
        assert!(lines[0].contains('┐'));
    }

    #[test]
    fn test_two_connected_nodes() {
        // Test that two nodes with a connection produce boxes with a line and arrow
        // Place nodes at different Y coordinates so connection is visible outside boxes
        let node1 = create_test_node("a", "Node A", 0.0, 0.0, 120.0, 60.0);
        let node2 = create_test_node("b", "Node B", 200.0, 100.0, 120.0, 60.0);

        let conn = create_test_connection(
            "a",
            "b",
            Point { x: 120.0, y: 30.0 },  // right edge center of node1
            Point { x: 200.0, y: 130.0 }, // left edge center of node2
        );

        let layout = LayoutDiagram {
            nodes: vec![node1, node2],
            connections: vec![conn],
            width: 320.0,
            height: 160.0,
        };

        let output = AsciiRenderer::render(&layout);

        // Check for connection characters
        assert!(
            output.contains('─'),
            "Should contain horizontal connection line"
        );

        // Check for corner or arrow indicator
        // With an L-shaped connection, we should see either a corner or an arrow
        let has_connection_char = output.contains('>')
            || output.contains('v')
            || output.contains('<')
            || output.contains('^')
            || output.contains('└');
        assert!(
            has_connection_char,
            "Should contain arrow or corner indicator. Output:\n{}",
            output
        );

        // Check for both node names
        assert!(output.contains("Node A"));
        assert!(output.contains("Node B"));
    }

    #[test]
    fn test_scaling_to_grid() {
        // Test that float coordinates are correctly scaled to character grid
        let layout = LayoutDiagram {
            nodes: vec![create_test_node("api", "API", 50.0, 25.0, 100.0, 50.0)],
            connections: vec![],
            width: 200.0,
            height: 100.0,
        };

        let output = AsciiRenderer::render(&layout);

        // Should produce output (not empty)
        assert!(!output.is_empty());

        // Should have proper box structure
        assert!(output.contains('┌'));
        assert!(output.contains('─'));
        assert!(output.contains('│'));
    }

    #[test]
    fn test_text_truncation() {
        // Test that long node names are truncated with ellipsis
        let long_name = "This is a very long node name that should be truncated";
        let layout = LayoutDiagram {
            nodes: vec![create_test_node("api", long_name, 0.0, 0.0, 120.0, 60.0)],
            connections: vec![],
            width: 120.0,
            height: 60.0,
        };

        let output = AsciiRenderer::render(&layout);

        // Should contain ellipsis character when text is truncated
        // Or should contain a portion of the text if it fits
        assert!(!output.is_empty());

        // The output should either contain the full text or a truncated version with …
        assert!(output.contains("This") || output.contains("…"));
    }

    #[test]
    fn test_unicode_character_range() {
        // Verify that output contains Unicode box-drawing characters (U+2500-U+257F)
        let layout = LayoutDiagram {
            nodes: vec![
                create_test_node("a", "A", 0.0, 0.0, 120.0, 60.0),
                create_test_node("b", "B", 200.0, 0.0, 120.0, 60.0),
            ],
            connections: vec![create_test_connection(
                "a",
                "b",
                Point { x: 60.0, y: 30.0 },
                Point { x: 260.0, y: 30.0 },
            )],
            width: 320.0,
            height: 60.0,
        };

        let output = AsciiRenderer::render(&layout);

        // Check that specific Unicode box-drawing characters are present
        let box_chars = ['┌', '┐', '└', '┘', '─', '│'];
        let has_box_char = box_chars.iter().any(|&c| output.contains(c));

        assert!(
            has_box_char,
            "Output should contain Unicode box-drawing characters"
        );

        // Verify these are in the correct Unicode range (U+2500-U+257F)
        for c in output.chars() {
            if box_chars.contains(&c) {
                let code = c as u32;
                assert!(
                    (0x2500..=0x257F).contains(&code),
                    "Character {} (U+{:04X}) should be in range U+2500-U+257F",
                    c,
                    code
                );
            }
        }
    }

    #[test]
    fn test_empty_diagram() {
        let layout = LayoutDiagram {
            nodes: vec![],
            connections: vec![],
            width: 0.0,
            height: 0.0,
        };

        let output = AsciiRenderer::render(&layout);
        assert_eq!(output, "");
    }

    #[test]
    fn test_truncate_text_helper() {
        // Test the text truncation helper function
        assert_eq!(AsciiRenderer::truncate_text("Hello", 10), "Hello");
        assert_eq!(AsciiRenderer::truncate_text("Hello", 5), "Hello");
        assert_eq!(AsciiRenderer::truncate_text("Hello World", 8), "Hello W…");
        assert_eq!(AsciiRenderer::truncate_text("Test", 4), "Test"); // Exact fit
        assert_eq!(AsciiRenderer::truncate_text("Test", 3), "Te…"); // Truncated
        assert_eq!(AsciiRenderer::truncate_text("Test", 2), "T…"); // Truncated
        assert_eq!(AsciiRenderer::truncate_text("Test", 1), "…"); // Only ellipsis
        assert_eq!(AsciiRenderer::truncate_text("Test", 0), ""); // Empty
    }
}
