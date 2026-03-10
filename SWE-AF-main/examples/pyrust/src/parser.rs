//! Parser using recursive descent with Pratt parsing for expressions
//!
//! Transforms a token stream into an Abstract Syntax Tree (AST).
//! Uses Pratt parsing to handle operator precedence correctly.
//! Target performance: ~10μs for 10-token input.

use crate::ast::{BinaryOperator, Expression, Program, Statement, UnaryOperator};
use crate::error::ParseError;
use crate::lexer::{Token, TokenKind};

/// Parser state for tracking position in token stream
pub struct Parser<'src> {
    /// Token stream to parse
    tokens: Vec<Token<'src>>,
    /// Current position in token stream
    pos: usize,
}

impl<'src> Parser<'src> {
    /// Creates a new parser for the given token stream
    fn new(tokens: Vec<Token<'src>>) -> Self {
        Self { tokens, pos: 0 }
    }

    /// Returns the current token without consuming it
    fn peek(&self) -> &Token<'src> {
        if self.pos < self.tokens.len() {
            &self.tokens[self.pos]
        } else {
            // Should always have EOF token at the end
            &self.tokens[self.tokens.len() - 1]
        }
    }

    /// Consumes and returns the current token
    fn advance(&mut self) -> &Token<'src> {
        let token = &self.tokens[self.pos];
        if self.pos < self.tokens.len() - 1 {
            self.pos += 1;
        }
        token
    }

    /// Checks if current token matches the expected kind
    fn check(&self, kind: TokenKind) -> bool {
        self.peek().kind == kind
    }

    /// Expects a specific token kind, returns error if not found
    fn expect(&mut self, kind: TokenKind, context: &str) -> Result<&Token<'src>, ParseError> {
        let token = self.peek();
        if token.kind == kind {
            Ok(self.advance())
        } else {
            Err(ParseError {
                message: format!("Expected {} in {}", token_kind_name(kind), context),
                line: token.line,
                column: token.column,
                found_token: token.text.to_string(),
                expected_tokens: vec![token_kind_name(kind)],
            })
        }
    }

    /// Skips newline tokens
    fn skip_newlines(&mut self) {
        while self.check(TokenKind::Newline) {
            self.advance();
        }
    }

    /// Parses a complete program
    fn parse_program(&mut self) -> Result<Program, ParseError> {
        let mut statements = Vec::new();

        // Skip leading newlines
        self.skip_newlines();

        while !self.check(TokenKind::Eof) {
            statements.push(self.parse_statement()?);
            self.skip_newlines();
        }

        Ok(Program { statements })
    }

    /// Parses a single statement
    fn parse_statement(&mut self) -> Result<Statement, ParseError> {
        // Check for function definition
        if self.check(TokenKind::Def) {
            return self.parse_function_def();
        }

        // Check for return statement
        if self.check(TokenKind::Return) {
            return self.parse_return_statement();
        }

        // Check for print statement
        if self.check(TokenKind::Print) {
            return self.parse_print_statement();
        }

        // Check for assignment (identifier followed by equals)
        if self.check(TokenKind::Identifier) {
            // Look ahead to see if this is an assignment
            if self.pos + 1 < self.tokens.len()
                && self.tokens[self.pos + 1].kind == TokenKind::Equals
            {
                return self.parse_assignment_statement();
            }
        }

        // Otherwise, it's an expression statement
        self.parse_expression_statement()
    }

    /// Parses an assignment statement: name = expression
    fn parse_assignment_statement(&mut self) -> Result<Statement, ParseError> {
        let name_token = self.expect(TokenKind::Identifier, "assignment statement")?;
        let name = name_token.text.to_string();

        self.expect(TokenKind::Equals, "assignment statement")?;

        let value = self.parse_expression()?;

        Ok(Statement::Assignment { name, value })
    }

    /// Parses a print statement: print(expression)
    fn parse_print_statement(&mut self) -> Result<Statement, ParseError> {
        self.expect(TokenKind::Print, "print statement")?;
        self.expect(TokenKind::LeftParen, "print statement")?;

        let value = self.parse_expression()?;

        self.expect(TokenKind::RightParen, "print statement")?;

        Ok(Statement::Print { value })
    }

    /// Parses an expression statement: standalone expression
    fn parse_expression_statement(&mut self) -> Result<Statement, ParseError> {
        let value = self.parse_expression()?;
        Ok(Statement::Expression { value })
    }

    /// Parses a function definition: def name(params): body
    fn parse_function_def(&mut self) -> Result<Statement, ParseError> {
        let def_token = self.expect(TokenKind::Def, "function definition")?;
        let def_indent = def_token.column;

        let name_token = self.expect(TokenKind::Identifier, "function definition")?;
        let name = name_token.text.to_string();

        self.expect(TokenKind::LeftParen, "function definition")?;

        // Parse parameter list
        let mut params = Vec::new();

        // Check if there are any parameters
        if !self.check(TokenKind::RightParen) {
            loop {
                let param_token = self.expect(TokenKind::Identifier, "function parameter list")?;
                params.push(param_token.text.to_string());

                // Check for comma (more parameters) or right paren (end of list)
                if self.check(TokenKind::Comma) {
                    self.advance();
                } else {
                    break;
                }
            }
        }

        self.expect(TokenKind::RightParen, "function definition")?;
        self.expect(TokenKind::Colon, "function definition")?;

        // Expect at least one newline after colon
        self.expect(TokenKind::Newline, "function definition")?;

        // Parse function body (indented statements)
        let mut body = Vec::new();

        // Skip any additional newlines
        self.skip_newlines();

        // Parse statements in the body until we hit EOF or a dedent
        // A dedent is when we encounter a non-empty line at the same or less indentation as the def
        while !self.check(TokenKind::Eof) {
            let token = self.peek();

            // Skip empty lines
            if self.check(TokenKind::Newline) {
                self.advance();
                continue;
            }

            // Check if this line is dedented (at or before the def indent level)
            // If so, we're done with the function body
            if token.column <= def_indent {
                break;
            }

            body.push(self.parse_statement()?);
            self.skip_newlines();
        }

        Ok(Statement::FunctionDef { name, params, body })
    }

    /// Parses a return statement: return [expression]
    fn parse_return_statement(&mut self) -> Result<Statement, ParseError> {
        self.expect(TokenKind::Return, "return statement")?;

        // Check if there's a value to return
        let value = if self.check(TokenKind::Newline) || self.check(TokenKind::Eof) {
            None
        } else {
            Some(self.parse_expression()?)
        };

        Ok(Statement::Return { value })
    }

    /// Parses a function call: name(args)
    fn parse_call(&mut self, name: String) -> Result<Expression, ParseError> {
        self.expect(TokenKind::LeftParen, "function call")?;

        let mut args = Vec::new();

        // Check if there are any arguments
        if !self.check(TokenKind::RightParen) {
            loop {
                args.push(self.parse_expression()?);

                // Check for comma (more arguments) or right paren (end of list)
                if self.check(TokenKind::Comma) {
                    self.advance();
                } else {
                    break;
                }
            }
        }

        self.expect(TokenKind::RightParen, "function call")?;

        Ok(Expression::Call { name, args })
    }

    /// Parses an expression using Pratt parsing
    fn parse_expression(&mut self) -> Result<Expression, ParseError> {
        self.parse_expression_with_precedence(0)
    }

    /// Parses an expression with minimum precedence (Pratt parsing)
    fn parse_expression_with_precedence(
        &mut self,
        min_precedence: u8,
    ) -> Result<Expression, ParseError> {
        // Parse left-hand side (prefix expression)
        let mut left = self.parse_primary()?;

        // Parse binary operators with precedence climbing
        loop {
            let token = self.peek();

            // Check if current token is a binary operator
            let op = match token.kind {
                TokenKind::Plus => BinaryOperator::Add,
                TokenKind::Minus => BinaryOperator::Sub,
                TokenKind::Star => BinaryOperator::Mul,
                TokenKind::Slash => BinaryOperator::Div,
                TokenKind::DoubleSlash => BinaryOperator::FloorDiv,
                TokenKind::Percent => BinaryOperator::Mod,
                _ => break, // Not a binary operator, done parsing
            };

            let precedence = op.precedence();

            // If precedence is too low, stop parsing
            if precedence < min_precedence {
                break;
            }

            // Consume the operator
            self.advance();

            // Parse right-hand side with higher precedence
            // Use precedence + 1 for left-associativity
            let right = self.parse_expression_with_precedence(precedence + 1)?;

            // Build binary operation
            left = Expression::BinaryOp {
                left: Box::new(left),
                op,
                right: Box::new(right),
            };
        }

        Ok(left)
    }

    /// Parses a primary expression (integer, variable, or parenthesized expression)
    fn parse_primary(&mut self) -> Result<Expression, ParseError> {
        let token = *self.peek();

        match token.kind {
            TokenKind::Plus | TokenKind::Minus => {
                // Handle unary operators
                let op = if token.kind == TokenKind::Plus {
                    UnaryOperator::Pos
                } else {
                    UnaryOperator::Neg
                };
                self.advance();

                // Parse the operand
                let operand = self.parse_primary()?;

                Ok(Expression::UnaryOp {
                    op,
                    operand: Box::new(operand),
                })
            }

            TokenKind::Integer => {
                let text = token.text;
                let line = token.line;
                let column = token.column;
                self.advance();

                // Parse the integer value
                let value = text.parse::<i64>().map_err(|_| ParseError {
                    message: format!("Integer literal '{}' is too large", text),
                    line,
                    column,
                    found_token: text.to_string(),
                    expected_tokens: vec!["valid integer".to_string()],
                })?;

                Ok(Expression::Integer(value))
            }

            TokenKind::Identifier => {
                let name = token.text.to_string();
                self.advance();

                // Check if this is a function call (identifier followed by left paren)
                if self.check(TokenKind::LeftParen) {
                    return self.parse_call(name);
                }

                Ok(Expression::Variable(name))
            }

            TokenKind::LeftParen => {
                self.advance();
                let expr = self.parse_expression()?;
                self.expect(TokenKind::RightParen, "parenthesized expression")?;
                Ok(expr)
            }

            _ => Err(ParseError {
                message: "Expected expression".to_string(),
                line: token.line,
                column: token.column,
                found_token: token.text.to_string(),
                expected_tokens: vec![
                    "integer".to_string(),
                    "identifier".to_string(),
                    "'('".to_string(),
                ],
            }),
        }
    }
}

/// Returns a human-readable name for a token kind
fn token_kind_name(kind: TokenKind) -> String {
    match kind {
        TokenKind::Integer => "integer".to_string(),
        TokenKind::Identifier => "identifier".to_string(),
        TokenKind::Plus => "'+'".to_string(),
        TokenKind::Minus => "'-'".to_string(),
        TokenKind::Star => "'*'".to_string(),
        TokenKind::Slash => "'/'".to_string(),
        TokenKind::DoubleSlash => "'//'".to_string(),
        TokenKind::Percent => "'%'".to_string(),
        TokenKind::LeftParen => "'('".to_string(),
        TokenKind::RightParen => "')'".to_string(),
        TokenKind::Colon => "':'".to_string(),
        TokenKind::Comma => "','".to_string(),
        TokenKind::Equals => "'='".to_string(),
        TokenKind::Print => "'print'".to_string(),
        TokenKind::Def => "'def'".to_string(),
        TokenKind::Return => "'return'".to_string(),
        TokenKind::Newline => "newline".to_string(),
        TokenKind::Eof => "end of file".to_string(),
    }
}

/// Parses a token stream into a Program AST
///
/// This is the main entry point for parsing. It uses recursive descent
/// with Pratt parsing for expression precedence.
///
/// # Arguments
/// * `tokens` - Vector of tokens from the lexer (must include EOF token)
///
/// # Returns
/// * `Ok(Program)` - Successfully parsed AST
/// * `Err(ParseError)` - Error with location information if parsing fails
///
/// # Examples
/// ```
/// use pyrust::lexer::lex;
/// use pyrust::parser::parse;
///
/// let tokens = lex("x = 42").unwrap();
/// let program = parse(tokens).unwrap();
/// assert_eq!(program.statements.len(), 1);
/// ```
pub fn parse(tokens: Vec<Token>) -> Result<Program, ParseError> {
    let mut parser = Parser::new(tokens);
    parser.parse_program()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::lex;

    #[test]
    fn test_parse_integer_literal() {
        let tokens = lex("42").unwrap();
        let program = parse(tokens).unwrap();

        assert_eq!(program.statements.len(), 1);
        match &program.statements[0] {
            Statement::Expression { value } => {
                assert_eq!(*value, Expression::Integer(42));
            }
            _ => panic!("Expected expression statement"),
        }
    }

    #[test]
    fn test_parse_variable() {
        let tokens = lex("x").unwrap();
        let program = parse(tokens).unwrap();

        assert_eq!(program.statements.len(), 1);
        match &program.statements[0] {
            Statement::Expression { value } => {
                assert_eq!(*value, Expression::Variable("x".to_string()));
            }
            _ => panic!("Expected expression statement"),
        }
    }

    #[test]
    fn test_parse_assignment() {
        let tokens = lex("x = 42").unwrap();
        let program = parse(tokens).unwrap();

        assert_eq!(program.statements.len(), 1);
        match &program.statements[0] {
            Statement::Assignment { name, value } => {
                assert_eq!(name, "x");
                assert_eq!(*value, Expression::Integer(42));
            }
            _ => panic!("Expected assignment statement"),
        }
    }

    #[test]
    fn test_parse_print() {
        let tokens = lex("print(42)").unwrap();
        let program = parse(tokens).unwrap();

        assert_eq!(program.statements.len(), 1);
        match &program.statements[0] {
            Statement::Print { value } => {
                assert_eq!(*value, Expression::Integer(42));
            }
            _ => panic!("Expected print statement"),
        }
    }

    #[test]
    fn test_parse_addition() {
        let tokens = lex("1 + 2").unwrap();
        let program = parse(tokens).unwrap();

        assert_eq!(program.statements.len(), 1);
        match &program.statements[0] {
            Statement::Expression { value } => match value {
                Expression::BinaryOp { left, op, right } => {
                    assert_eq!(**left, Expression::Integer(1));
                    assert_eq!(*op, BinaryOperator::Add);
                    assert_eq!(**right, Expression::Integer(2));
                }
                _ => panic!("Expected binary operation"),
            },
            _ => panic!("Expected expression statement"),
        }
    }

    #[test]
    fn test_parse_precedence_multiplication_before_addition() {
        // 1 + 2 * 3 should parse as 1 + (2 * 3)
        let tokens = lex("1 + 2 * 3").unwrap();
        let program = parse(tokens).unwrap();

        assert_eq!(program.statements.len(), 1);
        match &program.statements[0] {
            Statement::Expression { value } => {
                match value {
                    Expression::BinaryOp { left, op, right } => {
                        assert_eq!(**left, Expression::Integer(1));
                        assert_eq!(*op, BinaryOperator::Add);

                        // Right side should be 2 * 3
                        match &**right {
                            Expression::BinaryOp {
                                left: inner_left,
                                op: inner_op,
                                right: inner_right,
                            } => {
                                assert_eq!(**inner_left, Expression::Integer(2));
                                assert_eq!(*inner_op, BinaryOperator::Mul);
                                assert_eq!(**inner_right, Expression::Integer(3));
                            }
                            _ => panic!("Expected multiplication on right side"),
                        }
                    }
                    _ => panic!("Expected binary operation"),
                }
            }
            _ => panic!("Expected expression statement"),
        }
    }

    #[test]
    fn test_parse_precedence_division_before_subtraction() {
        // 10 - 4 / 2 should parse as 10 - (4 / 2)
        let tokens = lex("10 - 4 / 2").unwrap();
        let program = parse(tokens).unwrap();

        match &program.statements[0] {
            Statement::Expression { value } => match value {
                Expression::BinaryOp { left, op, right } => {
                    assert_eq!(**left, Expression::Integer(10));
                    assert_eq!(*op, BinaryOperator::Sub);

                    match &**right {
                        Expression::BinaryOp {
                            left: inner_left,
                            op: inner_op,
                            right: inner_right,
                        } => {
                            assert_eq!(**inner_left, Expression::Integer(4));
                            assert_eq!(*inner_op, BinaryOperator::Div);
                            assert_eq!(**inner_right, Expression::Integer(2));
                        }
                        _ => panic!("Expected division on right side"),
                    }
                }
                _ => panic!("Expected binary operation"),
            },
            _ => panic!("Expected expression statement"),
        }
    }

    #[test]
    fn test_parse_left_associativity() {
        // 1 - 2 - 3 should parse as (1 - 2) - 3
        let tokens = lex("1 - 2 - 3").unwrap();
        let program = parse(tokens).unwrap();

        match &program.statements[0] {
            Statement::Expression { value } => match value {
                Expression::BinaryOp { left, op, right } => {
                    assert_eq!(**right, Expression::Integer(3));
                    assert_eq!(*op, BinaryOperator::Sub);

                    match &**left {
                        Expression::BinaryOp {
                            left: inner_left,
                            op: inner_op,
                            right: inner_right,
                        } => {
                            assert_eq!(**inner_left, Expression::Integer(1));
                            assert_eq!(*inner_op, BinaryOperator::Sub);
                            assert_eq!(**inner_right, Expression::Integer(2));
                        }
                        _ => panic!("Expected subtraction on left side"),
                    }
                }
                _ => panic!("Expected binary operation"),
            },
            _ => panic!("Expected expression statement"),
        }
    }

    #[test]
    fn test_parse_parentheses() {
        // (1 + 2) * 3 should parse as (1 + 2) * 3, not 1 + (2 * 3)
        let tokens = lex("(1 + 2) * 3").unwrap();
        let program = parse(tokens).unwrap();

        match &program.statements[0] {
            Statement::Expression { value } => {
                match value {
                    Expression::BinaryOp { left, op, right } => {
                        assert_eq!(**right, Expression::Integer(3));
                        assert_eq!(*op, BinaryOperator::Mul);

                        // Left side should be 1 + 2
                        match &**left {
                            Expression::BinaryOp {
                                left: inner_left,
                                op: inner_op,
                                right: inner_right,
                            } => {
                                assert_eq!(**inner_left, Expression::Integer(1));
                                assert_eq!(*inner_op, BinaryOperator::Add);
                                assert_eq!(**inner_right, Expression::Integer(2));
                            }
                            _ => panic!("Expected addition on left side"),
                        }
                    }
                    _ => panic!("Expected binary operation"),
                }
            }
            _ => panic!("Expected expression statement"),
        }
    }

    #[test]
    fn test_parse_all_operators() {
        // Test all 6 binary operators
        let test_cases = vec![
            ("1 + 2", BinaryOperator::Add),
            ("1 - 2", BinaryOperator::Sub),
            ("1 * 2", BinaryOperator::Mul),
            ("1 / 2", BinaryOperator::Div),
            ("1 // 2", BinaryOperator::FloorDiv),
            ("1 % 2", BinaryOperator::Mod),
        ];

        for (source, expected_op) in test_cases {
            let tokens = lex(source).unwrap();
            let program = parse(tokens).unwrap();

            match &program.statements[0] {
                Statement::Expression { value } => match value {
                    Expression::BinaryOp { op, .. } => {
                        assert_eq!(*op, expected_op, "Failed for source: {}", source);
                    }
                    _ => panic!("Expected binary operation for: {}", source),
                },
                _ => panic!("Expected expression statement for: {}", source),
            }
        }
    }

    #[test]
    fn test_parse_complex_expression() {
        // (a + b) * c - d / e % f
        let tokens = lex("(a + b) * c - d / e % f").unwrap();
        let program = parse(tokens).unwrap();

        assert_eq!(program.statements.len(), 1);
        // Just verify it parses without error
    }

    #[test]
    fn test_parse_multiple_statements() {
        let tokens = lex("x = 10\ny = 20\nprint(x)").unwrap();
        let program = parse(tokens).unwrap();

        assert_eq!(program.statements.len(), 3);

        // First statement: x = 10
        match &program.statements[0] {
            Statement::Assignment { name, .. } => assert_eq!(name, "x"),
            _ => panic!("Expected assignment"),
        }

        // Second statement: y = 20
        match &program.statements[1] {
            Statement::Assignment { name, .. } => assert_eq!(name, "y"),
            _ => panic!("Expected assignment"),
        }

        // Third statement: print(x)
        match &program.statements[2] {
            Statement::Print { .. } => {}
            _ => panic!("Expected print"),
        }
    }

    #[test]
    fn test_parse_error_unexpected_token() {
        let tokens = lex("x = +").unwrap();
        let result = parse(tokens);

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.line, 1);
        // With unary operator support, + is parsed as unary operator,
        // then it looks for operand and finds EOF at column 6
        assert_eq!(err.column, 6);
        assert_eq!(err.found_token, "");
        assert!(err.expected_tokens.contains(&"integer".to_string()));
    }

    #[test]
    fn test_parse_error_missing_right_paren() {
        let tokens = lex("print(42").unwrap();
        let result = parse(tokens);

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.message.contains("Expected"));
        assert!(err.expected_tokens.contains(&"')'".to_string()));
    }

    #[test]
    fn test_parse_error_missing_expression_in_print() {
        let tokens = lex("print()").unwrap();
        let result = parse(tokens);

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.message.contains("Expected expression"));
    }

    #[test]
    fn test_parse_empty_program() {
        let tokens = lex("").unwrap();
        let program = parse(tokens).unwrap();

        assert_eq!(program.statements.len(), 0);
    }

    #[test]
    fn test_parse_newlines_only() {
        let tokens = lex("\n\n\n").unwrap();
        let program = parse(tokens).unwrap();

        assert_eq!(program.statements.len(), 0);
    }

    #[test]
    fn test_parse_with_leading_newlines() {
        let tokens = lex("\n\nx = 42").unwrap();
        let program = parse(tokens).unwrap();

        assert_eq!(program.statements.len(), 1);
    }

    #[test]
    fn test_parse_with_trailing_newlines() {
        let tokens = lex("x = 42\n\n").unwrap();
        let program = parse(tokens).unwrap();

        assert_eq!(program.statements.len(), 1);
    }

    #[test]
    fn test_parse_print_with_variable() {
        let tokens = lex("print(x)").unwrap();
        let program = parse(tokens).unwrap();

        match &program.statements[0] {
            Statement::Print { value } => {
                assert_eq!(*value, Expression::Variable("x".to_string()));
            }
            _ => panic!("Expected print statement"),
        }
    }

    #[test]
    fn test_parse_print_with_expression() {
        let tokens = lex("print(1 + 2)").unwrap();
        let program = parse(tokens).unwrap();

        match &program.statements[0] {
            Statement::Print { value } => match value {
                Expression::BinaryOp { left, op, right } => {
                    assert_eq!(**left, Expression::Integer(1));
                    assert_eq!(*op, BinaryOperator::Add);
                    assert_eq!(**right, Expression::Integer(2));
                }
                _ => panic!("Expected binary operation"),
            },
            _ => panic!("Expected print statement"),
        }
    }

    #[test]
    fn test_parse_assignment_with_expression() {
        let tokens = lex("result = 2 * 3 + 4").unwrap();
        let program = parse(tokens).unwrap();

        match &program.statements[0] {
            Statement::Assignment { name, value } => {
                assert_eq!(name, "result");
                // Verify it's a valid expression (should be 2 * 3 + 4 = (2 * 3) + 4)
                match value {
                    Expression::BinaryOp { op, .. } => {
                        assert_eq!(*op, BinaryOperator::Add);
                    }
                    _ => panic!("Expected binary operation"),
                }
            }
            _ => panic!("Expected assignment statement"),
        }
    }

    #[test]
    fn test_parse_nested_parentheses() {
        let tokens = lex("((1 + 2))").unwrap();
        let program = parse(tokens).unwrap();

        match &program.statements[0] {
            Statement::Expression { value } => match value {
                Expression::BinaryOp { left, op, right } => {
                    assert_eq!(**left, Expression::Integer(1));
                    assert_eq!(*op, BinaryOperator::Add);
                    assert_eq!(**right, Expression::Integer(2));
                }
                _ => panic!("Expected binary operation"),
            },
            _ => panic!("Expected expression statement"),
        }
    }

    #[test]
    fn test_precedence_same_level_left_associative() {
        // 5 - 3 + 2 should be (5 - 3) + 2 = 4
        let tokens = lex("5 - 3 + 2").unwrap();
        let program = parse(tokens).unwrap();

        match &program.statements[0] {
            Statement::Expression { value } => match value {
                Expression::BinaryOp { left, op, right } => {
                    assert_eq!(*op, BinaryOperator::Add);
                    assert_eq!(**right, Expression::Integer(2));

                    match &**left {
                        Expression::BinaryOp {
                            left: l,
                            op: o,
                            right: r,
                        } => {
                            assert_eq!(**l, Expression::Integer(5));
                            assert_eq!(*o, BinaryOperator::Sub);
                            assert_eq!(**r, Expression::Integer(3));
                        }
                        _ => panic!("Expected subtraction on left"),
                    }
                }
                _ => panic!("Expected binary operation"),
            },
            _ => panic!("Expected expression statement"),
        }
    }

    #[test]
    fn test_complex_precedence() {
        // 1 + 2 * 3 - 4 / 2 should be 1 + (2 * 3) - (4 / 2)
        let tokens = lex("1 + 2 * 3 - 4 / 2").unwrap();
        let program = parse(tokens).unwrap();

        // Just verify it parses correctly
        assert_eq!(program.statements.len(), 1);
    }

    #[test]
    fn test_error_location_information() {
        // Error at specific location - unary operator without operand
        let tokens = lex("x = +").unwrap();
        let result = parse(tokens);

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.line, 1);
        // With unary operator support, + is parsed as unary operator,
        // then it looks for operand and finds EOF at column 6
        assert_eq!(err.column, 6);
        assert!(err.found_token.is_empty());
        assert!(err.expected_tokens.contains(&"integer".to_string()));
    }

    // ========== Function Definition Tests ==========

    #[test]
    fn test_parse_function_def_no_params() {
        let tokens = lex("def foo():\n    return 42").unwrap();
        let program = parse(tokens).unwrap();

        assert_eq!(program.statements.len(), 1);
        match &program.statements[0] {
            Statement::FunctionDef { name, params, body } => {
                assert_eq!(name, "foo");
                assert_eq!(params.len(), 0);
                assert_eq!(body.len(), 1);
                match &body[0] {
                    Statement::Return { value } => {
                        assert!(value.is_some());
                        assert_eq!(value.as_ref().unwrap(), &Expression::Integer(42));
                    }
                    _ => panic!("Expected return statement"),
                }
            }
            _ => panic!("Expected function definition"),
        }
    }

    #[test]
    fn test_parse_function_def_one_param() {
        let tokens = lex("def square(x):\n    return x").unwrap();
        let program = parse(tokens).unwrap();

        match &program.statements[0] {
            Statement::FunctionDef { name, params, body } => {
                assert_eq!(name, "square");
                assert_eq!(params.len(), 1);
                assert_eq!(params[0], "x");
                assert_eq!(body.len(), 1);
            }
            _ => panic!("Expected function definition"),
        }
    }

    #[test]
    fn test_parse_function_def_multiple_params() {
        let tokens = lex("def add(a, b, c):\n    return a").unwrap();
        let program = parse(tokens).unwrap();

        match &program.statements[0] {
            Statement::FunctionDef {
                name,
                params,
                body: _,
            } => {
                assert_eq!(name, "add");
                assert_eq!(params.len(), 3);
                assert_eq!(params[0], "a");
                assert_eq!(params[1], "b");
                assert_eq!(params[2], "c");
            }
            _ => panic!("Expected function definition"),
        }
    }

    #[test]
    fn test_parse_function_def_with_body_statements() {
        let tokens = lex("def foo(x):\n    y = x + 1\n    print(y)\n    return y").unwrap();
        let program = parse(tokens).unwrap();

        match &program.statements[0] {
            Statement::FunctionDef { name, params, body } => {
                assert_eq!(name, "foo");
                assert_eq!(params.len(), 1);
                assert_eq!(body.len(), 3);
                assert!(matches!(body[0], Statement::Assignment { .. }));
                assert!(matches!(body[1], Statement::Print { .. }));
                assert!(matches!(body[2], Statement::Return { .. }));
            }
            _ => panic!("Expected function definition"),
        }
    }

    #[test]
    fn test_parse_function_def_empty_body() {
        // Empty body (just newlines after colon)
        let tokens = lex("def foo():\n\n\n").unwrap();
        let program = parse(tokens).unwrap();

        match &program.statements[0] {
            Statement::FunctionDef { name, params, body } => {
                assert_eq!(name, "foo");
                assert_eq!(params.len(), 0);
                assert_eq!(body.len(), 0);
            }
            _ => panic!("Expected function definition"),
        }
    }

    // ========== Return Statement Tests ==========

    #[test]
    fn test_parse_return_with_value() {
        let tokens = lex("def foo():\n    return 42").unwrap();
        let program = parse(tokens).unwrap();

        match &program.statements[0] {
            Statement::FunctionDef { body, .. } => match &body[0] {
                Statement::Return { value } => {
                    assert!(value.is_some());
                    assert_eq!(value.as_ref().unwrap(), &Expression::Integer(42));
                }
                _ => panic!("Expected return statement"),
            },
            _ => panic!("Expected function definition"),
        }
    }

    #[test]
    fn test_parse_return_without_value() {
        let tokens = lex("def foo():\n    return").unwrap();
        let program = parse(tokens).unwrap();

        match &program.statements[0] {
            Statement::FunctionDef { body, .. } => match &body[0] {
                Statement::Return { value } => {
                    assert!(value.is_none());
                }
                _ => panic!("Expected return statement"),
            },
            _ => panic!("Expected function definition"),
        }
    }

    #[test]
    fn test_parse_return_with_expression() {
        let tokens = lex("def foo():\n    return 1 + 2").unwrap();
        let program = parse(tokens).unwrap();

        match &program.statements[0] {
            Statement::FunctionDef { body, .. } => match &body[0] {
                Statement::Return { value } => {
                    assert!(value.is_some());
                    match value.as_ref().unwrap() {
                        Expression::BinaryOp { left, op, right } => {
                            assert_eq!(**left, Expression::Integer(1));
                            assert_eq!(*op, BinaryOperator::Add);
                            assert_eq!(**right, Expression::Integer(2));
                        }
                        _ => panic!("Expected binary operation"),
                    }
                }
                _ => panic!("Expected return statement"),
            },
            _ => panic!("Expected function definition"),
        }
    }

    // ========== Function Call Tests ==========

    #[test]
    fn test_parse_function_call_no_args() {
        let tokens = lex("foo()").unwrap();
        let program = parse(tokens).unwrap();

        assert_eq!(program.statements.len(), 1);
        match &program.statements[0] {
            Statement::Expression { value } => match value {
                Expression::Call { name, args } => {
                    assert_eq!(name, "foo");
                    assert_eq!(args.len(), 0);
                }
                _ => panic!("Expected function call"),
            },
            _ => panic!("Expected expression statement"),
        }
    }

    #[test]
    fn test_parse_function_call_one_arg() {
        let tokens = lex("foo(42)").unwrap();
        let program = parse(tokens).unwrap();

        match &program.statements[0] {
            Statement::Expression { value } => match value {
                Expression::Call { name, args } => {
                    assert_eq!(name, "foo");
                    assert_eq!(args.len(), 1);
                    assert_eq!(args[0], Expression::Integer(42));
                }
                _ => panic!("Expected function call"),
            },
            _ => panic!("Expected expression statement"),
        }
    }

    #[test]
    fn test_parse_function_call_multiple_args() {
        let tokens = lex("add(10, 20, 30)").unwrap();
        let program = parse(tokens).unwrap();

        match &program.statements[0] {
            Statement::Expression { value } => match value {
                Expression::Call { name, args } => {
                    assert_eq!(name, "add");
                    assert_eq!(args.len(), 3);
                    assert_eq!(args[0], Expression::Integer(10));
                    assert_eq!(args[1], Expression::Integer(20));
                    assert_eq!(args[2], Expression::Integer(30));
                }
                _ => panic!("Expected function call"),
            },
            _ => panic!("Expected expression statement"),
        }
    }

    #[test]
    fn test_parse_function_call_with_variables() {
        let tokens = lex("foo(x, y)").unwrap();
        let program = parse(tokens).unwrap();

        match &program.statements[0] {
            Statement::Expression { value } => match value {
                Expression::Call { name, args } => {
                    assert_eq!(name, "foo");
                    assert_eq!(args.len(), 2);
                    assert_eq!(args[0], Expression::Variable("x".to_string()));
                    assert_eq!(args[1], Expression::Variable("y".to_string()));
                }
                _ => panic!("Expected function call"),
            },
            _ => panic!("Expected expression statement"),
        }
    }

    #[test]
    fn test_parse_function_call_with_expression_args() {
        let tokens = lex("foo(1 + 2, x * 3)").unwrap();
        let program = parse(tokens).unwrap();

        match &program.statements[0] {
            Statement::Expression { value } => match value {
                Expression::Call { name, args } => {
                    assert_eq!(name, "foo");
                    assert_eq!(args.len(), 2);
                    assert!(matches!(args[0], Expression::BinaryOp { .. }));
                    assert!(matches!(args[1], Expression::BinaryOp { .. }));
                }
                _ => panic!("Expected function call"),
            },
            _ => panic!("Expected expression statement"),
        }
    }

    #[test]
    fn test_parse_nested_function_calls() {
        let tokens = lex("foo(bar(1), baz(2, 3))").unwrap();
        let program = parse(tokens).unwrap();

        match &program.statements[0] {
            Statement::Expression { value } => {
                match value {
                    Expression::Call { name, args } => {
                        assert_eq!(name, "foo");
                        assert_eq!(args.len(), 2);

                        // First arg is bar(1)
                        match &args[0] {
                            Expression::Call {
                                name: inner_name,
                                args: inner_args,
                            } => {
                                assert_eq!(inner_name, "bar");
                                assert_eq!(inner_args.len(), 1);
                            }
                            _ => panic!("Expected nested call"),
                        }

                        // Second arg is baz(2, 3)
                        match &args[1] {
                            Expression::Call {
                                name: inner_name,
                                args: inner_args,
                            } => {
                                assert_eq!(inner_name, "baz");
                                assert_eq!(inner_args.len(), 2);
                            }
                            _ => panic!("Expected nested call"),
                        }
                    }
                    _ => panic!("Expected function call"),
                }
            }
            _ => panic!("Expected expression statement"),
        }
    }

    #[test]
    fn test_parse_function_call_in_return() {
        let tokens = lex("def foo():\n    return bar()").unwrap();
        let program = parse(tokens).unwrap();

        match &program.statements[0] {
            Statement::FunctionDef { body, .. } => match &body[0] {
                Statement::Return { value } => {
                    assert!(value.is_some());
                    match value.as_ref().unwrap() {
                        Expression::Call { name, args } => {
                            assert_eq!(name, "bar");
                            assert_eq!(args.len(), 0);
                        }
                        _ => panic!("Expected call expression"),
                    }
                }
                _ => panic!("Expected return statement"),
            },
            _ => panic!("Expected function definition"),
        }
    }

    // ========== Error Cases ==========

    #[test]
    fn test_parse_function_error_missing_colon() {
        let tokens = lex("def foo()\n    return 42").unwrap();
        let result = parse(tokens);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.message.contains("':'"));
    }

    #[test]
    fn test_parse_function_error_missing_paren() {
        let tokens = lex("def foo(:\n    return 42").unwrap();
        let result = parse(tokens);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_function_call_error_missing_right_paren() {
        let tokens = lex("foo(1, 2").unwrap();
        let result = parse(tokens);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.message.contains("')'"));
    }

    // ========== Edge Cases ==========

    #[test]
    fn test_parse_multiple_function_definitions() {
        let tokens = lex("def foo():\n    return 1\ndef bar():\n    return 2").unwrap();
        let program = parse(tokens).unwrap();

        assert_eq!(program.statements.len(), 2);
        assert!(matches!(
            program.statements[0],
            Statement::FunctionDef { .. }
        ));
        assert!(matches!(
            program.statements[1],
            Statement::FunctionDef { .. }
        ));
    }

    #[test]
    fn test_parse_function_and_call() {
        let tokens = lex("def foo():\n    return 42\nfoo()").unwrap();
        let program = parse(tokens).unwrap();

        assert_eq!(program.statements.len(), 2);
        match &program.statements[0] {
            Statement::FunctionDef { name, .. } => assert_eq!(name, "foo"),
            _ => panic!("Expected function definition"),
        }
        match &program.statements[1] {
            Statement::Expression { value } => match value {
                Expression::Call { name, .. } => assert_eq!(name, "foo"),
                _ => panic!("Expected call"),
            },
            _ => panic!("Expected expression statement"),
        }
    }

    #[test]
    fn test_parse_function_call_in_print() {
        let tokens = lex("print(foo())").unwrap();
        let program = parse(tokens).unwrap();

        match &program.statements[0] {
            Statement::Print { value } => match value {
                Expression::Call { name, args } => {
                    assert_eq!(name, "foo");
                    assert_eq!(args.len(), 0);
                }
                _ => panic!("Expected call expression"),
            },
            _ => panic!("Expected print statement"),
        }
    }

    #[test]
    fn test_parse_function_call_in_assignment() {
        let tokens = lex("x = foo()").unwrap();
        let program = parse(tokens).unwrap();

        match &program.statements[0] {
            Statement::Assignment { name, value } => {
                assert_eq!(name, "x");
                match value {
                    Expression::Call { name, args } => {
                        assert_eq!(name, "foo");
                        assert_eq!(args.len(), 0);
                    }
                    _ => panic!("Expected call expression"),
                }
            }
            _ => panic!("Expected assignment statement"),
        }
    }

    // ========== Indentation Tracking Tests ==========

    #[test]
    fn test_parse_nested_functions() {
        // Nested function definitions (outer at column 1, inner indented)
        let source = "def outer():\n    def inner():\n        return 1\n    return inner()";
        let tokens = lex(source).unwrap();
        let program = parse(tokens).unwrap();

        assert_eq!(program.statements.len(), 1);
        match &program.statements[0] {
            Statement::FunctionDef { name, body, .. } => {
                assert_eq!(name, "outer");
                assert_eq!(body.len(), 2);
                assert!(matches!(body[0], Statement::FunctionDef { .. }));
                assert!(matches!(body[1], Statement::Return { .. }));
            }
            _ => panic!("Expected outer function definition"),
        }
    }

    #[test]
    fn test_parse_function_stops_at_dedent() {
        // Function body should stop when we hit a line at the same indent as def
        let source = "def foo():\n    return 1\nx = 2";
        let tokens = lex(source).unwrap();
        let program = parse(tokens).unwrap();

        assert_eq!(program.statements.len(), 2);
        match &program.statements[0] {
            Statement::FunctionDef { name, body, .. } => {
                assert_eq!(name, "foo");
                assert_eq!(body.len(), 1);
            }
            _ => panic!("Expected function definition"),
        }
        assert!(matches!(
            program.statements[1],
            Statement::Assignment { .. }
        ));
    }

    #[test]
    fn test_parse_function_at_indent_level_4() {
        // Function defined at column 5 (4 spaces of indentation)
        let source = "    def foo():\n        return 1";
        let tokens = lex(source).unwrap();
        let program = parse(tokens).unwrap();

        assert_eq!(program.statements.len(), 1);
        match &program.statements[0] {
            Statement::FunctionDef { name, body, .. } => {
                assert_eq!(name, "foo");
                assert_eq!(body.len(), 1);
            }
            _ => panic!("Expected function definition"),
        }
    }

    #[test]
    fn test_parse_function_body_with_blank_lines() {
        // Blank lines in function body shouldn't end the function
        let source = "def foo():\n    x = 1\n\n    y = 2\n    return y";
        let tokens = lex(source).unwrap();
        let program = parse(tokens).unwrap();

        match &program.statements[0] {
            Statement::FunctionDef { name, body, .. } => {
                assert_eq!(name, "foo");
                assert_eq!(body.len(), 3);
            }
            _ => panic!("Expected function definition"),
        }
    }

    #[test]
    fn test_parse_deeply_nested_functions() {
        // Three levels of function nesting
        let source = "def level1():\n    def level2():\n        def level3():\n            return 42\n        return level3()\n    return level2()";
        let tokens = lex(source).unwrap();
        let program = parse(tokens).unwrap();

        assert_eq!(program.statements.len(), 1);
        match &program.statements[0] {
            Statement::FunctionDef {
                name: name1,
                body: body1,
                ..
            } => {
                assert_eq!(name1, "level1");
                assert_eq!(body1.len(), 2);

                match &body1[0] {
                    Statement::FunctionDef {
                        name: name2,
                        body: body2,
                        ..
                    } => {
                        assert_eq!(name2, "level2");
                        assert_eq!(body2.len(), 2);

                        match &body2[0] {
                            Statement::FunctionDef {
                                name: name3,
                                body: body3,
                                ..
                            } => {
                                assert_eq!(name3, "level3");
                                assert_eq!(body3.len(), 1);
                            }
                            _ => panic!("Expected level3 function"),
                        }
                    }
                    _ => panic!("Expected level2 function"),
                }
            }
            _ => panic!("Expected level1 function"),
        }
    }

    #[test]
    fn test_parse_indented_function_followed_by_statements() {
        // Indented function followed by statement at lower indent
        let source = "    def foo():\n        return 1\n    x = 2\ny = 3";
        let tokens = lex(source).unwrap();
        let program = parse(tokens).unwrap();

        assert_eq!(program.statements.len(), 3);
        match &program.statements[0] {
            Statement::FunctionDef { name, body, .. } => {
                assert_eq!(name, "foo");
                assert_eq!(body.len(), 1);
            }
            _ => panic!("Expected function definition"),
        }
        assert!(matches!(
            program.statements[1],
            Statement::Assignment { .. }
        ));
        assert!(matches!(
            program.statements[2],
            Statement::Assignment { .. }
        ));
    }

    #[test]
    fn test_parse_function_with_mixed_indent_in_body() {
        // Function body where statements have varying indentation (all > def indent)
        let source = "def foo():\n    x = 1\n        y = 2\n    return x";
        let tokens = lex(source).unwrap();
        let program = parse(tokens).unwrap();

        match &program.statements[0] {
            Statement::FunctionDef { body, .. } => {
                assert_eq!(body.len(), 3);
            }
            _ => panic!("Expected function definition"),
        }
    }

    #[test]
    fn test_parse_function_complex_nesting() {
        // Complex scenario: function with nested function and statements after both
        let source = "def outer():\n    x = 1\n    def inner():\n        y = 2\n        return y\n    return inner()\nz = 3";
        let tokens = lex(source).unwrap();
        let program = parse(tokens).unwrap();

        assert_eq!(program.statements.len(), 2);
        match &program.statements[0] {
            Statement::FunctionDef { name, body, .. } => {
                assert_eq!(name, "outer");
                assert_eq!(body.len(), 3);
            }
            _ => panic!("Expected outer function"),
        }
        assert!(matches!(
            program.statements[1],
            Statement::Assignment { .. }
        ));
    }

    #[test]
    fn test_parse_function_with_deeply_indented_statement() {
        // Function with a single deeply indented statement
        let source = "def foo():\n            return 42";
        let tokens = lex(source).unwrap();
        let program = parse(tokens).unwrap();

        match &program.statements[0] {
            Statement::FunctionDef { name, body, .. } => {
                assert_eq!(name, "foo");
                assert_eq!(body.len(), 1);
            }
            _ => panic!("Expected function definition"),
        }
    }

    #[test]
    fn test_parse_two_functions_at_same_indent() {
        // Two functions at the same indentation level (column 5)
        let source = "    def foo():\n        return 1\n    def bar():\n        return 2";
        let tokens = lex(source).unwrap();
        let program = parse(tokens).unwrap();

        assert_eq!(program.statements.len(), 2);
        match &program.statements[0] {
            Statement::FunctionDef { name, body, .. } => {
                assert_eq!(name, "foo");
                assert_eq!(body.len(), 1);
            }
            _ => panic!("Expected function definition"),
        }
        match &program.statements[1] {
            Statement::FunctionDef { name, body, .. } => {
                assert_eq!(name, "bar");
                assert_eq!(body.len(), 1);
            }
            _ => panic!("Expected function definition"),
        }
    }

    #[test]
    fn test_parse_function_followed_by_same_indent() {
        // Function should end when we encounter a statement at the same indent
        let source = "def foo():\n    x = 1\n    return x\nprint(42)";
        let tokens = lex(source).unwrap();
        let program = parse(tokens).unwrap();

        assert_eq!(program.statements.len(), 2);
        match &program.statements[0] {
            Statement::FunctionDef { body, .. } => {
                assert_eq!(body.len(), 2);
            }
            _ => panic!("Expected function definition"),
        }
        assert!(matches!(program.statements[1], Statement::Print { .. }));
    }

    #[test]
    fn test_parse_multiple_statements_after_function() {
        // Function definition followed by different statement types
        let source = "def foo():\n    return 1\nx = 42\nprint(x)";
        let tokens = lex(source).unwrap();
        let program = parse(tokens).unwrap();

        assert_eq!(program.statements.len(), 3);
        assert!(matches!(
            program.statements[0],
            Statement::FunctionDef { .. }
        ));
        assert!(matches!(
            program.statements[1],
            Statement::Assignment { .. }
        ));
        assert!(matches!(program.statements[2], Statement::Print { .. }));
    }

    #[test]
    fn test_parse_empty_then_nonempty_function() {
        // Empty function followed by non-empty function
        let source = "def empty():\n\n\ndef nonempty():\n    return 1";
        let tokens = lex(source).unwrap();
        let program = parse(tokens).unwrap();

        assert_eq!(program.statements.len(), 2);
        match &program.statements[0] {
            Statement::FunctionDef { name, body, .. } => {
                assert_eq!(name, "empty");
                assert_eq!(body.len(), 0);
            }
            _ => panic!("Expected function definition"),
        }
        match &program.statements[1] {
            Statement::FunctionDef { name, body, .. } => {
                assert_eq!(name, "nonempty");
                assert_eq!(body.len(), 1);
            }
            _ => panic!("Expected function definition"),
        }
    }

    #[test]
    fn test_parse_varying_statement_indents() {
        // All statements in body have same indentation level (4 spaces)
        let source = "def foo():\n    x = 1\n    y = 2\n    return x";
        let tokens = lex(source).unwrap();
        let program = parse(tokens).unwrap();

        match &program.statements[0] {
            Statement::FunctionDef { name, body, .. } => {
                assert_eq!(name, "foo");
                assert_eq!(body.len(), 3);
            }
            _ => panic!("Expected function definition"),
        }
    }

    #[test]
    fn test_parse_multiple_funcs_at_root() {
        // Multiple functions at the same indentation level
        let source = "def foo():\n    return 1\ndef bar():\n    return 2";
        let tokens = lex(source).unwrap();
        let program = parse(tokens).unwrap();

        assert_eq!(program.statements.len(), 2);
        match &program.statements[0] {
            Statement::FunctionDef { name, body, .. } => {
                assert_eq!(name, "foo");
                assert_eq!(body.len(), 1);
            }
            _ => panic!("Expected function definition"),
        }
        match &program.statements[1] {
            Statement::FunctionDef { name, body, .. } => {
                assert_eq!(name, "bar");
                assert_eq!(body.len(), 1);
            }
            _ => panic!("Expected function definition"),
        }
    }
}
