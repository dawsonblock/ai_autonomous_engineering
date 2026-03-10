use crate::error::{DiagramError, SyntaxError};
use crate::types::SourcePosition;

/// Tokens recognized by the lexer.
///
/// Represents the fundamental lexical elements of the DSL,
/// including keywords, literals, and punctuation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Token {
    /// The `node` keyword
    Node,
    /// The `as` keyword
    As,
    /// The `type` keyword
    Type,
    /// An identifier (e.g., "api", "db")
    Identifier(String),
    /// A string literal (e.g., "API Gateway")
    String(String),
    /// The `->` arrow operator
    Arrow,
    /// The `:` colon separator
    Colon,
    /// The `[` left bracket
    LeftBracket,
    /// The `]` right bracket
    RightBracket,
    /// A newline character
    Newline,
    /// End of file marker
    Eof,
}

/// Token with source position for error reporting.
///
/// Combines a token with its location in the source file,
/// enabling precise error messages.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PositionedToken {
    /// The token itself
    pub token: Token,
    /// Location where this token was found
    pub position: SourcePosition,
}

/// Lexer for tokenizing DSL input.
///
/// Converts raw DSL source text into a stream of positioned tokens
/// for parsing. Tracks line and column numbers for error reporting.
pub struct Lexer {
    input: Vec<char>,
    position: usize,
    line: usize,
    column: usize,
}

impl Lexer {
    /// Create a new lexer from input string.
    ///
    /// # Arguments
    ///
    /// * `input` - The DSL source code to tokenize
    pub fn new(input: &str) -> Self {
        Self {
            input: input.chars().collect(),
            position: 0,
            line: 1,
            column: 1,
        }
    }

    /// Tokenize the entire input into a vector of positioned tokens.
    ///
    /// Scans through the input character by character, recognizing keywords,
    /// identifiers, strings, and punctuation. Comments (lines starting with `#`)
    /// are skipped.
    ///
    /// # Returns
    ///
    /// * `Ok(Vec<PositionedToken>)` - The token stream, always ending with EOF
    /// * `Err(DiagramError::Syntax)` - If invalid syntax is encountered
    pub fn tokenize(&mut self) -> Result<Vec<PositionedToken>, DiagramError> {
        let mut tokens = Vec::new();

        loop {
            self.skip_whitespace_except_newline();

            // Check for comment (# can appear at start of line or inline)
            if self.peek() == Some('#') {
                self.skip_comment();
                continue;
            }

            let position = SourcePosition {
                line: self.line,
                column: self.column,
            };

            let token = match self.peek() {
                None => Token::Eof,
                Some('\n') => {
                    self.advance();
                    Token::Newline
                }
                Some('"') => self.read_string()?,
                Some('-') => {
                    if self.peek_ahead(1) == Some('>') {
                        self.advance();
                        self.advance();
                        Token::Arrow
                    } else {
                        return Err(DiagramError::Syntax(SyntaxError {
                            message: "unexpected character: '-'".to_string(),
                            position,
                        }));
                    }
                }
                Some(':') => {
                    self.advance();
                    Token::Colon
                }
                Some('[') => {
                    self.advance();
                    Token::LeftBracket
                }
                Some(']') => {
                    self.advance();
                    Token::RightBracket
                }
                Some(c) if c.is_alphabetic() || c == '_' => self.read_identifier_or_keyword(),
                Some(c) => {
                    return Err(DiagramError::Syntax(SyntaxError {
                        message: format!("unexpected character: '{}'", c),
                        position,
                    }));
                }
            };

            let positioned_token = PositionedToken { token, position };

            if positioned_token.token == Token::Eof {
                tokens.push(positioned_token);
                break;
            }

            tokens.push(positioned_token);
        }

        Ok(tokens)
    }

    fn peek(&self) -> Option<char> {
        if self.position < self.input.len() {
            Some(self.input[self.position])
        } else {
            None
        }
    }

    fn peek_ahead(&self, offset: usize) -> Option<char> {
        let pos = self.position + offset;
        if pos < self.input.len() {
            Some(self.input[pos])
        } else {
            None
        }
    }

    fn advance(&mut self) -> Option<char> {
        if let Some(ch) = self.peek() {
            self.position += 1;
            if ch == '\n' {
                self.line += 1;
                self.column = 1;
            } else {
                self.column += 1;
            }
            Some(ch)
        } else {
            None
        }
    }

    fn skip_whitespace_except_newline(&mut self) {
        while let Some(ch) = self.peek() {
            if ch == ' ' || ch == '\t' || ch == '\r' {
                self.advance();
            } else {
                break;
            }
        }
    }

    fn skip_comment(&mut self) {
        // Skip until end of line, but don't consume the newline
        while let Some(ch) = self.peek() {
            if ch == '\n' {
                break;
            }
            self.advance();
        }
    }

    fn read_identifier_or_keyword(&mut self) -> Token {
        let mut identifier = String::new();

        while let Some(ch) = self.peek() {
            if ch.is_alphanumeric() || ch == '_' {
                identifier.push(ch);
                self.advance();
            } else {
                break;
            }
        }

        match identifier.as_str() {
            "node" => Token::Node,
            "as" => Token::As,
            "type" => Token::Type,
            _ => Token::Identifier(identifier),
        }
    }

    fn read_string(&mut self) -> Result<Token, DiagramError> {
        let start_position = SourcePosition {
            line: self.line,
            column: self.column,
        };

        // Skip opening quote
        self.advance();

        let mut string_value = String::new();

        loop {
            match self.peek() {
                None | Some('\n') => {
                    return Err(DiagramError::Syntax(SyntaxError {
                        message: "unterminated string literal".to_string(),
                        position: start_position,
                    }));
                }
                Some('"') => {
                    self.advance();
                    break;
                }
                Some('\\') => {
                    self.advance();
                    match self.peek() {
                        Some('n') => {
                            string_value.push('\n');
                            self.advance();
                        }
                        Some('t') => {
                            string_value.push('\t');
                            self.advance();
                        }
                        Some('r') => {
                            string_value.push('\r');
                            self.advance();
                        }
                        Some('\\') => {
                            string_value.push('\\');
                            self.advance();
                        }
                        Some('"') => {
                            string_value.push('"');
                            self.advance();
                        }
                        None | Some('\n') => {
                            return Err(DiagramError::Syntax(SyntaxError {
                                message: "unterminated string literal".to_string(),
                                position: start_position,
                            }));
                        }
                        Some(ch) => {
                            // For other characters, just include them as-is after backslash
                            string_value.push(ch);
                            self.advance();
                        }
                    }
                }
                Some(ch) => {
                    string_value.push(ch);
                    self.advance();
                }
            }
        }

        Ok(Token::String(string_value))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_input() {
        let mut lexer = Lexer::new("");
        let result = lexer.tokenize();
        assert!(result.is_ok());
        let tokens = result.unwrap();
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0].token, Token::Eof);
        assert_eq!(tokens[0].position.line, 1);
        assert_eq!(tokens[0].position.column, 1);
    }

    #[test]
    fn test_node_declaration() {
        let input = r#"node "API Gateway" as api"#;
        let mut lexer = Lexer::new(input);
        let result = lexer.tokenize();
        assert!(result.is_ok());
        let tokens = result.unwrap();

        assert_eq!(tokens.len(), 5); // node, string, as, identifier, eof
        assert_eq!(tokens[0].token, Token::Node);
        assert_eq!(tokens[1].token, Token::String("API Gateway".to_string()));
        assert_eq!(tokens[2].token, Token::As);
        assert_eq!(tokens[3].token, Token::Identifier("api".to_string()));
        assert_eq!(tokens[4].token, Token::Eof);
    }

    #[test]
    fn test_connection_with_arrow_and_colon() {
        let input = r#"api -> db : "query""#;
        let mut lexer = Lexer::new(input);
        let result = lexer.tokenize();
        assert!(result.is_ok());
        let tokens = result.unwrap();

        assert_eq!(tokens.len(), 6); // identifier, arrow, identifier, colon, string, eof
        assert_eq!(tokens[0].token, Token::Identifier("api".to_string()));
        assert_eq!(tokens[1].token, Token::Arrow);
        assert_eq!(tokens[2].token, Token::Identifier("db".to_string()));
        assert_eq!(tokens[3].token, Token::Colon);
        assert_eq!(tokens[4].token, Token::String("query".to_string()));
        assert_eq!(tokens[5].token, Token::Eof);
    }

    #[test]
    fn test_string_with_escape_sequences() {
        let input = r#""line1\nline2\ttab\"quote\\backslash""#;
        let mut lexer = Lexer::new(input);
        let result = lexer.tokenize();
        assert!(result.is_ok());
        let tokens = result.unwrap();

        assert_eq!(tokens.len(), 2); // string, eof
        assert_eq!(
            tokens[0].token,
            Token::String("line1\nline2\ttab\"quote\\backslash".to_string())
        );
    }

    #[test]
    fn test_comments_are_skipped() {
        let input = "# This is a comment\nnode \"API\" as api\n# Another comment\napi -> db";
        let mut lexer = Lexer::new(input);
        let result = lexer.tokenize();
        assert!(result.is_ok());
        let tokens = result.unwrap();

        // Should have: newline, node, string, as, identifier, newline, newline, identifier, arrow, identifier, eof
        assert_eq!(tokens.len(), 11);
        assert_eq!(tokens[0].token, Token::Newline);
        assert_eq!(tokens[1].token, Token::Node);
        assert_eq!(tokens[2].token, Token::String("API".to_string()));
        assert_eq!(tokens[3].token, Token::As);
        assert_eq!(tokens[4].token, Token::Identifier("api".to_string()));
        assert_eq!(tokens[5].token, Token::Newline);
        assert_eq!(tokens[6].token, Token::Newline);
        assert_eq!(tokens[7].token, Token::Identifier("api".to_string()));
        assert_eq!(tokens[8].token, Token::Arrow);
        assert_eq!(tokens[9].token, Token::Identifier("db".to_string()));
        assert_eq!(tokens[10].token, Token::Eof);
    }

    #[test]
    fn test_unterminated_string_error() {
        let input = r#"node "unterminated"#;
        let mut lexer = Lexer::new(input);
        let result = lexer.tokenize();
        assert!(result.is_err());

        if let Err(DiagramError::Syntax(err)) = result {
            assert!(err.message.contains("unterminated string"));
            assert_eq!(err.position.line, 1);
            assert_eq!(err.position.column, 6);
        } else {
            panic!("Expected SyntaxError for unterminated string");
        }
    }

    #[test]
    fn test_position_tracking() {
        let input = "node \"API\" as api\napi -> db";
        let mut lexer = Lexer::new(input);
        let result = lexer.tokenize();
        assert!(result.is_ok());
        let tokens = result.unwrap();

        // First token (node) at line 1, column 1
        assert_eq!(tokens[0].position.line, 1);
        assert_eq!(tokens[0].position.column, 1);

        // String at line 1, column 6
        assert_eq!(tokens[1].position.line, 1);
        assert_eq!(tokens[1].position.column, 6);

        // Newline at end of line 1
        assert_eq!(tokens[4].position.line, 1);
        assert_eq!(tokens[4].token, Token::Newline);

        // First identifier on line 2 at column 1
        assert_eq!(tokens[5].position.line, 2);
        assert_eq!(tokens[5].position.column, 1);
    }

    #[test]
    fn test_keywords() {
        let input = "node as type";
        let mut lexer = Lexer::new(input);
        let result = lexer.tokenize();
        assert!(result.is_ok());
        let tokens = result.unwrap();

        assert_eq!(tokens[0].token, Token::Node);
        assert_eq!(tokens[1].token, Token::As);
        assert_eq!(tokens[2].token, Token::Type);
    }

    #[test]
    fn test_brackets() {
        let input = "[ ]";
        let mut lexer = Lexer::new(input);
        let result = lexer.tokenize();
        assert!(result.is_ok());
        let tokens = result.unwrap();

        assert_eq!(tokens[0].token, Token::LeftBracket);
        assert_eq!(tokens[1].token, Token::RightBracket);
    }

    #[test]
    fn test_multiple_newlines() {
        let input = "node\n\n\napi";
        let mut lexer = Lexer::new(input);
        let result = lexer.tokenize();
        assert!(result.is_ok());
        let tokens = result.unwrap();

        assert_eq!(tokens[0].token, Token::Node);
        assert_eq!(tokens[1].token, Token::Newline);
        assert_eq!(tokens[2].token, Token::Newline);
        assert_eq!(tokens[3].token, Token::Newline);
        assert_eq!(tokens[4].token, Token::Identifier("api".to_string()));
    }

    #[test]
    fn test_whitespace_handling() {
        let input = "  node   \"API\"  \t  as\tapi  ";
        let mut lexer = Lexer::new(input);
        let result = lexer.tokenize();
        assert!(result.is_ok());
        let tokens = result.unwrap();

        assert_eq!(tokens[0].token, Token::Node);
        assert_eq!(tokens[1].token, Token::String("API".to_string()));
        assert_eq!(tokens[2].token, Token::As);
        assert_eq!(tokens[3].token, Token::Identifier("api".to_string()));
    }

    #[test]
    fn test_identifier_with_underscores() {
        let input = "my_api_service_1";
        let mut lexer = Lexer::new(input);
        let result = lexer.tokenize();
        assert!(result.is_ok());
        let tokens = result.unwrap();

        assert_eq!(
            tokens[0].token,
            Token::Identifier("my_api_service_1".to_string())
        );
    }

    #[test]
    fn test_invalid_character() {
        let input = "node @";
        let mut lexer = Lexer::new(input);
        let result = lexer.tokenize();
        assert!(result.is_err());

        if let Err(DiagramError::Syntax(err)) = result {
            assert!(err.message.contains("unexpected character"));
        } else {
            panic!("Expected SyntaxError for invalid character");
        }
    }

    #[test]
    fn test_string_with_newline_error() {
        let input = "\"line1\nline2\"";
        let mut lexer = Lexer::new(input);
        let result = lexer.tokenize();
        assert!(result.is_err());

        if let Err(DiagramError::Syntax(err)) = result {
            assert!(err.message.contains("unterminated string"));
        } else {
            panic!("Expected SyntaxError for string containing newline");
        }
    }

    #[test]
    fn test_comment_at_beginning() {
        let input = "# Comment\nnode";
        let mut lexer = Lexer::new(input);
        let result = lexer.tokenize();
        assert!(result.is_ok());
        let tokens = result.unwrap();

        // Should have: newline (from comment line), node, eof
        assert_eq!(tokens[0].token, Token::Newline);
        assert_eq!(tokens[1].token, Token::Node);
    }

    #[test]
    fn test_empty_string() {
        let input = r#""""#;
        let mut lexer = Lexer::new(input);
        let result = lexer.tokenize();
        assert!(result.is_ok());
        let tokens = result.unwrap();

        assert_eq!(tokens[0].token, Token::String("".to_string()));
    }

    #[test]
    fn test_arrow_vs_dash() {
        let input = "api - > db";
        let mut lexer = Lexer::new(input);
        let result = lexer.tokenize();
        // Single dash should cause error
        assert!(result.is_err());
    }

    #[test]
    fn test_multiline_diagram() {
        let input = "node \"API\" as api\nnode \"DB\" as db\napi -> db : \"query\"";
        let mut lexer = Lexer::new(input);
        let result = lexer.tokenize();
        assert!(result.is_ok());
        let tokens = result.unwrap();

        // Verify structure
        assert_eq!(tokens[0].token, Token::Node);
        assert_eq!(tokens[4].token, Token::Newline);
        assert_eq!(tokens[5].token, Token::Node);
        assert_eq!(tokens[9].token, Token::Newline);
        assert_eq!(tokens[10].token, Token::Identifier("api".to_string()));
        assert_eq!(tokens[11].token, Token::Arrow);
    }

    #[test]
    fn test_escape_in_middle_of_string() {
        let input = r#""prefix\nsuffix""#;
        let mut lexer = Lexer::new(input);
        let result = lexer.tokenize();
        assert!(result.is_ok());
        let tokens = result.unwrap();

        assert_eq!(tokens[0].token, Token::String("prefix\nsuffix".to_string()));
    }

    #[test]
    fn test_positioned_token_clone() {
        let token = PositionedToken {
            token: Token::Node,
            position: SourcePosition { line: 1, column: 1 },
        };
        let cloned = token.clone();
        assert_eq!(token, cloned);
    }

    #[test]
    fn test_token_debug() {
        let token = Token::Identifier("test".to_string());
        let debug_str = format!("{:?}", token);
        assert!(debug_str.contains("Identifier"));
    }

    #[test]
    fn test_carriage_return_handling() {
        let input = "node\r\napi";
        let mut lexer = Lexer::new(input);
        let result = lexer.tokenize();
        assert!(result.is_ok());
        let tokens = result.unwrap();

        assert_eq!(tokens[0].token, Token::Node);
        assert_eq!(tokens[1].token, Token::Newline);
        assert_eq!(tokens[2].token, Token::Identifier("api".to_string()));
    }
}
