//! Integration tests for merged error and ast modules
//!
//! These tests verify that the error and ast modules work correctly together
//! after being merged into the integration branch.

use pyrust::ast::{BinaryOperator, Expression, Program, Statement, UnaryOperator};
use pyrust::error::{CompileError, LexError, ParseError, PyRustError, RuntimeError};

/// Test that error module and ast module can be imported together
/// This tests the conflict resolution in src/lib.rs where both modules are exported
#[test]
fn test_module_imports() {
    // Should be able to create AST nodes
    let expr = Expression::Integer(42);
    assert_eq!(expr, Expression::Integer(42));

    // Should be able to create error types
    let lex_err = LexError {
        message: "test error".to_string(),
        line: 1,
        column: 1,
    };
    assert_eq!(lex_err.message, "test error");
}

/// Test that AST types can be used in error contexts
/// This simulates how a parser would report errors about AST construction
#[test]
fn test_parse_error_with_ast_context() {
    // Simulate a parse error that would occur during AST construction
    let parse_err = ParseError {
        message: "Expected expression after binary operator".to_string(),
        line: 1,
        column: 5,
        found_token: "+".to_string(),
        expected_tokens: vec!["integer".to_string(), "identifier".to_string()],
    };

    let pyrust_err: PyRustError = parse_err.into();
    let display = format!("{}", pyrust_err);

    // Verify error displays correctly
    assert!(display.contains("ParseError at 1:5"));
    assert!(display.contains("Expected expression after binary operator"));
    assert!(display.contains("Found: +"));
}

/// Test error handling for invalid AST operations
/// This tests how errors would be reported during AST traversal
#[test]
fn test_runtime_error_with_ast_expression() {
    // Create a division expression that would fail at runtime
    let _division_expr = Expression::BinaryOp {
        left: Box::new(Expression::Integer(10)),
        op: BinaryOperator::Div,
        right: Box::new(Expression::Integer(0)),
    };

    // Create a runtime error for division by zero
    let runtime_err = RuntimeError {
        message: "Division by zero in binary operation".to_string(),
        instruction_index: 5,
    };

    let pyrust_err: PyRustError = runtime_err.into();
    let display = format!("{}", pyrust_err);

    assert!(display.contains("RuntimeError at instruction 5"));
    assert!(display.contains("Division by zero"));
}

/// Test that complex AST structures work with error reporting
#[test]
fn test_complex_ast_with_error_handling() {
    // Build a complex AST: (x + y) * z / 0
    let complex_expr = Expression::BinaryOp {
        left: Box::new(Expression::BinaryOp {
            left: Box::new(Expression::BinaryOp {
                left: Box::new(Expression::Variable("x".to_string())),
                op: BinaryOperator::Add,
                right: Box::new(Expression::Variable("y".to_string())),
            }),
            op: BinaryOperator::Mul,
            right: Box::new(Expression::Variable("z".to_string())),
        }),
        op: BinaryOperator::Div,
        right: Box::new(Expression::Integer(0)),
    };

    // Verify AST is constructed correctly
    if let Expression::BinaryOp { op, .. } = &complex_expr {
        assert_eq!(*op, BinaryOperator::Div);
    } else {
        panic!("Expected BinaryOp");
    }

    // Create an error that might occur during evaluation
    let err = RuntimeError {
        message: "Division by zero in complex expression".to_string(),
        instruction_index: 10,
    };

    assert_eq!(err.message, "Division by zero in complex expression");
}

/// Test AST with all binary operators and error types
#[test]
fn test_all_operators_with_error_types() {
    let operators = vec![
        BinaryOperator::Add,
        BinaryOperator::Sub,
        BinaryOperator::Mul,
        BinaryOperator::Div,
        BinaryOperator::FloorDiv,
        BinaryOperator::Mod,
    ];

    for op in operators {
        let expr = Expression::BinaryOp {
            left: Box::new(Expression::Integer(10)),
            op,
            right: Box::new(Expression::Integer(2)),
        };

        // Verify each operator can be used in AST
        if let Expression::BinaryOp { op: actual_op, .. } = &expr {
            assert_eq!(*actual_op, op);
        } else {
            panic!("Expected BinaryOp");
        }
    }

    // Test that compile errors can reference operators
    let compile_err = CompileError {
        message: "Failed to compile FloorDiv operator".to_string(),
    };

    let err: PyRustError = compile_err.into();
    let display = format!("{}", err);
    assert!(display.contains("Failed to compile FloorDiv operator"));
}

/// Test statement types with error handling
#[test]
fn test_statements_with_errors() {
    // Test assignment statement
    let assign = Statement::Assignment {
        name: "x".to_string(),
        value: Expression::Integer(42),
    };

    if let Statement::Assignment { name, .. } = &assign {
        assert_eq!(name, "x");
    } else {
        panic!("Expected Assignment");
    }

    // Create an error for undefined variable
    let runtime_err = RuntimeError {
        message: "Undefined variable: x".to_string(),
        instruction_index: 0,
    };

    let err: PyRustError = runtime_err.into();
    assert!(format!("{}", err).contains("Undefined variable: x"));

    // Test print statement
    let print_stmt = Statement::Print {
        value: Expression::Variable("undefined_var".to_string()),
    };

    if let Statement::Print { value } = &print_stmt {
        if let Expression::Variable(name) = value {
            assert_eq!(name, "undefined_var");
        }
    }
}

/// Test program construction with error scenarios
#[test]
fn test_program_with_error_scenarios() {
    let program = Program {
        statements: vec![
            Statement::Assignment {
                name: "x".to_string(),
                value: Expression::Integer(10),
            },
            Statement::Assignment {
                name: "y".to_string(),
                value: Expression::Integer(0),
            },
            Statement::Expression {
                value: Expression::BinaryOp {
                    left: Box::new(Expression::Variable("x".to_string())),
                    op: BinaryOperator::Div,
                    right: Box::new(Expression::Variable("y".to_string())),
                },
            },
        ],
    };

    assert_eq!(program.statements.len(), 3);

    // Simulate runtime error during execution
    let err = RuntimeError {
        message: "Division by zero at statement 2".to_string(),
        instruction_index: 15,
    };

    let pyrust_err: PyRustError = err.into();
    assert!(format!("{}", pyrust_err).contains("Division by zero"));
}

/// Test unary operators with error handling
#[test]
fn test_unary_operators_with_errors() {
    let unary_operators = vec![UnaryOperator::Neg, UnaryOperator::Pos];

    for op in unary_operators {
        let expr = Expression::UnaryOp {
            op,
            operand: Box::new(Expression::Integer(42)),
        };

        if let Expression::UnaryOp { op: actual_op, .. } = &expr {
            assert_eq!(*actual_op, op);
        } else {
            panic!("Expected UnaryOp");
        }
    }
}

/// Test error trait implementations work correctly
#[test]
fn test_error_trait_implementations() {
    let lex_err = LexError {
        message: "Invalid token".to_string(),
        line: 1,
        column: 1,
    };

    // Test From trait
    let pyrust_err: PyRustError = lex_err.clone().into();

    // Test Display trait
    let display = format!("{}", pyrust_err);
    assert!(display.contains("LexError"));
    assert!(display.contains("Invalid token"));

    // Test std::error::Error trait
    let _: &dyn std::error::Error = &pyrust_err;
}

/// Test precedence rules with error scenarios
#[test]
fn test_precedence_with_errors() {
    // Test that precedence is correctly implemented
    assert_eq!(BinaryOperator::Add.precedence(), 1);
    assert_eq!(BinaryOperator::Mul.precedence(), 2);
    assert!(BinaryOperator::Mul.precedence() > BinaryOperator::Add.precedence());

    // Create an expression that tests precedence: 2 + 3 * 4
    let expr = Expression::BinaryOp {
        left: Box::new(Expression::Integer(2)),
        op: BinaryOperator::Add,
        right: Box::new(Expression::BinaryOp {
            left: Box::new(Expression::Integer(3)),
            op: BinaryOperator::Mul,
            right: Box::new(Expression::Integer(4)),
        }),
    };

    // Verify structure (multiplication should be nested on right due to precedence)
    if let Expression::BinaryOp { left, op, right } = &expr {
        assert_eq!(**left, Expression::Integer(2));
        assert_eq!(*op, BinaryOperator::Add);
        if let Expression::BinaryOp { op: mul_op, .. } = &**right {
            assert_eq!(*mul_op, BinaryOperator::Mul);
        } else {
            panic!("Expected nested BinaryOp");
        }
    }
}

/// Test Clone and PartialEq traits work across modules
#[test]
fn test_cross_module_traits() {
    // Test AST cloning
    let expr = Expression::Integer(42);
    let cloned_expr = expr.clone();
    assert_eq!(expr, cloned_expr);

    // Test error cloning
    let err = LexError {
        message: "test".to_string(),
        line: 1,
        column: 1,
    };
    let cloned_err = err.clone();
    assert_eq!(err, cloned_err);

    // Test that both modules' types implement required traits
    let _: bool = expr == cloned_expr; // PartialEq
    let _: bool = err == cloned_err; // PartialEq
}

/// Test error location information is preserved
#[test]
fn test_error_location_information() {
    // Test LexError location
    let lex_err = LexError {
        message: "Unexpected character '@'".to_string(),
        line: 5,
        column: 10,
    };
    assert_eq!(lex_err.line, 5);
    assert_eq!(lex_err.column, 10);

    // Test ParseError location
    let parse_err = ParseError {
        message: "Expected expression".to_string(),
        line: 3,
        column: 15,
        found_token: "EOF".to_string(),
        expected_tokens: vec!["integer".to_string()],
    };
    assert_eq!(parse_err.line, 3);
    assert_eq!(parse_err.column, 15);

    // Test RuntimeError location
    let runtime_err = RuntimeError {
        message: "Stack overflow".to_string(),
        instruction_index: 42,
    };
    assert_eq!(runtime_err.instruction_index, 42);
}

/// Test that error messages are descriptive and include context
#[test]
fn test_error_message_quality() {
    // Test ParseError with multiple expected tokens
    let parse_err = ParseError {
        message: "Unexpected token".to_string(),
        line: 1,
        column: 5,
        found_token: "=".to_string(),
        expected_tokens: vec![
            "integer".to_string(),
            "identifier".to_string(),
            "(".to_string(),
        ],
    };

    let display = format!("{}", PyRustError::from(parse_err));
    assert!(display.contains("ParseError at 1:5"));
    assert!(display.contains("Found: ="));
    assert!(display.contains("Expected: integer | identifier | ("));
}

/// Test error conversions don't lose information
#[test]
fn test_error_conversion_preserves_data() {
    let original = ParseError {
        message: "Test message".to_string(),
        line: 10,
        column: 20,
        found_token: "test_token".to_string(),
        expected_tokens: vec!["expected1".to_string(), "expected2".to_string()],
    };

    let converted: PyRustError = original.clone().into();

    if let PyRustError::ParseError(err) = converted {
        assert_eq!(err.message, original.message);
        assert_eq!(err.line, original.line);
        assert_eq!(err.column, original.column);
        assert_eq!(err.found_token, original.found_token);
        assert_eq!(err.expected_tokens, original.expected_tokens);
    } else {
        panic!("Expected ParseError variant");
    }
}
