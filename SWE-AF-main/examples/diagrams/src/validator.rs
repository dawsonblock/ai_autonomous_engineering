use crate::error::{DiagramError, SemanticError};
use crate::types::{Diagram, SourcePosition};
use std::collections::{HashMap, HashSet};

/// Validator performs semantic validation on Diagram AST.
///
/// Ensures that diagrams are logically valid by checking for:
/// - All connection endpoints reference defined nodes
/// - No duplicate node identifiers
/// - No self-referencing connections
pub struct Validator;

impl Validator {
    /// Validate a diagram for semantic correctness.
    ///
    /// Performs comprehensive semantic validation on a parsed diagram:
    /// - Ensures all connection endpoints reference defined nodes
    /// - Checks for duplicate node identifiers
    /// - Prevents self-referencing connections
    ///
    /// # Arguments
    ///
    /// * `diagram` - The parsed diagram AST to validate
    ///
    /// # Returns
    ///
    /// * `Ok(())` if the diagram is semantically valid
    /// * `Err(DiagramError::Semantic)` with details about the first validation failure
    pub fn validate(diagram: &Diagram) -> Result<(), DiagramError> {
        // Check for duplicate node identifiers
        Self::check_duplicate_nodes(diagram)?;

        // Build a set of valid node identifiers for efficient lookup
        let valid_nodes: HashSet<&str> = diagram
            .nodes
            .iter()
            .map(|node| node.identifier.as_str())
            .collect();

        // Check all connections
        for connection in &diagram.connections {
            // Check for self-connections
            if connection.from == connection.to {
                return Err(DiagramError::Semantic(SemanticError::SelfConnection {
                    identifier: connection.from.clone(),
                    position: connection.position,
                }));
            }

            // Check if source node exists
            if !valid_nodes.contains(connection.from.as_str()) {
                return Err(DiagramError::Semantic(SemanticError::UndefinedNode {
                    identifier: connection.from.clone(),
                    position: connection.position,
                }));
            }

            // Check if target node exists
            if !valid_nodes.contains(connection.to.as_str()) {
                return Err(DiagramError::Semantic(SemanticError::UndefinedNode {
                    identifier: connection.to.clone(),
                    position: connection.position,
                }));
            }
        }

        Ok(())
    }

    /// Check for duplicate node identifiers
    fn check_duplicate_nodes(diagram: &Diagram) -> Result<(), DiagramError> {
        let mut seen: HashMap<&str, SourcePosition> = HashMap::new();

        for node in &diagram.nodes {
            if let Some(&first_position) = seen.get(node.identifier.as_str()) {
                return Err(DiagramError::Semantic(SemanticError::DuplicateNode {
                    identifier: node.identifier.clone(),
                    first_position,
                    second_position: node.position,
                }));
            }
            seen.insert(&node.identifier, node.position);
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{Connection, Node, NodeType};

    #[test]
    fn test_valid_diagram_passes() {
        let diagram = Diagram {
            nodes: vec![
                Node {
                    identifier: "api".to_string(),
                    display_name: "API Gateway".to_string(),
                    node_type: NodeType::Service,
                    position: SourcePosition { line: 1, column: 1 },
                },
                Node {
                    identifier: "db".to_string(),
                    display_name: "Database".to_string(),
                    node_type: NodeType::Database,
                    position: SourcePosition { line: 2, column: 1 },
                },
            ],
            connections: vec![Connection {
                from: "api".to_string(),
                to: "db".to_string(),
                label: Some("SQL query".to_string()),
                position: SourcePosition { line: 3, column: 1 },
            }],
        };

        let result = Validator::validate(&diagram);
        assert!(result.is_ok());
    }

    #[test]
    fn test_undefined_source_node() {
        let diagram = Diagram {
            nodes: vec![Node {
                identifier: "db".to_string(),
                display_name: "Database".to_string(),
                node_type: NodeType::Database,
                position: SourcePosition { line: 1, column: 1 },
            }],
            connections: vec![Connection {
                from: "api".to_string(),
                to: "db".to_string(),
                label: None,
                position: SourcePosition { line: 2, column: 1 },
            }],
        };

        let result = Validator::validate(&diagram);
        assert!(result.is_err());

        match result {
            Err(DiagramError::Semantic(SemanticError::UndefinedNode {
                identifier,
                position,
            })) => {
                assert_eq!(identifier, "api");
                assert_eq!(position.line, 2);
            }
            _ => panic!("Expected SemanticError::UndefinedNode for source"),
        }
    }

    #[test]
    fn test_undefined_target_node() {
        let diagram = Diagram {
            nodes: vec![Node {
                identifier: "api".to_string(),
                display_name: "API Gateway".to_string(),
                node_type: NodeType::Service,
                position: SourcePosition { line: 1, column: 1 },
            }],
            connections: vec![Connection {
                from: "api".to_string(),
                to: "db".to_string(),
                label: None,
                position: SourcePosition { line: 2, column: 1 },
            }],
        };

        let result = Validator::validate(&diagram);
        assert!(result.is_err());

        match result {
            Err(DiagramError::Semantic(SemanticError::UndefinedNode {
                identifier,
                position,
            })) => {
                assert_eq!(identifier, "db");
                assert_eq!(position.line, 2);
            }
            _ => panic!("Expected SemanticError::UndefinedNode for target"),
        }
    }

    #[test]
    fn test_duplicate_node_identifier() {
        let diagram = Diagram {
            nodes: vec![
                Node {
                    identifier: "api".to_string(),
                    display_name: "API Gateway".to_string(),
                    node_type: NodeType::Service,
                    position: SourcePosition { line: 1, column: 1 },
                },
                Node {
                    identifier: "api".to_string(),
                    display_name: "Another API".to_string(),
                    node_type: NodeType::Service,
                    position: SourcePosition { line: 5, column: 1 },
                },
            ],
            connections: vec![],
        };

        let result = Validator::validate(&diagram);
        assert!(result.is_err());

        match result {
            Err(DiagramError::Semantic(SemanticError::DuplicateNode {
                identifier,
                first_position,
                second_position,
            })) => {
                assert_eq!(identifier, "api");
                assert_eq!(first_position.line, 1);
                assert_eq!(second_position.line, 5);
            }
            _ => panic!("Expected SemanticError::DuplicateNode"),
        }
    }

    #[test]
    fn test_self_connection() {
        let diagram = Diagram {
            nodes: vec![Node {
                identifier: "api".to_string(),
                display_name: "API Gateway".to_string(),
                node_type: NodeType::Service,
                position: SourcePosition { line: 1, column: 1 },
            }],
            connections: vec![Connection {
                from: "api".to_string(),
                to: "api".to_string(),
                label: Some("self loop".to_string()),
                position: SourcePosition { line: 2, column: 1 },
            }],
        };

        let result = Validator::validate(&diagram);
        assert!(result.is_err());

        match result {
            Err(DiagramError::Semantic(SemanticError::SelfConnection {
                identifier,
                position,
            })) => {
                assert_eq!(identifier, "api");
                assert_eq!(position.line, 2);
            }
            _ => panic!("Expected SemanticError::SelfConnection"),
        }
    }

    #[test]
    fn test_valid_diagram_with_multiple_nodes_and_connections() {
        let diagram = Diagram {
            nodes: vec![
                Node {
                    identifier: "api".to_string(),
                    display_name: "API Gateway".to_string(),
                    node_type: NodeType::Service,
                    position: SourcePosition { line: 1, column: 1 },
                },
                Node {
                    identifier: "db".to_string(),
                    display_name: "Database".to_string(),
                    node_type: NodeType::Database,
                    position: SourcePosition { line: 2, column: 1 },
                },
                Node {
                    identifier: "cache".to_string(),
                    display_name: "Cache".to_string(),
                    node_type: NodeType::Database,
                    position: SourcePosition { line: 3, column: 1 },
                },
            ],
            connections: vec![
                Connection {
                    from: "api".to_string(),
                    to: "db".to_string(),
                    label: Some("SQL query".to_string()),
                    position: SourcePosition { line: 4, column: 1 },
                },
                Connection {
                    from: "api".to_string(),
                    to: "cache".to_string(),
                    label: Some("GET".to_string()),
                    position: SourcePosition { line: 5, column: 1 },
                },
            ],
        };

        let result = Validator::validate(&diagram);
        assert!(result.is_ok());
    }

    #[test]
    fn test_empty_diagram() {
        let diagram = Diagram {
            nodes: vec![],
            connections: vec![],
        };

        let result = Validator::validate(&diagram);
        assert!(result.is_ok());
    }

    #[test]
    fn test_diagram_with_only_nodes() {
        let diagram = Diagram {
            nodes: vec![
                Node {
                    identifier: "api".to_string(),
                    display_name: "API Gateway".to_string(),
                    node_type: NodeType::Service,
                    position: SourcePosition { line: 1, column: 1 },
                },
                Node {
                    identifier: "db".to_string(),
                    display_name: "Database".to_string(),
                    node_type: NodeType::Database,
                    position: SourcePosition { line: 2, column: 1 },
                },
            ],
            connections: vec![],
        };

        let result = Validator::validate(&diagram);
        assert!(result.is_ok());
    }

    #[test]
    fn test_multiple_connections_same_nodes() {
        let diagram = Diagram {
            nodes: vec![
                Node {
                    identifier: "api".to_string(),
                    display_name: "API Gateway".to_string(),
                    node_type: NodeType::Service,
                    position: SourcePosition { line: 1, column: 1 },
                },
                Node {
                    identifier: "db".to_string(),
                    display_name: "Database".to_string(),
                    node_type: NodeType::Database,
                    position: SourcePosition { line: 2, column: 1 },
                },
            ],
            connections: vec![
                Connection {
                    from: "api".to_string(),
                    to: "db".to_string(),
                    label: Some("write".to_string()),
                    position: SourcePosition { line: 3, column: 1 },
                },
                Connection {
                    from: "api".to_string(),
                    to: "db".to_string(),
                    label: Some("read".to_string()),
                    position: SourcePosition { line: 4, column: 1 },
                },
            ],
        };

        let result = Validator::validate(&diagram);
        assert!(result.is_ok());
    }

    #[test]
    fn test_bidirectional_connections() {
        let diagram = Diagram {
            nodes: vec![
                Node {
                    identifier: "api".to_string(),
                    display_name: "API Gateway".to_string(),
                    node_type: NodeType::Service,
                    position: SourcePosition { line: 1, column: 1 },
                },
                Node {
                    identifier: "db".to_string(),
                    display_name: "Database".to_string(),
                    node_type: NodeType::Database,
                    position: SourcePosition { line: 2, column: 1 },
                },
            ],
            connections: vec![
                Connection {
                    from: "api".to_string(),
                    to: "db".to_string(),
                    label: Some("query".to_string()),
                    position: SourcePosition { line: 3, column: 1 },
                },
                Connection {
                    from: "db".to_string(),
                    to: "api".to_string(),
                    label: Some("response".to_string()),
                    position: SourcePosition { line: 4, column: 1 },
                },
            ],
        };

        let result = Validator::validate(&diagram);
        assert!(result.is_ok());
    }

    #[test]
    fn test_all_node_types() {
        let diagram = Diagram {
            nodes: vec![
                Node {
                    identifier: "svc".to_string(),
                    display_name: "Service".to_string(),
                    node_type: NodeType::Service,
                    position: SourcePosition { line: 1, column: 1 },
                },
                Node {
                    identifier: "db".to_string(),
                    display_name: "Database".to_string(),
                    node_type: NodeType::Database,
                    position: SourcePosition { line: 2, column: 1 },
                },
                Node {
                    identifier: "ext".to_string(),
                    display_name: "External".to_string(),
                    node_type: NodeType::External,
                    position: SourcePosition { line: 3, column: 1 },
                },
                Node {
                    identifier: "q".to_string(),
                    display_name: "Queue".to_string(),
                    node_type: NodeType::Queue,
                    position: SourcePosition { line: 4, column: 1 },
                },
            ],
            connections: vec![
                Connection {
                    from: "svc".to_string(),
                    to: "db".to_string(),
                    label: None,
                    position: SourcePosition { line: 5, column: 1 },
                },
                Connection {
                    from: "svc".to_string(),
                    to: "ext".to_string(),
                    label: None,
                    position: SourcePosition { line: 6, column: 1 },
                },
                Connection {
                    from: "svc".to_string(),
                    to: "q".to_string(),
                    label: None,
                    position: SourcePosition { line: 7, column: 1 },
                },
            ],
        };

        let result = Validator::validate(&diagram);
        assert!(result.is_ok());
    }

    #[test]
    fn test_case_sensitive_identifiers() {
        let diagram = Diagram {
            nodes: vec![
                Node {
                    identifier: "API".to_string(),
                    display_name: "API".to_string(),
                    node_type: NodeType::Service,
                    position: SourcePosition { line: 1, column: 1 },
                },
                Node {
                    identifier: "api".to_string(),
                    display_name: "api".to_string(),
                    node_type: NodeType::Service,
                    position: SourcePosition { line: 2, column: 1 },
                },
            ],
            connections: vec![],
        };

        let result = Validator::validate(&diagram);
        // Both "API" and "api" should be treated as different identifiers
        assert!(result.is_ok());
    }
}
