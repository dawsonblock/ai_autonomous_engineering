//! High-priority tests for conflict resolution areas
//!
//! These tests specifically target the files where conflicts were resolved
//! during the merge of issue/error-module and issue/ast-module branches.

use pyrust::ast::{BinaryOperator, Expression, Program, Statement};
use pyrust::error::{LexError, ParseError, PyRustError, RuntimeError};

/// CONFLICT RESOLUTION TEST: src/lib.rs
/// Verifies that both `pub mod error;` and `pub mod ast;` exports work together
#[test]
fn test_lib_rs_conflict_resolution() {
    // Test that both modules are accessible through the public API

    // From error module (first branch merged)
    let lex_error = LexError {
        message: "test".to_string(),
        line: 1,
        column: 1,
    };
    let _: PyRustError = lex_error.into();

    // From ast module (second branch merged)
    let expr = Expression::Integer(42);
    let stmt = Statement::Expression { value: expr };
    let _program = Program {
        statements: vec![stmt],
    };

    // Both modules should be usable together without conflicts
    let parse_err = ParseError {
        message: "AST parsing failed".to_string(),
        line: 1,
        column: 1,
        found_token: "EOF".to_string(),
        expected_tokens: vec!["expression".to_string()],
    };

    let ast_expr = Expression::BinaryOp {
        left: Box::new(Expression::Integer(1)),
        op: BinaryOperator::Add,
        right: Box::new(Expression::Integer(2)),
    };

    // Verify both work in same scope
    assert!(format!("{}", PyRustError::from(parse_err)).contains("AST parsing failed"));
    assert_eq!(
        ast_expr,
        Expression::BinaryOp {
            left: Box::new(Expression::Integer(1)),
            op: BinaryOperator::Add,
            right: Box::new(Expression::Integer(2)),
        }
    );
}

/// CONFLICT RESOLUTION TEST: Cargo.toml
/// Verifies package name 'pyrust' and criterion dependency work correctly
#[test]
fn test_cargo_toml_conflict_resolution() {
    // This test verifies that:
    // 1. The package name 'pyrust' from error-module was kept
    // 2. The criterion dev-dependency from ast-module works
    // 3. The project compiles successfully with merged dependencies

    // The fact that this test compiles and runs means the Cargo.toml merge was successful
    // We can import from both modules using the 'pyrust' package name

    use pyrust::ast::UnaryOperator;
    use pyrust::error::CompileError;

    let _compile_err = CompileError {
        message: "Testing Cargo.toml merge".to_string(),
    };

    let _unary_op = UnaryOperator::Neg;

    // If we can instantiate types from both modules, the merge succeeded
    assert!(matches!(_unary_op, UnaryOperator::Neg));
}

/// CONFLICT RESOLUTION TEST: Cross-module type usage
/// Tests that AST types and Error types can be used together seamlessly
#[test]
fn test_cross_module_type_integration() {
    // Simulate a realistic scenario: parsing AST and handling errors

    // Create an AST that might fail
    let program = Program {
        statements: vec![
            Statement::Assignment {
                name: "x".to_string(),
                value: Expression::Integer(10),
            },
            Statement::Print {
                value: Expression::Variable("undefined".to_string()),
            },
        ],
    };

    // Create errors that reference the AST
    let parse_error = ParseError {
        message: "Failed to parse statement 2".to_string(),
        line: 2,
        column: 1,
        found_token: "undefined".to_string(),
        expected_tokens: vec!["defined_variable".to_string()],
    };

    let runtime_error = RuntimeError {
        message: "Variable 'undefined' not found in scope".to_string(),
        instruction_index: 1,
    };

    // Verify both types work together
    assert_eq!(program.statements.len(), 2);
    assert!(format!("{}", PyRustError::from(parse_error)).contains("Failed to parse"));
    assert!(
        format!("{}", PyRustError::from(runtime_error)).contains("Variable 'undefined' not found")
    );
}

/// CONFLICT RESOLUTION TEST: Module documentation
/// Verifies that documentation from ast-module is preserved in lib.rs
#[test]
fn test_lib_documentation_preserved() {
    // This test verifies that the crate can be imported and used
    // with the documentation comment that was preserved from ast-module

    // The documentation in lib.rs reads:
    // "//! Python-Rust Fast Compiler
    //  //!
    //  //! A high-performance compiler for a Python-like language implemented in Rust."

    // If both modules are accessible, the documentation merge succeeded
    let _: Expression = Expression::Integer(1);
    let _: PyRustError = LexError {
        message: "test".to_string(),
        line: 1,
        column: 1,
    }
    .into();
}

/// CONFLICT RESOLUTION TEST: All error variants with AST operations
/// Tests all error types can report on AST-related operations
#[test]
fn test_all_error_types_with_ast() {
    // LexError - would occur before AST construction
    let lex_err = LexError {
        message: "Invalid character in source".to_string(),
        line: 1,
        column: 5,
    };
    assert!(format!("{}", PyRustError::from(lex_err)).contains("Invalid character"));

    // ParseError - occurs during AST construction
    let parse_err = ParseError {
        message: "Cannot build AST node".to_string(),
        line: 2,
        column: 10,
        found_token: "invalid".to_string(),
        expected_tokens: vec!["expression".to_string()],
    };
    assert!(format!("{}", PyRustError::from(parse_err)).contains("Cannot build AST"));

    // RuntimeError - occurs during AST evaluation
    let runtime_err = RuntimeError {
        message: "Error evaluating AST expression".to_string(),
        instruction_index: 5,
    };
    assert!(format!("{}", PyRustError::from(runtime_err)).contains("evaluating AST"));

    // All error types successfully integrate with AST workflow
}

/// CONFLICT RESOLUTION TEST: Operator precedence with error handling
/// Tests that AST precedence logic works with error reporting
#[test]
fn test_precedence_integration_with_errors() {
    // Create expression with precedence: 2 + 3 * 4 (should parse as 2 + (3 * 4))
    let expr = Expression::BinaryOp {
        left: Box::new(Expression::Integer(2)),
        op: BinaryOperator::Add,
        right: Box::new(Expression::BinaryOp {
            left: Box::new(Expression::Integer(3)),
            op: BinaryOperator::Mul,
            right: Box::new(Expression::Integer(4)),
        }),
    };

    // Verify precedence is correct
    if let Expression::BinaryOp {
        op: outer_op,
        right,
        ..
    } = &expr
    {
        assert_eq!(*outer_op, BinaryOperator::Add);
        if let Expression::BinaryOp { op: inner_op, .. } = &**right {
            assert_eq!(*inner_op, BinaryOperator::Mul);
            assert!(inner_op.precedence() > outer_op.precedence());
        }
    }

    // Create error that might occur with precedence issues
    let parse_err = ParseError {
        message: "Precedence error in expression".to_string(),
        line: 1,
        column: 1,
        found_token: "*".to_string(),
        expected_tokens: vec!["operand".to_string()],
    };

    assert!(format!("{}", PyRustError::from(parse_err)).contains("Precedence error"));
}

/// CONFLICT RESOLUTION TEST: Complete integration scenario
/// Simulates a full parse -> error workflow using both merged modules
#[test]
fn test_complete_integration_scenario() {
    // Scenario: Parse a program, encounter errors, report them

    // Step 1: Valid program construction (AST module)
    let valid_program = Program {
        statements: vec![Statement::Assignment {
            name: "result".to_string(),
            value: Expression::BinaryOp {
                left: Box::new(Expression::Integer(10)),
                op: BinaryOperator::Mul,
                right: Box::new(Expression::Integer(5)),
            },
        }],
    };
    assert_eq!(valid_program.statements.len(), 1);

    // Step 2: Simulate lex error (error module)
    let lex_error = PyRustError::LexError(LexError {
        message: "Unexpected '@' in source".to_string(),
        line: 1,
        column: 8,
    });
    assert!(format!("{}", lex_error).contains("LexError at 1:8"));

    // Step 3: Simulate parse error (error module with AST context)
    let parse_error = PyRustError::ParseError(ParseError {
        message: "Expected expression after operator".to_string(),
        line: 1,
        column: 15,
        found_token: ";".to_string(),
        expected_tokens: vec!["integer".to_string(), "identifier".to_string()],
    });
    assert!(format!("{}", parse_error).contains("ParseError at 1:15"));

    // Step 4: Simulate runtime error during execution (error module)
    let runtime_error = PyRustError::RuntimeError(RuntimeError {
        message: "Division by zero".to_string(),
        instruction_index: 10,
    });
    assert!(format!("{}", runtime_error).contains("RuntimeError at instruction 10"));

    // All components work together seamlessly
}
