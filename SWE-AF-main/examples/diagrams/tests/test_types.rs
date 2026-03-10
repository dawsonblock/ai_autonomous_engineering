use diagrams::types::*;

// Test 1: Type instantiation works correctly
#[test]
fn test_source_position_instantiation() {
    let pos = SourcePosition { line: 1, column: 5 };
    assert_eq!(pos.line, 1);
    assert_eq!(pos.column, 5);
}

#[test]
fn test_node_type_instantiation() {
    let service = NodeType::Service;
    let database = NodeType::Database;
    let external = NodeType::External;
    let queue = NodeType::Queue;

    // Just verify they can be created
    assert!(matches!(service, NodeType::Service));
    assert!(matches!(database, NodeType::Database));
    assert!(matches!(external, NodeType::External));
    assert!(matches!(queue, NodeType::Queue));
}

#[test]
fn test_node_instantiation() {
    let node = Node {
        identifier: String::from("api"),
        display_name: String::from("API Gateway"),
        node_type: NodeType::Service,
        position: SourcePosition { line: 1, column: 1 },
    };

    assert_eq!(node.identifier, "api");
    assert_eq!(node.display_name, "API Gateway");
    assert_eq!(node.node_type, NodeType::Service);
    assert_eq!(node.position.line, 1);
}

#[test]
fn test_connection_instantiation() {
    let conn = Connection {
        from: String::from("api"),
        to: String::from("db"),
        label: Some(String::from("SQL query")),
        position: SourcePosition { line: 3, column: 1 },
    };

    assert_eq!(conn.from, "api");
    assert_eq!(conn.to, "db");
    assert_eq!(conn.label, Some(String::from("SQL query")));
    assert_eq!(conn.position.line, 3);
}

#[test]
fn test_connection_without_label() {
    let conn = Connection {
        from: String::from("api"),
        to: String::from("db"),
        label: None,
        position: SourcePosition { line: 3, column: 1 },
    };

    assert_eq!(conn.label, None);
}

#[test]
fn test_diagram_instantiation() {
    let node1 = Node {
        identifier: String::from("api"),
        display_name: String::from("API"),
        node_type: NodeType::Service,
        position: SourcePosition { line: 1, column: 1 },
    };

    let node2 = Node {
        identifier: String::from("db"),
        display_name: String::from("Database"),
        node_type: NodeType::Database,
        position: SourcePosition { line: 2, column: 1 },
    };

    let conn = Connection {
        from: String::from("api"),
        to: String::from("db"),
        label: None,
        position: SourcePosition { line: 3, column: 1 },
    };

    let diagram = Diagram {
        nodes: vec![node1, node2],
        connections: vec![conn],
    };

    assert_eq!(diagram.nodes.len(), 2);
    assert_eq!(diagram.connections.len(), 1);
}

#[test]
fn test_point_instantiation() {
    let point = Point { x: 10.5, y: 20.3 };
    assert_eq!(point.x, 10.5);
    assert_eq!(point.y, 20.3);
}

#[test]
fn test_positioned_node_instantiation() {
    let node = Node {
        identifier: String::from("api"),
        display_name: String::from("API"),
        node_type: NodeType::Service,
        position: SourcePosition { line: 1, column: 1 },
    };

    let positioned = PositionedNode {
        node,
        position: Point { x: 0.0, y: 0.0 },
        width: 120.0,
        height: 60.0,
    };

    assert_eq!(positioned.position.x, 0.0);
    assert_eq!(positioned.width, 120.0);
    assert_eq!(positioned.height, 60.0);
}

#[test]
fn test_positioned_connection_instantiation() {
    let conn = Connection {
        from: String::from("api"),
        to: String::from("db"),
        label: None,
        position: SourcePosition { line: 3, column: 1 },
    };

    let positioned_conn = PositionedConnection {
        connection: conn,
        start: Point { x: 60.0, y: 30.0 },
        end: Point { x: 180.0, y: 30.0 },
    };

    assert_eq!(positioned_conn.start.x, 60.0);
    assert_eq!(positioned_conn.end.x, 180.0);
}

#[test]
fn test_layout_diagram_instantiation() {
    let node = Node {
        identifier: String::from("api"),
        display_name: String::from("API"),
        node_type: NodeType::Service,
        position: SourcePosition { line: 1, column: 1 },
    };

    let positioned = PositionedNode {
        node,
        position: Point { x: 0.0, y: 0.0 },
        width: 120.0,
        height: 60.0,
    };

    let layout = LayoutDiagram {
        nodes: vec![positioned],
        connections: vec![],
        width: 200.0,
        height: 160.0,
    };

    assert_eq!(layout.nodes.len(), 1);
    assert_eq!(layout.width, 200.0);
    assert_eq!(layout.height, 160.0);
}

// Test 2: Debug formatting produces expected output
#[test]
fn test_source_position_debug() {
    let pos = SourcePosition {
        line: 5,
        column: 10,
    };
    let debug_str = format!("{:?}", pos);
    assert!(debug_str.contains("SourcePosition"));
    assert!(debug_str.contains("line"));
    assert!(debug_str.contains("5"));
    assert!(debug_str.contains("column"));
    assert!(debug_str.contains("10"));
}

#[test]
fn test_node_type_debug() {
    let service = NodeType::Service;
    assert_eq!(format!("{:?}", service), "Service");

    let database = NodeType::Database;
    assert_eq!(format!("{:?}", database), "Database");

    let external = NodeType::External;
    assert_eq!(format!("{:?}", external), "External");

    let queue = NodeType::Queue;
    assert_eq!(format!("{:?}", queue), "Queue");
}

#[test]
fn test_node_debug() {
    let node = Node {
        identifier: String::from("api"),
        display_name: String::from("API"),
        node_type: NodeType::Service,
        position: SourcePosition { line: 1, column: 1 },
    };

    let debug_str = format!("{:?}", node);
    assert!(debug_str.contains("Node"));
    assert!(debug_str.contains("api"));
    assert!(debug_str.contains("API"));
}

#[test]
fn test_point_debug() {
    let point = Point { x: 10.0, y: 20.0 };
    let debug_str = format!("{:?}", point);
    assert!(debug_str.contains("Point"));
    assert!(debug_str.contains("10"));
    assert!(debug_str.contains("20"));
}

// Test 3: Clone trait works
#[test]
fn test_source_position_clone() {
    let pos1 = SourcePosition { line: 1, column: 5 };
    let pos2 = pos1;
    assert_eq!(pos1.line, pos2.line);
    assert_eq!(pos1.column, pos2.column);
}

#[test]
fn test_node_type_clone() {
    let type1 = NodeType::Database;
    let type2 = type1;
    assert_eq!(type1, type2);
}

#[test]
fn test_node_clone() {
    let node1 = Node {
        identifier: String::from("api"),
        display_name: String::from("API"),
        node_type: NodeType::Service,
        position: SourcePosition { line: 1, column: 1 },
    };

    let node2 = node1.clone();
    assert_eq!(node1.identifier, node2.identifier);
    assert_eq!(node1.display_name, node2.display_name);
    assert_eq!(node1.node_type, node2.node_type);
}

#[test]
fn test_connection_clone() {
    let conn1 = Connection {
        from: String::from("api"),
        to: String::from("db"),
        label: Some(String::from("SQL")),
        position: SourcePosition { line: 3, column: 1 },
    };

    let conn2 = conn1.clone();
    assert_eq!(conn1.from, conn2.from);
    assert_eq!(conn1.to, conn2.to);
    assert_eq!(conn1.label, conn2.label);
}

#[test]
fn test_diagram_clone() {
    let node = Node {
        identifier: String::from("api"),
        display_name: String::from("API"),
        node_type: NodeType::Service,
        position: SourcePosition { line: 1, column: 1 },
    };

    let diagram1 = Diagram {
        nodes: vec![node],
        connections: vec![],
    };

    let diagram2 = diagram1.clone();
    assert_eq!(diagram1.nodes.len(), diagram2.nodes.len());
    assert_eq!(diagram1.connections.len(), diagram2.connections.len());
}

#[test]
fn test_point_clone() {
    let point1 = Point { x: 10.0, y: 20.0 };
    let point2 = point1;
    assert_eq!(point1.x, point2.x);
    assert_eq!(point1.y, point2.y);
}

#[test]
fn test_positioned_node_clone() {
    let node = Node {
        identifier: String::from("api"),
        display_name: String::from("API"),
        node_type: NodeType::Service,
        position: SourcePosition { line: 1, column: 1 },
    };

    let positioned1 = PositionedNode {
        node,
        position: Point { x: 0.0, y: 0.0 },
        width: 120.0,
        height: 60.0,
    };

    let positioned2 = positioned1.clone();
    assert_eq!(positioned1.width, positioned2.width);
    assert_eq!(positioned1.height, positioned2.height);
}

#[test]
fn test_layout_diagram_clone() {
    let layout1 = LayoutDiagram {
        nodes: vec![],
        connections: vec![],
        width: 200.0,
        height: 160.0,
    };

    let layout2 = layout1.clone();
    assert_eq!(layout1.width, layout2.width);
    assert_eq!(layout1.height, layout2.height);
}

// Test 4: PartialEq trait works
#[test]
fn test_source_position_partial_eq() {
    let pos1 = SourcePosition { line: 1, column: 5 };
    let pos2 = SourcePosition { line: 1, column: 5 };
    let pos3 = SourcePosition { line: 2, column: 5 };

    assert_eq!(pos1, pos2);
    assert_ne!(pos1, pos3);
}

#[test]
fn test_node_type_partial_eq() {
    let type1 = NodeType::Service;
    let type2 = NodeType::Service;
    let type3 = NodeType::Database;

    assert_eq!(type1, type2);
    assert_ne!(type1, type3);
}

#[test]
fn test_node_partial_eq() {
    let node1 = Node {
        identifier: String::from("api"),
        display_name: String::from("API"),
        node_type: NodeType::Service,
        position: SourcePosition { line: 1, column: 1 },
    };

    let node2 = Node {
        identifier: String::from("api"),
        display_name: String::from("API"),
        node_type: NodeType::Service,
        position: SourcePosition { line: 1, column: 1 },
    };

    let node3 = Node {
        identifier: String::from("db"),
        display_name: String::from("DB"),
        node_type: NodeType::Database,
        position: SourcePosition { line: 2, column: 1 },
    };

    assert_eq!(node1, node2);
    assert_ne!(node1, node3);
}

#[test]
fn test_connection_partial_eq() {
    let conn1 = Connection {
        from: String::from("api"),
        to: String::from("db"),
        label: Some(String::from("SQL")),
        position: SourcePosition { line: 3, column: 1 },
    };

    let conn2 = Connection {
        from: String::from("api"),
        to: String::from("db"),
        label: Some(String::from("SQL")),
        position: SourcePosition { line: 3, column: 1 },
    };

    let conn3 = Connection {
        from: String::from("api"),
        to: String::from("cache"),
        label: None,
        position: SourcePosition { line: 4, column: 1 },
    };

    assert_eq!(conn1, conn2);
    assert_ne!(conn1, conn3);
}

#[test]
fn test_diagram_partial_eq() {
    let node = Node {
        identifier: String::from("api"),
        display_name: String::from("API"),
        node_type: NodeType::Service,
        position: SourcePosition { line: 1, column: 1 },
    };

    let diagram1 = Diagram {
        nodes: vec![node.clone()],
        connections: vec![],
    };

    let diagram2 = Diagram {
        nodes: vec![node.clone()],
        connections: vec![],
    };

    let diagram3 = Diagram {
        nodes: vec![],
        connections: vec![],
    };

    assert_eq!(diagram1, diagram2);
    assert_ne!(diagram1, diagram3);
}

#[test]
fn test_point_partial_eq() {
    let point1 = Point { x: 10.0, y: 20.0 };
    let point2 = Point { x: 10.0, y: 20.0 };
    let point3 = Point { x: 15.0, y: 25.0 };

    assert_eq!(point1, point2);
    assert_ne!(point1, point3);
}

#[test]
fn test_positioned_node_partial_eq() {
    let node = Node {
        identifier: String::from("api"),
        display_name: String::from("API"),
        node_type: NodeType::Service,
        position: SourcePosition { line: 1, column: 1 },
    };

    let positioned1 = PositionedNode {
        node: node.clone(),
        position: Point { x: 0.0, y: 0.0 },
        width: 120.0,
        height: 60.0,
    };

    let positioned2 = PositionedNode {
        node: node.clone(),
        position: Point { x: 0.0, y: 0.0 },
        width: 120.0,
        height: 60.0,
    };

    let positioned3 = PositionedNode {
        node: node.clone(),
        position: Point { x: 100.0, y: 0.0 },
        width: 120.0,
        height: 60.0,
    };

    assert_eq!(positioned1, positioned2);
    assert_ne!(positioned1, positioned3);
}

#[test]
fn test_layout_diagram_partial_eq() {
    let layout1 = LayoutDiagram {
        nodes: vec![],
        connections: vec![],
        width: 200.0,
        height: 160.0,
    };

    let layout2 = LayoutDiagram {
        nodes: vec![],
        connections: vec![],
        width: 200.0,
        height: 160.0,
    };

    let layout3 = LayoutDiagram {
        nodes: vec![],
        connections: vec![],
        width: 300.0,
        height: 260.0,
    };

    assert_eq!(layout1, layout2);
    assert_ne!(layout1, layout3);
}

// Test 5: Constants have correct values
#[test]
fn test_default_node_width_constant() {
    assert_eq!(DEFAULT_NODE_WIDTH, 120.0);
}

#[test]
fn test_default_node_height_constant() {
    assert_eq!(DEFAULT_NODE_HEIGHT, 60.0);
}

#[test]
fn test_node_horizontal_spacing_constant() {
    assert_eq!(NODE_HORIZONTAL_SPACING, 80.0);
}

#[test]
fn test_node_vertical_spacing_constant() {
    assert_eq!(NODE_VERTICAL_SPACING, 100.0);
}

#[test]
fn test_svg_font_size_constant() {
    assert_eq!(SVG_FONT_SIZE, 14.0);
}

#[test]
fn test_svg_stroke_width_constant() {
    assert_eq!(SVG_STROKE_WIDTH, 2.0);
}

#[test]
fn test_ascii_node_padding_constant() {
    assert_eq!(ASCII_NODE_PADDING, 2);
}

#[test]
fn test_ascii_min_node_width_constant() {
    assert_eq!(ASCII_MIN_NODE_WIDTH, 10);
}

// Edge case tests
#[test]
fn test_empty_diagram() {
    let diagram = Diagram {
        nodes: vec![],
        connections: vec![],
    };

    assert_eq!(diagram.nodes.len(), 0);
    assert_eq!(diagram.connections.len(), 0);
}

#[test]
fn test_diagram_with_multiple_nodes() {
    let nodes: Vec<Node> = (0..5)
        .map(|i| Node {
            identifier: format!("node{}", i),
            display_name: format!("Node {}", i),
            node_type: NodeType::Service,
            position: SourcePosition {
                line: i + 1,
                column: 1,
            },
        })
        .collect();

    let diagram = Diagram {
        nodes,
        connections: vec![],
    };

    assert_eq!(diagram.nodes.len(), 5);
}

#[test]
fn test_diagram_with_multiple_connections() {
    let connections: Vec<Connection> = (0..3)
        .map(|i| Connection {
            from: format!("node{}", i),
            to: format!("node{}", i + 1),
            label: Some(format!("label{}", i)),
            position: SourcePosition {
                line: i + 10,
                column: 1,
            },
        })
        .collect();

    let diagram = Diagram {
        nodes: vec![],
        connections,
    };

    assert_eq!(diagram.connections.len(), 3);
}

#[test]
fn test_point_with_negative_coordinates() {
    let point = Point { x: -10.0, y: -20.0 };
    assert_eq!(point.x, -10.0);
    assert_eq!(point.y, -20.0);
}

#[test]
fn test_point_with_zero_coordinates() {
    let point = Point { x: 0.0, y: 0.0 };
    assert_eq!(point.x, 0.0);
    assert_eq!(point.y, 0.0);
}

#[test]
fn test_positioned_node_with_zero_dimensions() {
    let node = Node {
        identifier: String::from("api"),
        display_name: String::from("API"),
        node_type: NodeType::Service,
        position: SourcePosition { line: 1, column: 1 },
    };

    let positioned = PositionedNode {
        node,
        position: Point { x: 0.0, y: 0.0 },
        width: 0.0,
        height: 0.0,
    };

    assert_eq!(positioned.width, 0.0);
    assert_eq!(positioned.height, 0.0);
}

#[test]
fn test_layout_diagram_with_zero_dimensions() {
    let layout = LayoutDiagram {
        nodes: vec![],
        connections: vec![],
        width: 0.0,
        height: 0.0,
    };

    assert_eq!(layout.width, 0.0);
    assert_eq!(layout.height, 0.0);
}

#[test]
fn test_connection_label_with_empty_string() {
    let conn = Connection {
        from: String::from("api"),
        to: String::from("db"),
        label: Some(String::from("")),
        position: SourcePosition { line: 3, column: 1 },
    };

    assert_eq!(conn.label, Some(String::from("")));
}

#[test]
fn test_node_with_empty_identifier() {
    let node = Node {
        identifier: String::from(""),
        display_name: String::from("API"),
        node_type: NodeType::Service,
        position: SourcePosition { line: 1, column: 1 },
    };

    assert_eq!(node.identifier, "");
}

#[test]
fn test_node_with_empty_display_name() {
    let node = Node {
        identifier: String::from("api"),
        display_name: String::from(""),
        node_type: NodeType::Service,
        position: SourcePosition { line: 1, column: 1 },
    };

    assert_eq!(node.display_name, "");
}

#[test]
fn test_source_position_at_zero() {
    let pos = SourcePosition { line: 0, column: 0 };
    assert_eq!(pos.line, 0);
    assert_eq!(pos.column, 0);
}

#[test]
fn test_all_node_types_are_distinct() {
    let service = NodeType::Service;
    let database = NodeType::Database;
    let external = NodeType::External;
    let queue = NodeType::Queue;

    assert_ne!(service, database);
    assert_ne!(service, external);
    assert_ne!(service, queue);
    assert_ne!(database, external);
    assert_ne!(database, queue);
    assert_ne!(external, queue);
}
