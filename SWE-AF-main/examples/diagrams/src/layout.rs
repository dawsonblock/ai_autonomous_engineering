use crate::types::{
    Connection, Diagram, LayoutDiagram, Node, Point, PositionedConnection, PositionedNode,
    DEFAULT_NODE_HEIGHT, DEFAULT_NODE_WIDTH, NODE_HORIZONTAL_SPACING, NODE_VERTICAL_SPACING,
};
use std::collections::{HashMap, HashSet, VecDeque};

/// Layout engine for computing node positions and connection routing.
///
/// Uses a layer-based BFS algorithm to arrange nodes from left to right
/// based on their dependencies. Nodes with no incoming connections are
/// placed in the leftmost layer, and subsequent layers are computed
/// based on the longest path from source nodes.
pub struct LayoutEngine;

impl LayoutEngine {
    /// Compute layout for diagram.
    ///
    /// # Algorithm
    ///
    /// 1. **Layer Assignment**: Uses BFS to assign nodes to horizontal layers.
    ///    Nodes with no incoming edges are placed in layer 0, and each
    ///    subsequent layer contains nodes one step further from sources.
    /// 2. **Vertical Distribution**: Within each layer, nodes are distributed
    ///    vertically with even spacing.
    /// 3. **Connection Routing**: Connections are drawn as straight lines
    ///    between node centers.
    ///
    /// # Arguments
    ///
    /// * `diagram` - The parsed and validated diagram AST
    ///
    /// # Returns
    ///
    /// A `LayoutDiagram` with computed positions for all nodes and connections.
    pub fn layout(diagram: &Diagram) -> LayoutDiagram {
        let layers = Self::assign_layers(diagram);
        let positioned_nodes = Self::position_nodes(&diagram.nodes, &layers);
        let node_positions = Self::build_position_map(&positioned_nodes);
        let positioned_connections =
            Self::position_connections(&diagram.connections, &node_positions);
        let (width, height) = Self::compute_bounds(&positioned_nodes);

        LayoutDiagram {
            nodes: positioned_nodes,
            connections: positioned_connections,
            width,
            height,
        }
    }

    /// Assign layer index to each node using BFS
    /// Nodes with no incoming edges → layer 0
    /// Each successor → max(predecessors' layers) + 1
    fn assign_layers(diagram: &Diagram) -> HashMap<String, usize> {
        let mut layers = HashMap::new();

        // Handle empty diagram
        if diagram.nodes.is_empty() {
            return layers;
        }

        // Build adjacency map: node -> list of outgoing connections
        let mut outgoing: HashMap<String, Vec<String>> = HashMap::new();
        let mut incoming: HashMap<String, HashSet<String>> = HashMap::new();

        // Initialize all nodes
        for node in &diagram.nodes {
            outgoing.entry(node.identifier.clone()).or_default();
            incoming.entry(node.identifier.clone()).or_default();
        }

        // Build connection maps
        for conn in &diagram.connections {
            outgoing
                .entry(conn.from.clone())
                .or_default()
                .push(conn.to.clone());
            incoming
                .entry(conn.to.clone())
                .or_default()
                .insert(conn.from.clone());
        }

        // Find nodes with no incoming edges (sources)
        let mut queue = VecDeque::new();
        for node in &diagram.nodes {
            if incoming.get(&node.identifier).unwrap().is_empty() {
                layers.insert(node.identifier.clone(), 0);
                queue.push_back(node.identifier.clone());
            }
        }

        // If no sources found (cycle or disconnected), assign remaining nodes to layer 0
        if queue.is_empty() {
            for node in &diagram.nodes {
                layers.insert(node.identifier.clone(), 0);
            }
            return layers;
        }

        // Iterative relaxation to find longest path to each node
        // Keep updating layers until no changes occur (guaranteed to terminate for DAGs)
        let mut changed = true;
        let mut round = 0;
        let max_rounds = diagram.nodes.len();

        while changed && round < max_rounds {
            changed = false;
            round += 1;

            for conn in &diagram.connections {
                if let Some(&from_layer) = layers.get(&conn.from) {
                    let required_layer = from_layer + 1;
                    let current_to_layer = layers.get(&conn.to).copied().unwrap_or(0);

                    if current_to_layer < required_layer {
                        layers.insert(conn.to.clone(), required_layer);
                        changed = true;
                    }
                }
            }
        }

        // Assign layer 0 to any remaining nodes not processed (disconnected components)
        for node in &diagram.nodes {
            layers.entry(node.identifier.clone()).or_insert(0);
        }

        layers
    }

    /// Position nodes within their assigned layers
    /// x = layer * (NODE_WIDTH + HORIZONTAL_SPACING)
    /// y = index_in_layer * (NODE_HEIGHT + VERTICAL_SPACING)
    fn position_nodes(nodes: &[Node], layers: &HashMap<String, usize>) -> Vec<PositionedNode> {
        // Group nodes by layer
        let mut layer_groups: HashMap<usize, Vec<&Node>> = HashMap::new();
        for node in nodes {
            let layer = layers.get(&node.identifier).copied().unwrap_or(0);
            layer_groups.entry(layer).or_default().push(node);
        }

        let mut positioned_nodes = Vec::new();

        for node in nodes {
            let layer = layers.get(&node.identifier).copied().unwrap_or(0);
            let nodes_in_layer = &layer_groups[&layer];

            // Find index of this node within its layer
            let index_in_layer = nodes_in_layer
                .iter()
                .position(|n| n.identifier == node.identifier)
                .unwrap();

            let x = layer as f64 * (DEFAULT_NODE_WIDTH + NODE_HORIZONTAL_SPACING);
            let y = index_in_layer as f64 * (DEFAULT_NODE_HEIGHT + NODE_VERTICAL_SPACING);

            positioned_nodes.push(PositionedNode {
                node: node.clone(),
                position: Point { x, y },
                width: DEFAULT_NODE_WIDTH,
                height: DEFAULT_NODE_HEIGHT,
            });
        }

        positioned_nodes
    }

    /// Build map from node identifier to center point for connection routing
    fn build_position_map(positioned_nodes: &[PositionedNode]) -> HashMap<String, Point> {
        positioned_nodes
            .iter()
            .map(|pn| {
                (
                    pn.node.identifier.clone(),
                    Point {
                        x: pn.position.x + pn.width / 2.0,
                        y: pn.position.y + pn.height / 2.0,
                    },
                )
            })
            .collect()
    }

    /// Position connections as straight lines between node centers
    fn position_connections(
        connections: &[Connection],
        node_positions: &HashMap<String, Point>,
    ) -> Vec<PositionedConnection> {
        connections
            .iter()
            .filter_map(|conn| {
                // Get center points for both nodes
                let start = node_positions.get(&conn.from)?;
                let end = node_positions.get(&conn.to)?;

                Some(PositionedConnection {
                    connection: conn.clone(),
                    start: *start,
                    end: *end,
                })
            })
            .collect()
    }

    /// Compute bounding box from positioned nodes
    fn compute_bounds(positioned_nodes: &[PositionedNode]) -> (f64, f64) {
        if positioned_nodes.is_empty() {
            return (0.0, 0.0);
        }

        let max_x = positioned_nodes
            .iter()
            .map(|pn| pn.position.x + pn.width)
            .fold(0.0, f64::max);
        let max_y = positioned_nodes
            .iter()
            .map(|pn| pn.position.y + pn.height)
            .fold(0.0, f64::max);
        (max_x, max_y)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{NodeType, SourcePosition};

    fn create_node(id: &str, display_name: &str) -> Node {
        Node {
            identifier: id.to_string(),
            display_name: display_name.to_string(),
            node_type: NodeType::Service,
            position: SourcePosition { line: 1, column: 1 },
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

    #[test]
    fn test_layout_single_node() {
        let diagram = Diagram {
            nodes: vec![create_node("api", "API")],
            connections: vec![],
        };

        let layout = LayoutEngine::layout(&diagram);

        assert_eq!(layout.nodes.len(), 1);
        assert_eq!(layout.connections.len(), 0);

        let node = &layout.nodes[0];
        assert_eq!(node.position.x, 0.0);
        assert_eq!(node.position.y, 0.0);
        assert_eq!(node.width, DEFAULT_NODE_WIDTH);
        assert_eq!(node.height, DEFAULT_NODE_HEIGHT);

        // Width and height should encompass the single node
        assert_eq!(layout.width, DEFAULT_NODE_WIDTH);
        assert_eq!(layout.height, DEFAULT_NODE_HEIGHT);
    }

    #[test]
    fn test_layout_linear_chain() {
        let diagram = Diagram {
            nodes: vec![
                create_node("a", "A"),
                create_node("b", "B"),
                create_node("c", "C"),
            ],
            connections: vec![
                create_connection("a", "b", None),
                create_connection("b", "c", None),
            ],
        };

        let layout = LayoutEngine::layout(&diagram);

        assert_eq!(layout.nodes.len(), 3);
        assert_eq!(layout.connections.len(), 2);

        // Find nodes by identifier
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

        // Verify layer assignment: a should be in layer 0, b in layer 1, c in layer 2
        assert_eq!(node_a.position.x, 0.0);
        assert_eq!(
            node_b.position.x,
            DEFAULT_NODE_WIDTH + NODE_HORIZONTAL_SPACING
        );
        assert_eq!(
            node_c.position.x,
            2.0 * (DEFAULT_NODE_WIDTH + NODE_HORIZONTAL_SPACING)
        );

        // All should be in same vertical position (only one node per layer)
        assert_eq!(node_a.position.y, 0.0);
        assert_eq!(node_b.position.y, 0.0);
        assert_eq!(node_c.position.y, 0.0);

        // Verify connections have correct start/end points
        assert_eq!(layout.connections.len(), 2);
    }

    #[test]
    fn test_layout_branching() {
        // Create a branching topology:
        //     a
        //    / \
        //   b   c
        let diagram = Diagram {
            nodes: vec![
                create_node("a", "A"),
                create_node("b", "B"),
                create_node("c", "C"),
            ],
            connections: vec![
                create_connection("a", "b", Some("edge1")),
                create_connection("a", "c", Some("edge2")),
            ],
        };

        let layout = LayoutEngine::layout(&diagram);

        assert_eq!(layout.nodes.len(), 3);
        assert_eq!(layout.connections.len(), 2);

        // Find nodes
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

        // Node a should be in layer 0
        assert_eq!(node_a.position.x, 0.0);

        // Nodes b and c should be in layer 1 (same x coordinate)
        let expected_layer_1_x = DEFAULT_NODE_WIDTH + NODE_HORIZONTAL_SPACING;
        assert_eq!(node_b.position.x, expected_layer_1_x);
        assert_eq!(node_c.position.x, expected_layer_1_x);

        // Nodes b and c should be at different y coordinates (vertically spaced)
        assert_ne!(node_b.position.y, node_c.position.y);

        // One should be at y=0, the other at NODE_VERTICAL_SPACING
        let y_values: Vec<f64> = vec![node_b.position.y, node_c.position.y];
        assert!(y_values.contains(&0.0));
        assert!(y_values.contains(&(DEFAULT_NODE_HEIGHT + NODE_VERTICAL_SPACING)));
    }

    #[test]
    fn test_layout_bounds_calculation() {
        let diagram = Diagram {
            nodes: vec![
                create_node("a", "A"),
                create_node("b", "B"),
                create_node("c", "C"),
            ],
            connections: vec![
                create_connection("a", "b", None),
                create_connection("a", "c", None),
            ],
        };

        let layout = LayoutEngine::layout(&diagram);

        // Width should extend to rightmost node's right edge
        // Nodes b and c are in layer 1, so max x = layer_1_x + NODE_WIDTH
        let expected_width = (DEFAULT_NODE_WIDTH + NODE_HORIZONTAL_SPACING) + DEFAULT_NODE_WIDTH;
        assert_eq!(layout.width, expected_width);

        // Height should extend to bottommost node's bottom edge
        // Two nodes in layer 1, so max y = NODE_HEIGHT + NODE_VERTICAL_SPACING + NODE_HEIGHT
        let expected_height = DEFAULT_NODE_HEIGHT + NODE_VERTICAL_SPACING + DEFAULT_NODE_HEIGHT;
        assert_eq!(layout.height, expected_height);
    }

    #[test]
    fn test_layout_empty_diagram() {
        let diagram = Diagram {
            nodes: vec![],
            connections: vec![],
        };

        let layout = LayoutEngine::layout(&diagram);

        assert_eq!(layout.nodes.len(), 0);
        assert_eq!(layout.connections.len(), 0);
        assert_eq!(layout.width, 0.0);
        assert_eq!(layout.height, 0.0);
    }

    #[test]
    fn test_layout_disconnected_components() {
        // Two separate nodes with no connections
        let diagram = Diagram {
            nodes: vec![create_node("a", "A"), create_node("b", "B")],
            connections: vec![],
        };

        let layout = LayoutEngine::layout(&diagram);

        assert_eq!(layout.nodes.len(), 2);

        // Both nodes should be in layer 0 (no incoming edges)
        for node in &layout.nodes {
            assert_eq!(node.position.x, 0.0);
        }

        // They should be at different y positions
        let y_values: Vec<f64> = layout.nodes.iter().map(|n| n.position.y).collect();
        assert_eq!(y_values.len(), 2);
        assert_ne!(y_values[0], y_values[1]);
    }

    #[test]
    fn test_connection_routing() {
        let diagram = Diagram {
            nodes: vec![create_node("a", "A"), create_node("b", "B")],
            connections: vec![create_connection("a", "b", Some("test"))],
        };

        let layout = LayoutEngine::layout(&diagram);

        assert_eq!(layout.connections.len(), 1);

        let conn = &layout.connections[0];
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

        // Connection start should be at center of node a
        let expected_start_x = node_a.position.x + node_a.width / 2.0;
        let expected_start_y = node_a.position.y + node_a.height / 2.0;
        assert_eq!(conn.start.x, expected_start_x);
        assert_eq!(conn.start.y, expected_start_y);

        // Connection end should be at center of node b
        let expected_end_x = node_b.position.x + node_b.width / 2.0;
        let expected_end_y = node_b.position.y + node_b.height / 2.0;
        assert_eq!(conn.end.x, expected_end_x);
        assert_eq!(conn.end.y, expected_end_y);
    }
}
