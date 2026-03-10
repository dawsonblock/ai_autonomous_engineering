use crate::error::{DiagramError, SyntaxError};
use crate::lexer::{PositionedToken, Token};
use crate::types::{Connection, Diagram, Node, NodeType, SourcePosition};

/// Parser for building Diagram AST from token stream.
///
/// Implements a recursive descent parser that converts a sequence
/// of tokens into a structured diagram representation.
pub struct Parser {
    tokens: Vec<PositionedToken>,
    position: usize,
}

impl Parser {
    /// Create parser from token stream.
    ///
    /// # Arguments
    ///
    /// * `tokens` - The tokens produced by the lexer
    pub fn new(tokens: Vec<PositionedToken>) -> Self {
        Self {
            tokens,
            position: 0,
        }
    }

    /// Parse tokens into Diagram AST.
    ///
    /// Implements the following grammar:
    /// ```text
    /// diagram := statement*
    /// statement := node_decl | connection_decl
    /// node_decl := "node" STRING "as" IDENTIFIER type_annotation? NEWLINE
    /// type_annotation := "[" "type" ":" IDENTIFIER "]"
    /// connection_decl := IDENTIFIER "->" IDENTIFIER (":" STRING)? NEWLINE
    /// ```
    ///
    /// # Returns
    ///
    /// * `Ok(Diagram)` - The parsed diagram AST
    /// * `Err(DiagramError::Syntax)` - If the token stream doesn't match the grammar
    pub fn parse(&mut self) -> Result<Diagram, DiagramError> {
        let mut nodes = Vec::new();
        let mut connections = Vec::new();

        while !self.is_at_end() {
            self.skip_newlines();
            if self.is_at_end() {
                break;
            }

            match self.peek_token() {
                Token::Node => nodes.push(self.parse_node_decl()?),
                Token::Identifier(_) => connections.push(self.parse_connection_decl()?),
                Token::Eof => break,
                _ => return Err(self.syntax_error("expected 'node' or connection")),
            }
        }

        Ok(Diagram { nodes, connections })
    }

    fn parse_node_decl(&mut self) -> Result<Node, DiagramError> {
        let position = self.current_position();

        // Expect "node" keyword
        self.expect(Token::Node)?;

        // Expect display name string
        let display_name = if let Token::String(name) = self.peek_token() {
            let name = name.clone();
            self.advance();
            name
        } else {
            return Err(self.syntax_error("expected string for node display name"));
        };

        // Expect "as" keyword
        self.expect(Token::As)?;

        // Expect identifier
        let identifier = if let Token::Identifier(id) = self.peek_token() {
            let id = id.clone();
            self.advance();
            id
        } else {
            return Err(self.syntax_error("expected identifier after 'as'"));
        };

        // Parse optional type annotation
        let node_type = self.parse_node_type()?;

        // Expect newline or EOF
        if !self.is_at_end() && self.peek_token() != &Token::Newline {
            return Err(self.syntax_error("expected newline after node declaration"));
        }
        if self.peek_token() == &Token::Newline {
            self.advance();
        }

        Ok(Node {
            identifier,
            display_name,
            node_type,
            position,
        })
    }

    fn parse_connection_decl(&mut self) -> Result<Connection, DiagramError> {
        let position = self.current_position();

        // Parse source identifier
        let from = if let Token::Identifier(id) = self.peek_token() {
            let id = id.clone();
            self.advance();
            id
        } else {
            return Err(self.syntax_error("expected source identifier for connection"));
        };

        // Expect arrow
        self.expect(Token::Arrow)?;

        // Parse target identifier
        let to = if let Token::Identifier(id) = self.peek_token() {
            let id = id.clone();
            self.advance();
            id
        } else {
            return Err(self.syntax_error("expected target identifier after '->'"));
        };

        // Parse optional label
        let label = if self.peek_token() == &Token::Colon {
            self.advance(); // consume colon
            if let Token::String(label_text) = self.peek_token() {
                let label_text = label_text.clone();
                self.advance();
                Some(label_text)
            } else {
                return Err(self.syntax_error("expected string after ':' in connection"));
            }
        } else {
            None
        };

        // Expect newline or EOF
        if !self.is_at_end() && self.peek_token() != &Token::Newline {
            return Err(self.syntax_error("expected newline after connection declaration"));
        }
        if self.peek_token() == &Token::Newline {
            self.advance();
        }

        Ok(Connection {
            from,
            to,
            label,
            position,
        })
    }

    fn parse_node_type(&mut self) -> Result<NodeType, DiagramError> {
        // Check for type annotation: [type: service|database|external|queue]
        if self.peek_token() == &Token::LeftBracket {
            self.advance(); // consume '['

            // Expect "type" keyword
            self.expect(Token::Type)?;

            // Expect colon
            self.expect(Token::Colon)?;

            // Parse type identifier
            let type_name = if let Token::Identifier(type_id) = self.peek_token() {
                let type_id = type_id.clone();
                self.advance();
                type_id
            } else {
                return Err(self.syntax_error("expected type identifier after 'type:'"));
            };

            // Parse node type from identifier
            let node_type = match type_name.as_str() {
                "service" => NodeType::Service,
                "database" => NodeType::Database,
                "external" => NodeType::External,
                "queue" => NodeType::Queue,
                _ => {
                    return Err(self.syntax_error(&format!(
                        "invalid node type '{}', expected one of: service, database, external, queue",
                        type_name
                    )));
                }
            };

            // Expect closing bracket
            self.expect(Token::RightBracket)?;

            Ok(node_type)
        } else {
            // Default to Service if no type annotation
            Ok(NodeType::Service)
        }
    }

    fn peek_token(&self) -> &Token {
        if self.position < self.tokens.len() {
            &self.tokens[self.position].token
        } else {
            // If we're past the end, assume EOF
            &Token::Eof
        }
    }

    fn current_position(&self) -> SourcePosition {
        if self.position < self.tokens.len() {
            self.tokens[self.position].position
        } else if !self.tokens.is_empty() {
            // Return position of last token if we're past the end
            self.tokens[self.tokens.len() - 1].position
        } else {
            SourcePosition { line: 1, column: 1 }
        }
    }

    fn advance(&mut self) -> &PositionedToken {
        let tok = &self.tokens[self.position];
        if self.position < self.tokens.len() {
            self.position += 1;
        }
        tok
    }

    fn expect(&mut self, expected: Token) -> Result<(), DiagramError> {
        if self.peek_token() == &expected {
            self.advance();
            Ok(())
        } else {
            Err(self.syntax_error(&format!("expected {:?}", expected)))
        }
    }

    fn skip_newlines(&mut self) {
        while self.peek_token() == &Token::Newline {
            self.advance();
        }
    }

    fn is_at_end(&self) -> bool {
        matches!(self.peek_token(), Token::Eof)
    }

    fn syntax_error(&self, message: &str) -> DiagramError {
        DiagramError::Syntax(SyntaxError {
            message: message.to_string(),
            position: self.current_position(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_token(token: Token, line: usize, column: usize) -> PositionedToken {
        PositionedToken {
            token,
            position: SourcePosition { line, column },
        }
    }

    #[test]
    fn test_parse_single_node() {
        let tokens = vec![
            make_token(Token::Node, 1, 1),
            make_token(Token::String("API".to_string()), 1, 6),
            make_token(Token::As, 1, 12),
            make_token(Token::Identifier("api".to_string()), 1, 15),
            make_token(Token::Newline, 1, 18),
            make_token(Token::Eof, 2, 1),
        ];

        let mut parser = Parser::new(tokens);
        let result = parser.parse();

        assert!(result.is_ok());
        let diagram = result.unwrap();
        assert_eq!(diagram.nodes.len(), 1);
        assert_eq!(diagram.connections.len(), 0);

        let node = &diagram.nodes[0];
        assert_eq!(node.identifier, "api");
        assert_eq!(node.display_name, "API");
        assert_eq!(node.node_type, NodeType::Service); // default type
        assert_eq!(node.position.line, 1);
    }

    #[test]
    fn test_parse_node_with_type_annotation() {
        let tokens = vec![
            make_token(Token::Node, 1, 1),
            make_token(Token::String("Database".to_string()), 1, 6),
            make_token(Token::As, 1, 16),
            make_token(Token::Identifier("db".to_string()), 1, 19),
            make_token(Token::LeftBracket, 1, 22),
            make_token(Token::Type, 1, 23),
            make_token(Token::Colon, 1, 27),
            make_token(Token::Identifier("database".to_string()), 1, 29),
            make_token(Token::RightBracket, 1, 37),
            make_token(Token::Newline, 1, 38),
            make_token(Token::Eof, 2, 1),
        ];

        let mut parser = Parser::new(tokens);
        let result = parser.parse();

        assert!(result.is_ok());
        let diagram = result.unwrap();
        assert_eq!(diagram.nodes.len(), 1);

        let node = &diagram.nodes[0];
        assert_eq!(node.identifier, "db");
        assert_eq!(node.display_name, "Database");
        assert_eq!(node.node_type, NodeType::Database);
    }

    #[test]
    fn test_parse_connection_with_label() {
        let tokens = vec![
            make_token(Token::Identifier("api".to_string()), 1, 1),
            make_token(Token::Arrow, 1, 5),
            make_token(Token::Identifier("db".to_string()), 1, 8),
            make_token(Token::Colon, 1, 11),
            make_token(Token::String("SQL query".to_string()), 1, 13),
            make_token(Token::Newline, 1, 24),
            make_token(Token::Eof, 2, 1),
        ];

        let mut parser = Parser::new(tokens);
        let result = parser.parse();

        assert!(result.is_ok());
        let diagram = result.unwrap();
        assert_eq!(diagram.nodes.len(), 0);
        assert_eq!(diagram.connections.len(), 1);

        let conn = &diagram.connections[0];
        assert_eq!(conn.from, "api");
        assert_eq!(conn.to, "db");
        assert_eq!(conn.label, Some("SQL query".to_string()));
        assert_eq!(conn.position.line, 1);
    }

    #[test]
    fn test_parse_connection_without_label() {
        let tokens = vec![
            make_token(Token::Identifier("api".to_string()), 1, 1),
            make_token(Token::Arrow, 1, 5),
            make_token(Token::Identifier("db".to_string()), 1, 8),
            make_token(Token::Newline, 1, 10),
            make_token(Token::Eof, 2, 1),
        ];

        let mut parser = Parser::new(tokens);
        let result = parser.parse();

        assert!(result.is_ok());
        let diagram = result.unwrap();
        assert_eq!(diagram.connections.len(), 1);

        let conn = &diagram.connections[0];
        assert_eq!(conn.from, "api");
        assert_eq!(conn.to, "db");
        assert_eq!(conn.label, None);
    }

    #[test]
    fn test_parse_multiple_statements() {
        let tokens = vec![
            // node "API" as api
            make_token(Token::Node, 1, 1),
            make_token(Token::String("API".to_string()), 1, 6),
            make_token(Token::As, 1, 12),
            make_token(Token::Identifier("api".to_string()), 1, 15),
            make_token(Token::Newline, 1, 18),
            // node "DB" as db [type: database]
            make_token(Token::Node, 2, 1),
            make_token(Token::String("DB".to_string()), 2, 6),
            make_token(Token::As, 2, 11),
            make_token(Token::Identifier("db".to_string()), 2, 14),
            make_token(Token::LeftBracket, 2, 17),
            make_token(Token::Type, 2, 18),
            make_token(Token::Colon, 2, 22),
            make_token(Token::Identifier("database".to_string()), 2, 24),
            make_token(Token::RightBracket, 2, 32),
            make_token(Token::Newline, 2, 33),
            // api -> db : "query"
            make_token(Token::Identifier("api".to_string()), 3, 1),
            make_token(Token::Arrow, 3, 5),
            make_token(Token::Identifier("db".to_string()), 3, 8),
            make_token(Token::Colon, 3, 11),
            make_token(Token::String("query".to_string()), 3, 13),
            make_token(Token::Newline, 3, 20),
            make_token(Token::Eof, 4, 1),
        ];

        let mut parser = Parser::new(tokens);
        let result = parser.parse();

        assert!(result.is_ok());
        let diagram = result.unwrap();
        assert_eq!(diagram.nodes.len(), 2);
        assert_eq!(diagram.connections.len(), 1);

        assert_eq!(diagram.nodes[0].identifier, "api");
        assert_eq!(diagram.nodes[1].identifier, "db");
        assert_eq!(diagram.nodes[1].node_type, NodeType::Database);

        assert_eq!(diagram.connections[0].from, "api");
        assert_eq!(diagram.connections[0].to, "db");
    }

    #[test]
    fn test_error_malformed_node_missing_as() {
        let tokens = vec![
            make_token(Token::Node, 1, 1),
            make_token(Token::String("API".to_string()), 1, 6),
            make_token(Token::Identifier("api".to_string()), 1, 12),
            make_token(Token::Eof, 1, 15),
        ];

        let mut parser = Parser::new(tokens);
        let result = parser.parse();

        assert!(result.is_err());
        if let Err(DiagramError::Syntax(err)) = result {
            assert!(err.message.contains("expected"));
            assert_eq!(err.position.line, 1);
        } else {
            panic!("Expected SyntaxError");
        }
    }

    #[test]
    fn test_error_invalid_node_type() {
        let tokens = vec![
            make_token(Token::Node, 1, 1),
            make_token(Token::String("API".to_string()), 1, 6),
            make_token(Token::As, 1, 12),
            make_token(Token::Identifier("api".to_string()), 1, 15),
            make_token(Token::LeftBracket, 1, 19),
            make_token(Token::Type, 1, 20),
            make_token(Token::Colon, 1, 24),
            make_token(Token::Identifier("invalid_type".to_string()), 1, 26),
            make_token(Token::RightBracket, 1, 38),
            make_token(Token::Eof, 1, 39),
        ];

        let mut parser = Parser::new(tokens);
        let result = parser.parse();

        assert!(result.is_err());
        if let Err(DiagramError::Syntax(err)) = result {
            assert!(err.message.contains("invalid node type"));
            assert!(err.message.contains("invalid_type"));
        } else {
            panic!("Expected SyntaxError for invalid node type");
        }
    }

    #[test]
    fn test_parse_all_node_types() {
        // Test service type
        let tokens = vec![
            make_token(Token::Node, 1, 1),
            make_token(Token::String("Service".to_string()), 1, 6),
            make_token(Token::As, 1, 15),
            make_token(Token::Identifier("svc".to_string()), 1, 18),
            make_token(Token::LeftBracket, 1, 22),
            make_token(Token::Type, 1, 23),
            make_token(Token::Colon, 1, 27),
            make_token(Token::Identifier("service".to_string()), 1, 29),
            make_token(Token::RightBracket, 1, 36),
            make_token(Token::Eof, 1, 37),
        ];
        let mut parser = Parser::new(tokens);
        let result = parser.parse();
        assert!(result.is_ok());
        assert_eq!(result.unwrap().nodes[0].node_type, NodeType::Service);

        // Test database type
        let tokens = vec![
            make_token(Token::Node, 1, 1),
            make_token(Token::String("DB".to_string()), 1, 6),
            make_token(Token::As, 1, 10),
            make_token(Token::Identifier("db".to_string()), 1, 13),
            make_token(Token::LeftBracket, 1, 16),
            make_token(Token::Type, 1, 17),
            make_token(Token::Colon, 1, 21),
            make_token(Token::Identifier("database".to_string()), 1, 23),
            make_token(Token::RightBracket, 1, 31),
            make_token(Token::Eof, 1, 32),
        ];
        let mut parser = Parser::new(tokens);
        let result = parser.parse();
        assert!(result.is_ok());
        assert_eq!(result.unwrap().nodes[0].node_type, NodeType::Database);

        // Test external type
        let tokens = vec![
            make_token(Token::Node, 1, 1),
            make_token(Token::String("External".to_string()), 1, 6),
            make_token(Token::As, 1, 16),
            make_token(Token::Identifier("ext".to_string()), 1, 19),
            make_token(Token::LeftBracket, 1, 23),
            make_token(Token::Type, 1, 24),
            make_token(Token::Colon, 1, 28),
            make_token(Token::Identifier("external".to_string()), 1, 30),
            make_token(Token::RightBracket, 1, 38),
            make_token(Token::Eof, 1, 39),
        ];
        let mut parser = Parser::new(tokens);
        let result = parser.parse();
        assert!(result.is_ok());
        assert_eq!(result.unwrap().nodes[0].node_type, NodeType::External);

        // Test queue type
        let tokens = vec![
            make_token(Token::Node, 1, 1),
            make_token(Token::String("Queue".to_string()), 1, 6),
            make_token(Token::As, 1, 13),
            make_token(Token::Identifier("q".to_string()), 1, 16),
            make_token(Token::LeftBracket, 1, 18),
            make_token(Token::Type, 1, 19),
            make_token(Token::Colon, 1, 23),
            make_token(Token::Identifier("queue".to_string()), 1, 25),
            make_token(Token::RightBracket, 1, 30),
            make_token(Token::Eof, 1, 31),
        ];
        let mut parser = Parser::new(tokens);
        let result = parser.parse();
        assert!(result.is_ok());
        assert_eq!(result.unwrap().nodes[0].node_type, NodeType::Queue);
    }

    #[test]
    fn test_parse_empty_input() {
        let tokens = vec![make_token(Token::Eof, 1, 1)];
        let mut parser = Parser::new(tokens);
        let result = parser.parse();

        assert!(result.is_ok());
        let diagram = result.unwrap();
        assert_eq!(diagram.nodes.len(), 0);
        assert_eq!(diagram.connections.len(), 0);
    }

    #[test]
    fn test_parse_with_leading_newlines() {
        let tokens = vec![
            make_token(Token::Newline, 1, 1),
            make_token(Token::Newline, 2, 1),
            make_token(Token::Node, 3, 1),
            make_token(Token::String("API".to_string()), 3, 6),
            make_token(Token::As, 3, 12),
            make_token(Token::Identifier("api".to_string()), 3, 15),
            make_token(Token::Eof, 3, 18),
        ];

        let mut parser = Parser::new(tokens);
        let result = parser.parse();

        assert!(result.is_ok());
        let diagram = result.unwrap();
        assert_eq!(diagram.nodes.len(), 1);
    }

    #[test]
    fn test_error_missing_arrow_in_connection() {
        let tokens = vec![
            make_token(Token::Identifier("api".to_string()), 1, 1),
            make_token(Token::Identifier("db".to_string()), 1, 5),
            make_token(Token::Eof, 1, 7),
        ];

        let mut parser = Parser::new(tokens);
        let result = parser.parse();

        assert!(result.is_err());
        if let Err(DiagramError::Syntax(err)) = result {
            assert!(err.message.contains("expected"));
        } else {
            panic!("Expected SyntaxError");
        }
    }

    #[test]
    fn test_error_missing_string_after_colon() {
        let tokens = vec![
            make_token(Token::Identifier("api".to_string()), 1, 1),
            make_token(Token::Arrow, 1, 5),
            make_token(Token::Identifier("db".to_string()), 1, 8),
            make_token(Token::Colon, 1, 11),
            make_token(Token::Eof, 1, 12),
        ];

        let mut parser = Parser::new(tokens);
        let result = parser.parse();

        assert!(result.is_err());
        if let Err(DiagramError::Syntax(err)) = result {
            assert!(err.message.contains("expected string after ':'"));
        } else {
            panic!("Expected SyntaxError");
        }
    }

    #[test]
    fn test_node_without_newline_at_end() {
        // Test that a node at EOF without trailing newline is valid
        let tokens = vec![
            make_token(Token::Node, 1, 1),
            make_token(Token::String("API".to_string()), 1, 6),
            make_token(Token::As, 1, 12),
            make_token(Token::Identifier("api".to_string()), 1, 15),
            make_token(Token::Eof, 1, 18),
        ];

        let mut parser = Parser::new(tokens);
        let result = parser.parse();

        assert!(result.is_ok());
        let diagram = result.unwrap();
        assert_eq!(diagram.nodes.len(), 1);
    }

    #[test]
    fn test_connection_without_newline_at_end() {
        // Test that a connection at EOF without trailing newline is valid
        let tokens = vec![
            make_token(Token::Identifier("api".to_string()), 1, 1),
            make_token(Token::Arrow, 1, 5),
            make_token(Token::Identifier("db".to_string()), 1, 8),
            make_token(Token::Eof, 1, 10),
        ];

        let mut parser = Parser::new(tokens);
        let result = parser.parse();

        assert!(result.is_ok());
        let diagram = result.unwrap();
        assert_eq!(diagram.connections.len(), 1);
    }
}
