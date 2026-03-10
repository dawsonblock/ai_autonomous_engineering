/// Position in source text (for error reporting).
///
/// Tracks line and column numbers in the input DSL file to provide
/// precise error messages pointing to the exact location of syntax
/// or semantic issues.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SourcePosition {
    /// Line number (1-indexed)
    pub line: usize,
    /// Column number (1-indexed)
    pub column: usize,
}

/// Node type determines visual rendering style.
///
/// Each node type is rendered with a distinct visual appearance:
/// - `Service`: Rounded rectangle (default)
/// - `Database`: Cylinder shape
/// - `External`: Rounded rectangle with distinct coloring
/// - `Queue`: Rounded rectangle with message queue styling
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NodeType {
    /// Service component (default node type)
    Service,
    /// Database or data store
    Database,
    /// External system or third-party service
    External,
    /// Message queue or event bus
    Queue,
}

/// A node in the architecture diagram.
///
/// Represents a component in the system architecture such as
/// a service, database, or external system.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Node {
    /// Unique identifier used in connections (e.g., "api")
    pub identifier: String,
    /// Display name shown in the diagram (e.g., "API Gateway")
    pub display_name: String,
    /// Visual rendering style for this node
    pub node_type: NodeType,
    /// Source location where this node was defined
    pub position: SourcePosition,
}

/// A directed connection between two nodes.
///
/// Represents a relationship or data flow from one component
/// to another in the architecture diagram.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Connection {
    /// Source node identifier
    pub from: String,
    /// Target node identifier
    pub to: String,
    /// Optional label describing the connection (e.g., "HTTP", "SQL query")
    pub label: Option<String>,
    /// Source location where this connection was defined
    pub position: SourcePosition,
}

/// Complete diagram Abstract Syntax Tree (AST).
///
/// Represents the parsed structure of a DSL file,
/// containing all nodes and connections defined in the input.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Diagram {
    /// All nodes in the diagram
    pub nodes: Vec<Node>,
    /// All connections between nodes
    pub connections: Vec<Connection>,
}

/// 2D coordinate for layout positioning.
///
/// Represents a point in the diagram's coordinate system,
/// with origin at top-left (0, 0).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Point {
    /// Horizontal position (pixels from left edge)
    pub x: f64,
    /// Vertical position (pixels from top edge)
    pub y: f64,
}

/// Positioned node after layout algorithm runs.
///
/// Combines node metadata with computed position and dimensions
/// for rendering.
#[derive(Debug, Clone, PartialEq)]
pub struct PositionedNode {
    /// The node with its metadata
    pub node: Node,
    /// Top-left corner position
    pub position: Point,
    /// Node width in pixels
    pub width: f64,
    /// Node height in pixels
    pub height: f64,
}

/// Positioned connection after layout algorithm runs.
///
/// Combines connection metadata with computed start and end points
/// for rendering.
#[derive(Debug, Clone, PartialEq)]
pub struct PositionedConnection {
    /// The connection with its metadata
    pub connection: Connection,
    /// Start point (center of source node)
    pub start: Point,
    /// End point (center of target node)
    pub end: Point,
}

/// Diagram with computed layout.
///
/// Result of the layout algorithm, containing nodes and connections
/// with their final positions and the overall diagram dimensions.
#[derive(Debug, Clone, PartialEq)]
pub struct LayoutDiagram {
    /// Nodes with computed positions and dimensions
    pub nodes: Vec<PositionedNode>,
    /// Connections with computed start and end points
    pub connections: Vec<PositionedConnection>,
    /// Total diagram width in pixels
    pub width: f64,
    /// Total diagram height in pixels
    pub height: f64,
}

/// Default node width in pixels for layout calculations
pub const DEFAULT_NODE_WIDTH: f64 = 120.0;

/// Default node height in pixels for layout calculations
pub const DEFAULT_NODE_HEIGHT: f64 = 60.0;

/// Horizontal spacing between node layers in pixels
pub const NODE_HORIZONTAL_SPACING: f64 = 80.0;

/// Vertical spacing between nodes in the same layer in pixels
pub const NODE_VERTICAL_SPACING: f64 = 100.0;

/// Font size for SVG text elements
pub const SVG_FONT_SIZE: f64 = 14.0;

/// Stroke width for SVG shapes and lines
pub const SVG_STROKE_WIDTH: f64 = 2.0;

/// Padding around text in ASCII nodes (characters)
pub const ASCII_NODE_PADDING: usize = 2;

/// Minimum node width in ASCII rendering (characters)
pub const ASCII_MIN_NODE_WIDTH: usize = 10;
