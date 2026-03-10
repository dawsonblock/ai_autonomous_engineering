use crate::types::SourcePosition;
use std::fmt;
use std::io;

/// Top-level error type for the entire application.
///
/// Categorizes errors into syntax (parsing), semantic (validation),
/// and I/O (file operations) errors. Each variant maps to a specific
/// exit code as defined in the PRD.
#[derive(Debug)]
pub enum DiagramError {
    /// Syntax error during lexing or parsing (exit code 1)
    Syntax(SyntaxError),
    /// Semantic error during validation (exit code 2)
    Semantic(SemanticError),
    /// I/O error during file operations (exit code 3)
    Io(IoError),
}

/// Syntax error during lexing or parsing.
///
/// Indicates invalid DSL syntax such as unexpected tokens,
/// malformed statements, or unterminated strings.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SyntaxError {
    /// Error description
    pub message: String,
    /// Location in source file where the error occurred
    pub position: SourcePosition,
}

/// Semantic error during validation.
///
/// Indicates logically invalid diagrams such as undefined nodes
/// in connections, duplicate node identifiers, or self-referencing
/// connections.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SemanticError {
    /// Connection references a node that hasn't been defined
    UndefinedNode {
        /// The undefined node identifier
        identifier: String,
        /// Location where the undefined node was referenced
        position: SourcePosition,
    },
    /// Multiple nodes defined with the same identifier
    DuplicateNode {
        /// The duplicate node identifier
        identifier: String,
        /// Location of first definition
        first_position: SourcePosition,
        /// Location of duplicate definition
        second_position: SourcePosition,
    },
    /// A node attempts to connect to itself
    SelfConnection {
        /// The node identifier
        identifier: String,
        /// Location of the self-connection
        position: SourcePosition,
    },
}

/// I/O error during file operations.
///
/// Wraps standard I/O errors with additional context about
/// what file operation failed.
#[derive(Debug)]
pub struct IoError {
    /// Error description including file path
    pub message: String,
    /// Original I/O error if available
    #[allow(dead_code)]
    pub source: Option<io::Error>,
}

impl DiagramError {
    /// Map error to exit code as specified in PRD
    pub fn exit_code(&self) -> i32 {
        match self {
            DiagramError::Syntax(_) => 1,
            DiagramError::Semantic(_) => 2,
            DiagramError::Io(_) => 3,
        }
    }

    /// Format error for human-readable output
    pub fn format_detailed(&self) -> String {
        match self {
            DiagramError::Syntax(e) => format!(
                "Syntax error at line {}, column {}: {}",
                e.position.line, e.position.column, e.message
            ),
            DiagramError::Semantic(e) => match e {
                SemanticError::UndefinedNode {
                    identifier,
                    position,
                } => format!(
                    "Semantic error at line {}, column {}: undefined node '{}'",
                    position.line, position.column, identifier
                ),
                SemanticError::DuplicateNode {
                    identifier,
                    first_position,
                    second_position,
                } => format!(
                    "Semantic error: node '{}' defined multiple times (first at line {}, duplicate at line {})",
                    identifier, first_position.line, second_position.line
                ),
                SemanticError::SelfConnection {
                    identifier,
                    position,
                } => format!(
                    "Semantic error at line {}, column {}: node '{}' cannot connect to itself",
                    position.line, position.column, identifier
                ),
            },
            DiagramError::Io(e) => format!("I/O error: {}", e.message),
        }
    }
}

impl fmt::Display for DiagramError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.format_detailed())
    }
}

impl std::error::Error for DiagramError {}

impl From<io::Error> for DiagramError {
    fn from(err: io::Error) -> Self {
        DiagramError::Io(IoError {
            message: err.to_string(),
            source: Some(err),
        })
    }
}

/// Type alias for Results with DiagramError as the error type.
///
/// Used throughout the application for operations that can fail
/// with syntax, semantic, or I/O errors.
pub type Result<T> = std::result::Result<T, DiagramError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_exit_code_syntax() {
        let error = DiagramError::Syntax(SyntaxError {
            message: "test".to_string(),
            position: SourcePosition { line: 1, column: 1 },
        });
        assert_eq!(error.exit_code(), 1);
    }

    #[test]
    fn test_exit_code_semantic() {
        let error = DiagramError::Semantic(SemanticError::UndefinedNode {
            identifier: "test".to_string(),
            position: SourcePosition { line: 1, column: 1 },
        });
        assert_eq!(error.exit_code(), 2);
    }

    #[test]
    fn test_exit_code_io() {
        let error = DiagramError::Io(IoError {
            message: "test".to_string(),
            source: None,
        });
        assert_eq!(error.exit_code(), 3);
    }

    #[test]
    fn test_format_detailed_syntax() {
        let error = DiagramError::Syntax(SyntaxError {
            message: "unexpected token".to_string(),
            position: SourcePosition {
                line: 42,
                column: 15,
            },
        });
        let formatted = error.format_detailed();
        assert!(formatted.contains("Syntax error"));
        assert!(formatted.contains("line 42"));
        assert!(formatted.contains("column 15"));
        assert!(formatted.contains("unexpected token"));
    }

    #[test]
    fn test_format_detailed_semantic_undefined_node() {
        let error = DiagramError::Semantic(SemanticError::UndefinedNode {
            identifier: "node_abc".to_string(),
            position: SourcePosition {
                line: 10,
                column: 5,
            },
        });
        let formatted = error.format_detailed();
        assert!(formatted.contains("Semantic error"));
        assert!(formatted.contains("line 10"));
        assert!(formatted.contains("column 5"));
        assert!(formatted.contains("undefined node"));
        assert!(formatted.contains("node_abc"));
    }

    #[test]
    fn test_format_detailed_semantic_duplicate_node() {
        let error = DiagramError::Semantic(SemanticError::DuplicateNode {
            identifier: "api".to_string(),
            first_position: SourcePosition { line: 5, column: 1 },
            second_position: SourcePosition {
                line: 12,
                column: 1,
            },
        });
        let formatted = error.format_detailed();
        assert!(formatted.contains("Semantic error"));
        assert!(formatted.contains("api"));
        assert!(formatted.contains("defined multiple times"));
        assert!(formatted.contains("first at line 5"));
        assert!(formatted.contains("duplicate at line 12"));
    }

    #[test]
    fn test_format_detailed_semantic_self_connection() {
        let error = DiagramError::Semantic(SemanticError::SelfConnection {
            identifier: "self_node".to_string(),
            position: SourcePosition {
                line: 8,
                column: 10,
            },
        });
        let formatted = error.format_detailed();
        assert!(formatted.contains("Semantic error"));
        assert!(formatted.contains("line 8"));
        assert!(formatted.contains("column 10"));
        assert!(formatted.contains("self_node"));
        assert!(formatted.contains("cannot connect to itself"));
    }

    #[test]
    fn test_format_detailed_io() {
        let error = DiagramError::Io(IoError {
            message: "file not found".to_string(),
            source: None,
        });
        let formatted = error.format_detailed();
        assert!(formatted.contains("I/O error"));
        assert!(formatted.contains("file not found"));
    }

    #[test]
    fn test_display_trait() {
        let error = DiagramError::Syntax(SyntaxError {
            message: "test error".to_string(),
            position: SourcePosition { line: 1, column: 1 },
        });
        let display_output = format!("{}", error);
        assert_eq!(display_output, error.format_detailed());
    }

    #[test]
    fn test_from_io_error() {
        let io_err = io::Error::new(io::ErrorKind::NotFound, "file not found");
        let diagram_error: DiagramError = io_err.into();

        assert_eq!(diagram_error.exit_code(), 3);

        let formatted = diagram_error.format_detailed();
        assert!(formatted.contains("I/O error"));
        assert!(formatted.contains("file not found"));
    }

    #[test]
    fn test_semantic_error_undefined_node_construct() {
        let error = SemanticError::UndefinedNode {
            identifier: "test_node".to_string(),
            position: SourcePosition { line: 1, column: 1 },
        };
        match error {
            SemanticError::UndefinedNode { identifier, .. } => {
                assert_eq!(identifier, "test_node");
            }
            _ => panic!("Expected UndefinedNode variant"),
        }
    }

    #[test]
    fn test_semantic_error_duplicate_node_construct() {
        let error = SemanticError::DuplicateNode {
            identifier: "dup".to_string(),
            first_position: SourcePosition { line: 1, column: 1 },
            second_position: SourcePosition { line: 2, column: 1 },
        };
        match error {
            SemanticError::DuplicateNode {
                identifier,
                first_position,
                second_position,
            } => {
                assert_eq!(identifier, "dup");
                assert_eq!(first_position.line, 1);
                assert_eq!(second_position.line, 2);
            }
            _ => panic!("Expected DuplicateNode variant"),
        }
    }

    #[test]
    fn test_semantic_error_self_connection_construct() {
        let error = SemanticError::SelfConnection {
            identifier: "self".to_string(),
            position: SourcePosition { line: 3, column: 5 },
        };
        match error {
            SemanticError::SelfConnection {
                identifier,
                position,
            } => {
                assert_eq!(identifier, "self");
                assert_eq!(position.line, 3);
                assert_eq!(position.column, 5);
            }
            _ => panic!("Expected SelfConnection variant"),
        }
    }

    #[test]
    fn test_result_type_alias() {
        // Test that Result<T> type alias works correctly
        fn test_function() -> Result<String> {
            Ok("success".to_string())
        }

        fn test_error_function() -> Result<String> {
            Err(DiagramError::Syntax(SyntaxError {
                message: "error".to_string(),
                position: SourcePosition { line: 1, column: 1 },
            }))
        }

        assert!(test_function().is_ok());
        assert!(test_error_function().is_err());
    }

    #[test]
    fn test_syntax_error_equality() {
        let err1 = SyntaxError {
            message: "test".to_string(),
            position: SourcePosition { line: 1, column: 1 },
        };
        let err2 = SyntaxError {
            message: "test".to_string(),
            position: SourcePosition { line: 1, column: 1 },
        };
        assert_eq!(err1, err2);
    }

    #[test]
    fn test_semantic_error_equality() {
        let err1 = SemanticError::UndefinedNode {
            identifier: "node".to_string(),
            position: SourcePosition { line: 1, column: 1 },
        };
        let err2 = SemanticError::UndefinedNode {
            identifier: "node".to_string(),
            position: SourcePosition { line: 1, column: 1 },
        };
        assert_eq!(err1, err2);
    }

    #[test]
    fn test_error_trait_implemented() {
        let error = DiagramError::Syntax(SyntaxError {
            message: "test".to_string(),
            position: SourcePosition { line: 1, column: 1 },
        });
        // This test verifies that std::error::Error is implemented
        let _: &dyn std::error::Error = &error;
    }
}
