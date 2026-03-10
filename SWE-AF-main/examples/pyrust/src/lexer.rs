//! Zero-copy lexer for Python-like source code
//!
//! Implements a single-pass O(n) tokenizer that converts source code into a token stream.
//! Uses lifetime parameters to store zero-copy &str slices, avoiding allocations.
//! Target performance: ~5μs for 50-byte input.

use crate::error::LexError;

/// All token types supported in Phase 1
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokenKind {
    // Literals and identifiers
    Integer,
    Identifier,

    // Operators
    Plus,        // +
    Minus,       // -
    Star,        // *
    Slash,       // /
    DoubleSlash, // //
    Percent,     // %

    // Delimiters
    LeftParen,  // (
    RightParen, // )
    Colon,      // :
    Comma,      // ,

    // Assignment
    Equals, // =

    // Keywords
    Print,  // print
    Def,    // def
    Return, // return

    // Special
    Newline, // \n
    Eof,     // End of file
}

/// Token with location tracking and zero-copy text slice
///
/// The lifetime parameter 'src ensures that tokens cannot outlive the source string.
/// This enables zero-copy lexing without heap allocations for token text.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Token<'src> {
    pub kind: TokenKind,
    /// Zero-copy slice into the source string
    pub text: &'src str,
    /// 1-indexed line number
    pub line: usize,
    /// 1-indexed column number (byte offset from line start + 1)
    pub column: usize,
}

impl<'src> Token<'src> {
    /// Creates a new token
    fn new(kind: TokenKind, text: &'src str, line: usize, column: usize) -> Self {
        Self {
            kind,
            text,
            line,
            column,
        }
    }
}

/// Lexer state for tracking position in source
struct Lexer<'src> {
    /// Source code being lexed
    source: &'src str,
    /// Current byte position in source
    pos: usize,
    /// Current line number (1-indexed)
    line: usize,
    /// Current column number (1-indexed, byte offset from line start + 1)
    column: usize,
}

impl<'src> Lexer<'src> {
    /// Creates a new lexer for the given source
    fn new(source: &'src str) -> Self {
        Self {
            source,
            pos: 0,
            line: 1,
            column: 1,
        }
    }

    /// Returns the current character without consuming it
    fn peek(&self) -> Option<char> {
        self.source[self.pos..].chars().next()
    }

    /// Consumes and returns the current character
    fn advance(&mut self) -> Option<char> {
        let ch = self.peek()?;
        self.pos += ch.len_utf8();
        if ch == '\n' {
            self.line += 1;
            self.column = 1;
        } else {
            self.column += 1;
        }
        Some(ch)
    }

    /// Skips whitespace (except newlines, which are tokens)
    fn skip_whitespace(&mut self) {
        while let Some(ch) = self.peek() {
            if ch == ' ' || ch == '\t' || ch == '\r' {
                self.advance();
            } else {
                break;
            }
        }
    }

    /// Lexes an integer literal
    fn lex_integer(
        &mut self,
        start_pos: usize,
        start_line: usize,
        start_column: usize,
    ) -> Result<Token<'src>, LexError> {
        while let Some(ch) = self.peek() {
            if ch.is_ascii_digit() {
                self.advance();
            } else {
                break;
            }
        }

        let text = &self.source[start_pos..self.pos];

        // Validate integer doesn't overflow i64
        if text.parse::<i64>().is_err() {
            return Err(LexError {
                message: format!(
                    "Integer literal '{}' is too large (exceeds i64 range)",
                    text
                ),
                line: start_line,
                column: start_column,
            });
        }

        Ok(Token::new(
            TokenKind::Integer,
            text,
            start_line,
            start_column,
        ))
    }

    /// Lexes an identifier or keyword
    fn lex_identifier(
        &mut self,
        start_pos: usize,
        start_line: usize,
        start_column: usize,
    ) -> Token<'src> {
        while let Some(ch) = self.peek() {
            if ch.is_ascii_alphanumeric() || ch == '_' {
                self.advance();
            } else {
                break;
            }
        }

        let text = &self.source[start_pos..self.pos];

        // Check if it's a keyword
        let kind = match text {
            "print" => TokenKind::Print,
            "def" => TokenKind::Def,
            "return" => TokenKind::Return,
            _ => TokenKind::Identifier,
        };

        Token::new(kind, text, start_line, start_column)
    }

    /// Lexes the next token
    fn next_token(&mut self) -> Result<Option<Token<'src>>, LexError> {
        self.skip_whitespace();

        let start_pos = self.pos;
        let start_line = self.line;
        let start_column = self.column;

        let ch = match self.peek() {
            Some(ch) => ch,
            None => {
                // End of file
                return Ok(Some(Token::new(
                    TokenKind::Eof,
                    "",
                    start_line,
                    start_column,
                )));
            }
        };

        let token = match ch {
            // Newline
            '\n' => {
                self.advance();
                Token::new(
                    TokenKind::Newline,
                    &self.source[start_pos..self.pos],
                    start_line,
                    start_column,
                )
            }

            // Single character tokens
            '+' => {
                self.advance();
                Token::new(
                    TokenKind::Plus,
                    &self.source[start_pos..self.pos],
                    start_line,
                    start_column,
                )
            }
            '-' => {
                self.advance();
                Token::new(
                    TokenKind::Minus,
                    &self.source[start_pos..self.pos],
                    start_line,
                    start_column,
                )
            }
            '*' => {
                self.advance();
                Token::new(
                    TokenKind::Star,
                    &self.source[start_pos..self.pos],
                    start_line,
                    start_column,
                )
            }
            '%' => {
                self.advance();
                Token::new(
                    TokenKind::Percent,
                    &self.source[start_pos..self.pos],
                    start_line,
                    start_column,
                )
            }
            '(' => {
                self.advance();
                Token::new(
                    TokenKind::LeftParen,
                    &self.source[start_pos..self.pos],
                    start_line,
                    start_column,
                )
            }
            ')' => {
                self.advance();
                Token::new(
                    TokenKind::RightParen,
                    &self.source[start_pos..self.pos],
                    start_line,
                    start_column,
                )
            }
            '=' => {
                self.advance();
                Token::new(
                    TokenKind::Equals,
                    &self.source[start_pos..self.pos],
                    start_line,
                    start_column,
                )
            }
            ':' => {
                self.advance();
                Token::new(
                    TokenKind::Colon,
                    &self.source[start_pos..self.pos],
                    start_line,
                    start_column,
                )
            }
            ',' => {
                self.advance();
                Token::new(
                    TokenKind::Comma,
                    &self.source[start_pos..self.pos],
                    start_line,
                    start_column,
                )
            }

            // Slash or DoubleSlash
            '/' => {
                self.advance();
                // Check for //
                if self.peek() == Some('/') {
                    self.advance();
                    Token::new(
                        TokenKind::DoubleSlash,
                        &self.source[start_pos..self.pos],
                        start_line,
                        start_column,
                    )
                } else {
                    Token::new(
                        TokenKind::Slash,
                        &self.source[start_pos..self.pos],
                        start_line,
                        start_column,
                    )
                }
            }

            // Integer literal
            '0'..='9' => {
                return self
                    .lex_integer(start_pos, start_line, start_column)
                    .map(Some);
            }

            // Identifier or keyword
            'a'..='z' | 'A'..='Z' | '_' => {
                return Ok(Some(self.lex_identifier(
                    start_pos,
                    start_line,
                    start_column,
                )));
            }

            // Unexpected character
            _ => {
                return Err(LexError {
                    message: format!("Unexpected character '{}'", ch),
                    line: start_line,
                    column: start_column,
                });
            }
        };

        Ok(Some(token))
    }
}

/// Tokenizes Python source code into a vector of tokens
///
/// This is the main entry point for lexing. It performs a single-pass O(n) scan
/// of the source code and returns zero-copy tokens.
///
/// # Arguments
/// * `source` - The source code to tokenize
///
/// # Returns
/// * `Ok(Vec<Token>)` - Vector of tokens including a final Eof token
/// * `Err(LexError)` - Error with location information if lexing fails
///
/// # Examples
/// ```
/// use pyrust::lexer::{lex, TokenKind};
///
/// let tokens = lex("x = 42").unwrap();
/// assert_eq!(tokens.len(), 4); // Identifier, Equals, Integer, Eof
/// assert_eq!(tokens[0].kind, TokenKind::Identifier);
/// assert_eq!(tokens[0].text, "x");
/// ```
pub fn lex(source: &str) -> Result<Vec<Token<'_>>, LexError> {
    let mut lexer = Lexer::new(source);
    let mut tokens = Vec::new();

    loop {
        match lexer.next_token()? {
            Some(token) => {
                let is_eof = token.kind == TokenKind::Eof;
                tokens.push(token);
                if is_eof {
                    break;
                }
            }
            None => {
                // Should not happen, but handle gracefully
                tokens.push(Token::new(TokenKind::Eof, "", lexer.line, lexer.column));
                break;
            }
        }
    }

    Ok(tokens)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_source() {
        let tokens = lex("").unwrap();
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0].kind, TokenKind::Eof);
    }

    #[test]
    fn test_single_integer() {
        let tokens = lex("42").unwrap();
        assert_eq!(tokens.len(), 2);
        assert_eq!(tokens[0].kind, TokenKind::Integer);
        assert_eq!(tokens[0].text, "42");
        assert_eq!(tokens[0].line, 1);
        assert_eq!(tokens[0].column, 1);
        assert_eq!(tokens[1].kind, TokenKind::Eof);
    }

    #[test]
    fn test_integer_overflow() {
        // i64::MAX is 9223372036854775807
        let result = lex("99999999999999999999999999999");
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.message.contains("too large"));
        assert_eq!(err.line, 1);
        assert_eq!(err.column, 1);
    }

    #[test]
    fn test_identifier() {
        let tokens = lex("hello_world123").unwrap();
        assert_eq!(tokens.len(), 2);
        assert_eq!(tokens[0].kind, TokenKind::Identifier);
        assert_eq!(tokens[0].text, "hello_world123");
    }

    #[test]
    fn test_print_keyword() {
        let tokens = lex("print").unwrap();
        assert_eq!(tokens.len(), 2);
        assert_eq!(tokens[0].kind, TokenKind::Print);
        assert_eq!(tokens[0].text, "print");
    }

    #[test]
    fn test_print_vs_identifier() {
        // "print" should be a keyword
        let tokens = lex("print").unwrap();
        assert_eq!(tokens[0].kind, TokenKind::Print);

        // "printer" should be an identifier
        let tokens = lex("printer").unwrap();
        assert_eq!(tokens[0].kind, TokenKind::Identifier);

        // "printing" should be an identifier
        let tokens = lex("printing").unwrap();
        assert_eq!(tokens[0].kind, TokenKind::Identifier);
    }

    #[test]
    fn test_operators() {
        let tokens = lex("+ - * / % = ( )").unwrap();
        assert_eq!(tokens.len(), 9); // 8 operators + EOF
        assert_eq!(tokens[0].kind, TokenKind::Plus);
        assert_eq!(tokens[1].kind, TokenKind::Minus);
        assert_eq!(tokens[2].kind, TokenKind::Star);
        assert_eq!(tokens[3].kind, TokenKind::Slash);
        assert_eq!(tokens[4].kind, TokenKind::Percent);
        assert_eq!(tokens[5].kind, TokenKind::Equals);
        assert_eq!(tokens[6].kind, TokenKind::LeftParen);
        assert_eq!(tokens[7].kind, TokenKind::RightParen);
    }

    #[test]
    fn test_double_slash() {
        let tokens = lex("//").unwrap();
        assert_eq!(tokens.len(), 2);
        assert_eq!(tokens[0].kind, TokenKind::DoubleSlash);
        assert_eq!(tokens[0].text, "//");
    }

    #[test]
    fn test_double_slash_not_two_slashes() {
        // Ensure // is lexed as a single token, not two Slash tokens
        let tokens = lex("//").unwrap();
        assert_eq!(tokens.len(), 2); // DoubleSlash + Eof
        assert_eq!(tokens[0].kind, TokenKind::DoubleSlash);
        assert_ne!(tokens[0].kind, TokenKind::Slash);
    }

    #[test]
    fn test_slash_vs_double_slash() {
        // Single slash
        let tokens = lex("10 / 2").unwrap();
        assert_eq!(tokens[1].kind, TokenKind::Slash);
        assert_eq!(tokens[1].text, "/");

        // Double slash
        let tokens = lex("10 // 2").unwrap();
        assert_eq!(tokens[1].kind, TokenKind::DoubleSlash);
        assert_eq!(tokens[1].text, "//");
    }

    #[test]
    fn test_newline() {
        let tokens = lex("x\ny").unwrap();
        assert_eq!(tokens.len(), 4); // x, newline, y, eof
        assert_eq!(tokens[0].kind, TokenKind::Identifier);
        assert_eq!(tokens[1].kind, TokenKind::Newline);
        assert_eq!(tokens[2].kind, TokenKind::Identifier);
    }

    #[test]
    fn test_location_tracking() {
        let source = "x = 42\ny = 10";
        let tokens = lex(source).unwrap();

        // x at line 1, column 1
        assert_eq!(tokens[0].line, 1);
        assert_eq!(tokens[0].column, 1);

        // = at line 1, column 3
        assert_eq!(tokens[1].line, 1);
        assert_eq!(tokens[1].column, 3);

        // 42 at line 1, column 5
        assert_eq!(tokens[2].line, 1);
        assert_eq!(tokens[2].column, 5);

        // newline at line 1, column 7
        assert_eq!(tokens[3].line, 1);
        assert_eq!(tokens[3].column, 7);

        // y at line 2, column 1
        assert_eq!(tokens[4].line, 2);
        assert_eq!(tokens[4].column, 1);

        // = at line 2, column 3
        assert_eq!(tokens[5].line, 2);
        assert_eq!(tokens[5].column, 3);

        // 10 at line 2, column 5
        assert_eq!(tokens[6].line, 2);
        assert_eq!(tokens[6].column, 5);
    }

    #[test]
    fn test_assignment_statement() {
        let tokens = lex("x = 42").unwrap();
        assert_eq!(tokens.len(), 4); // Identifier, Equals, Integer, Eof
        assert_eq!(tokens[0].kind, TokenKind::Identifier);
        assert_eq!(tokens[0].text, "x");
        assert_eq!(tokens[1].kind, TokenKind::Equals);
        assert_eq!(tokens[2].kind, TokenKind::Integer);
        assert_eq!(tokens[2].text, "42");
    }

    #[test]
    fn test_print_statement() {
        let tokens = lex("print(x)").unwrap();
        assert_eq!(tokens.len(), 5); // Print, LeftParen, Identifier, RightParen, Eof
        assert_eq!(tokens[0].kind, TokenKind::Print);
        assert_eq!(tokens[1].kind, TokenKind::LeftParen);
        assert_eq!(tokens[2].kind, TokenKind::Identifier);
        assert_eq!(tokens[3].kind, TokenKind::RightParen);
    }

    #[test]
    fn test_arithmetic_expression() {
        let tokens = lex("1 + 2 * 3 - 4 / 5 % 6").unwrap();
        assert_eq!(tokens[0].kind, TokenKind::Integer);
        assert_eq!(tokens[1].kind, TokenKind::Plus);
        assert_eq!(tokens[2].kind, TokenKind::Integer);
        assert_eq!(tokens[3].kind, TokenKind::Star);
        assert_eq!(tokens[4].kind, TokenKind::Integer);
        assert_eq!(tokens[5].kind, TokenKind::Minus);
        assert_eq!(tokens[6].kind, TokenKind::Integer);
        assert_eq!(tokens[7].kind, TokenKind::Slash);
        assert_eq!(tokens[8].kind, TokenKind::Integer);
        assert_eq!(tokens[9].kind, TokenKind::Percent);
        assert_eq!(tokens[10].kind, TokenKind::Integer);
    }

    #[test]
    fn test_floor_division_expression() {
        let tokens = lex("10 // 3").unwrap();
        assert_eq!(tokens.len(), 4); // Integer, DoubleSlash, Integer, Eof
        assert_eq!(tokens[0].kind, TokenKind::Integer);
        assert_eq!(tokens[1].kind, TokenKind::DoubleSlash);
        assert_eq!(tokens[2].kind, TokenKind::Integer);
    }

    #[test]
    fn test_unexpected_character() {
        let result = lex("x = @");
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.message.contains("Unexpected character"));
        assert!(err.message.contains("@"));
        assert_eq!(err.line, 1);
        assert_eq!(err.column, 5);
    }

    #[test]
    fn test_unexpected_character_multiline() {
        let result = lex("x = 1\ny = $");
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.message.contains("$"));
        assert_eq!(err.line, 2);
        assert_eq!(err.column, 5);
    }

    #[test]
    fn test_zero_copy_tokens() {
        // Verify that tokens contain slices into the original source
        let source = "x = 42";
        let tokens = lex(source).unwrap();

        // Get the text from token
        let token_text = tokens[0].text;

        // Verify it's pointing into the original source
        let source_ptr = source.as_ptr() as usize;
        let token_ptr = token_text.as_ptr() as usize;

        // The token text should be within the source string's memory range
        assert!(token_ptr >= source_ptr);
        assert!(token_ptr < source_ptr + source.len());
    }

    #[test]
    fn test_whitespace_handling() {
        let tokens = lex("  x   =   42  ").unwrap();
        assert_eq!(tokens.len(), 4); // Identifier, Equals, Integer, Eof
        assert_eq!(tokens[0].text, "x");
        assert_eq!(tokens[1].text, "=");
        assert_eq!(tokens[2].text, "42");
    }

    #[test]
    fn test_tabs_and_spaces() {
        let tokens = lex("\tx\t=\t42\t").unwrap();
        assert_eq!(tokens.len(), 4);
        assert_eq!(tokens[0].kind, TokenKind::Identifier);
        assert_eq!(tokens[1].kind, TokenKind::Equals);
        assert_eq!(tokens[2].kind, TokenKind::Integer);
    }

    #[test]
    fn test_multiple_newlines() {
        let tokens = lex("x\n\n\ny").unwrap();
        assert_eq!(tokens.len(), 6); // x, newline, newline, newline, y, eof
        assert_eq!(tokens[0].kind, TokenKind::Identifier);
        assert_eq!(tokens[1].kind, TokenKind::Newline);
        assert_eq!(tokens[2].kind, TokenKind::Newline);
        assert_eq!(tokens[3].kind, TokenKind::Newline);
        assert_eq!(tokens[4].kind, TokenKind::Identifier);
    }

    #[test]
    fn test_complex_program() {
        let source = "x = 10\ny = 20\nz = x + y\nprint(z)\n";
        let tokens = lex(source).unwrap();

        // Verify we get all expected tokens
        let kinds: Vec<_> = tokens.iter().map(|t| t.kind).collect();
        assert_eq!(
            kinds,
            vec![
                TokenKind::Identifier, // x
                TokenKind::Equals,     // =
                TokenKind::Integer,    // 10
                TokenKind::Newline,    // \n
                TokenKind::Identifier, // y
                TokenKind::Equals,     // =
                TokenKind::Integer,    // 20
                TokenKind::Newline,    // \n
                TokenKind::Identifier, // z
                TokenKind::Equals,     // =
                TokenKind::Identifier, // x
                TokenKind::Plus,       // +
                TokenKind::Identifier, // y
                TokenKind::Newline,    // \n
                TokenKind::Print,      // print
                TokenKind::LeftParen,  // (
                TokenKind::Identifier, // z
                TokenKind::RightParen, // )
                TokenKind::Newline,    // \n
                TokenKind::Eof,        // EOF
            ]
        );
    }

    #[test]
    fn test_all_phase_1_tokens() {
        // Test that all Phase 1 tokens can be lexed
        let source = "x = 123\nprint(x + y - z * a / b // c % d)\n";
        let result = lex(source);
        assert!(result.is_ok());

        let tokens = result.unwrap();

        // Verify we have all token types
        let kinds: Vec<_> = tokens.iter().map(|t| t.kind).collect();
        assert!(kinds.contains(&TokenKind::Identifier));
        assert!(kinds.contains(&TokenKind::Integer));
        assert!(kinds.contains(&TokenKind::Plus));
        assert!(kinds.contains(&TokenKind::Minus));
        assert!(kinds.contains(&TokenKind::Star));
        assert!(kinds.contains(&TokenKind::Slash));
        assert!(kinds.contains(&TokenKind::DoubleSlash));
        assert!(kinds.contains(&TokenKind::Percent));
        assert!(kinds.contains(&TokenKind::LeftParen));
        assert!(kinds.contains(&TokenKind::RightParen));
        assert!(kinds.contains(&TokenKind::Equals));
        assert!(kinds.contains(&TokenKind::Print));
        assert!(kinds.contains(&TokenKind::Newline));
        assert!(kinds.contains(&TokenKind::Eof));
    }

    #[test]
    fn test_large_integer() {
        // i64::MAX = 9223372036854775807
        let tokens = lex("9223372036854775807").unwrap();
        assert_eq!(tokens[0].kind, TokenKind::Integer);
        assert_eq!(tokens[0].text, "9223372036854775807");
    }

    #[test]
    fn test_zero() {
        let tokens = lex("0").unwrap();
        assert_eq!(tokens[0].kind, TokenKind::Integer);
        assert_eq!(tokens[0].text, "0");
    }

    #[test]
    fn test_multi_digit_integer() {
        let tokens = lex("1234567890").unwrap();
        assert_eq!(tokens[0].kind, TokenKind::Integer);
        assert_eq!(tokens[0].text, "1234567890");
    }

    #[test]
    fn test_carriage_return() {
        // Carriage returns should be skipped like spaces
        let tokens = lex("x\r=\r42").unwrap();
        assert_eq!(tokens.len(), 4); // x, =, 42, eof
        assert_eq!(tokens[0].kind, TokenKind::Identifier);
        assert_eq!(tokens[1].kind, TokenKind::Equals);
        assert_eq!(tokens[2].kind, TokenKind::Integer);
    }

    #[test]
    fn test_underscore_identifier() {
        let tokens = lex("_private").unwrap();
        assert_eq!(tokens[0].kind, TokenKind::Identifier);
        assert_eq!(tokens[0].text, "_private");
    }

    #[test]
    fn test_uppercase_identifier() {
        let tokens = lex("CONSTANT").unwrap();
        assert_eq!(tokens[0].kind, TokenKind::Identifier);
        assert_eq!(tokens[0].text, "CONSTANT");
    }

    #[test]
    fn test_mixed_case_identifier() {
        let tokens = lex("camelCase").unwrap();
        assert_eq!(tokens[0].kind, TokenKind::Identifier);
        assert_eq!(tokens[0].text, "camelCase");
    }

    // ========== Function-related Token Tests ==========

    #[test]
    fn test_def_keyword() {
        let tokens = lex("def").unwrap();
        assert_eq!(tokens.len(), 2);
        assert_eq!(tokens[0].kind, TokenKind::Def);
        assert_eq!(tokens[0].text, "def");
    }

    #[test]
    fn test_return_keyword() {
        let tokens = lex("return").unwrap();
        assert_eq!(tokens.len(), 2);
        assert_eq!(tokens[0].kind, TokenKind::Return);
        assert_eq!(tokens[0].text, "return");
    }

    #[test]
    fn test_colon_token() {
        let tokens = lex(":").unwrap();
        assert_eq!(tokens.len(), 2);
        assert_eq!(tokens[0].kind, TokenKind::Colon);
        assert_eq!(tokens[0].text, ":");
    }

    #[test]
    fn test_comma_token() {
        let tokens = lex(",").unwrap();
        assert_eq!(tokens.len(), 2);
        assert_eq!(tokens[0].kind, TokenKind::Comma);
        assert_eq!(tokens[0].text, ",");
    }

    #[test]
    fn test_def_vs_identifier() {
        // "def" should be a keyword
        let tokens = lex("def").unwrap();
        assert_eq!(tokens[0].kind, TokenKind::Def);

        // "define" should be an identifier
        let tokens = lex("define").unwrap();
        assert_eq!(tokens[0].kind, TokenKind::Identifier);

        // "defy" should be an identifier
        let tokens = lex("defy").unwrap();
        assert_eq!(tokens[0].kind, TokenKind::Identifier);
    }

    #[test]
    fn test_return_vs_identifier() {
        // "return" should be a keyword
        let tokens = lex("return").unwrap();
        assert_eq!(tokens[0].kind, TokenKind::Return);

        // "returns" should be an identifier
        let tokens = lex("returns").unwrap();
        assert_eq!(tokens[0].kind, TokenKind::Identifier);

        // "returned" should be an identifier
        let tokens = lex("returned").unwrap();
        assert_eq!(tokens[0].kind, TokenKind::Identifier);
    }

    #[test]
    fn test_function_definition_tokens() {
        let tokens = lex("def foo(x, y):").unwrap();
        assert_eq!(tokens.len(), 9); // def, foo, (, x, ,, y, ), :, eof
        assert_eq!(tokens[0].kind, TokenKind::Def);
        assert_eq!(tokens[1].kind, TokenKind::Identifier);
        assert_eq!(tokens[2].kind, TokenKind::LeftParen);
        assert_eq!(tokens[3].kind, TokenKind::Identifier);
        assert_eq!(tokens[4].kind, TokenKind::Comma);
        assert_eq!(tokens[5].kind, TokenKind::Identifier);
        assert_eq!(tokens[6].kind, TokenKind::RightParen);
        assert_eq!(tokens[7].kind, TokenKind::Colon);
    }

    #[test]
    fn test_return_statement_tokens() {
        let tokens = lex("return 42").unwrap();
        assert_eq!(tokens.len(), 3); // return, 42, eof
        assert_eq!(tokens[0].kind, TokenKind::Return);
        assert_eq!(tokens[1].kind, TokenKind::Integer);
    }

    #[test]
    fn test_function_call_tokens() {
        let tokens = lex("foo(1, 2, 3)").unwrap();
        assert_eq!(tokens.len(), 9); // foo, (, 1, ,, 2, ,, 3, ), eof
        assert_eq!(tokens[0].kind, TokenKind::Identifier);
        assert_eq!(tokens[1].kind, TokenKind::LeftParen);
        assert_eq!(tokens[2].kind, TokenKind::Integer);
        assert_eq!(tokens[3].kind, TokenKind::Comma);
        assert_eq!(tokens[4].kind, TokenKind::Integer);
        assert_eq!(tokens[5].kind, TokenKind::Comma);
        assert_eq!(tokens[6].kind, TokenKind::Integer);
        assert_eq!(tokens[7].kind, TokenKind::RightParen);
        assert_eq!(tokens[8].kind, TokenKind::Eof);
    }
}
