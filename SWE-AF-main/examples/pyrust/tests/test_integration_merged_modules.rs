//! Integration tests for merged modules (lexer, value, bytecode)
//!
//! Priority 1: Test conflict resolution areas (lib.rs module declarations)
//! Priority 2: Test cross-feature interactions between merged modules
//! Priority 3: Test shared type usage (BinaryOperator, UnaryOperator from AST)

use pyrust::ast::{BinaryOperator, UnaryOperator};
use pyrust::bytecode::{BytecodeBuilder, Instruction};
use pyrust::lexer::{lex, TokenKind};
use pyrust::value::Value;

/// PRIORITY 1: Conflict Resolution Tests
/// Tests that all three modules (lexer, value, bytecode) are properly accessible
/// after being merged into lib.rs

#[test]
fn test_all_modules_accessible() {
    // Test lexer module is accessible and functional
    let tokens = lex("42").unwrap();
    assert_eq!(tokens[0].kind, TokenKind::Integer);

    // Test value module is accessible and functional
    let val = Value::Integer(42);
    assert_eq!(val.as_integer(), 42);

    // Test bytecode module is accessible and functional
    let mut builder = BytecodeBuilder::new();
    builder.emit_load_const(0, 42);
    let bytecode = builder.build();
    assert_eq!(bytecode.instructions.len(), 2); // LoadConst + Halt
}

#[test]
fn test_lib_exports_all_merged_modules() {
    // Verify that lib.rs correctly exports all three merged modules
    // by attempting to use types from each module

    // From lexer module
    let _: TokenKind = TokenKind::Integer;

    // From value module
    let _: Value = Value::Integer(0);

    // From bytecode module
    let _: BytecodeBuilder = BytecodeBuilder::new();
}

/// PRIORITY 2: Cross-Feature Interaction Tests
/// Tests interactions between lexer, value, and bytecode modules

#[test]
fn test_lexer_to_value_integer_flow() {
    // Lexer tokenizes integer -> Value stores integer
    let source = "42";
    let tokens = lex(source).unwrap();

    assert_eq!(tokens.len(), 2); // Integer + Eof
    assert_eq!(tokens[0].kind, TokenKind::Integer);
    assert_eq!(tokens[0].text, "42");

    // Simulate conversion to Value (what parser/compiler would do)
    let parsed_int: i64 = tokens[0].text.parse().unwrap();
    let value = Value::Integer(parsed_int);
    assert_eq!(value.as_integer(), 42);
}

#[test]
fn test_lexer_operators_match_ast_operators() {
    // Verify that lexer token types correspond to AST operators
    // This tests the boundary between lexer and the shared AST types

    let source = "+ - * / // %";
    let tokens = lex(source).unwrap();

    // Verify lexer produces expected token types
    assert_eq!(tokens[0].kind, TokenKind::Plus);
    assert_eq!(tokens[1].kind, TokenKind::Minus);
    assert_eq!(tokens[2].kind, TokenKind::Star);
    assert_eq!(tokens[3].kind, TokenKind::Slash);
    assert_eq!(tokens[4].kind, TokenKind::DoubleSlash);
    assert_eq!(tokens[5].kind, TokenKind::Percent);

    // Verify these map to AST operators
    let _add_op = BinaryOperator::Add;
    let _sub_op = BinaryOperator::Sub;
    let _mul_op = BinaryOperator::Mul;
    let _div_op = BinaryOperator::Div;
    let _floor_div_op = BinaryOperator::FloorDiv;
    let _mod_op = BinaryOperator::Mod;
}

#[test]
fn test_bytecode_uses_ast_operators() {
    // Verify bytecode module correctly uses BinaryOperator from AST
    let mut builder = BytecodeBuilder::new();

    // Test all binary operators work in bytecode
    builder.emit_binary_op(0, 1, BinaryOperator::Add, 2);
    builder.emit_binary_op(0, 1, BinaryOperator::Sub, 2);
    builder.emit_binary_op(0, 1, BinaryOperator::Mul, 2);
    builder.emit_binary_op(0, 1, BinaryOperator::Div, 2);
    builder.emit_binary_op(0, 1, BinaryOperator::FloorDiv, 2);
    builder.emit_binary_op(0, 1, BinaryOperator::Mod, 2);

    let bytecode = builder.build();

    // Should have 6 binary ops + 1 Halt
    assert_eq!(bytecode.instructions.len(), 7);

    // Verify instructions are BinaryOp type
    for i in 0..6 {
        assert!(matches!(
            bytecode.instructions[i],
            Instruction::BinaryOp { .. }
        ));
    }
}

#[test]
fn test_value_executes_ast_operators() {
    // Verify value module correctly executes operations using AST operators
    let left = Value::Integer(10);
    let right = Value::Integer(5);

    // Test all binary operators from AST work with Value
    let add_result = left.binary_op(BinaryOperator::Add, &right).unwrap();
    assert_eq!(add_result.as_integer(), 15);

    let sub_result = left.binary_op(BinaryOperator::Sub, &right).unwrap();
    assert_eq!(sub_result.as_integer(), 5);

    let mul_result = left.binary_op(BinaryOperator::Mul, &right).unwrap();
    assert_eq!(mul_result.as_integer(), 50);

    let div_result = left.binary_op(BinaryOperator::Div, &right).unwrap();
    assert_eq!(div_result.as_integer(), 2);

    let floor_div_result = left.binary_op(BinaryOperator::FloorDiv, &right).unwrap();
    assert_eq!(floor_div_result.as_integer(), 2);

    let mod_result = left.binary_op(BinaryOperator::Mod, &right).unwrap();
    assert_eq!(mod_result.as_integer(), 0);
}

#[test]
fn test_lexer_to_bytecode_operator_mapping() {
    // End-to-end test: Lexer tokens -> AST operators -> Bytecode instructions

    // Lexer phase: tokenize expression "10 + 5"
    let tokens = lex("10 + 5").unwrap();
    assert_eq!(tokens[0].kind, TokenKind::Integer);
    assert_eq!(tokens[1].kind, TokenKind::Plus);
    assert_eq!(tokens[2].kind, TokenKind::Integer);

    // Simulate compilation: tokens -> bytecode
    let mut builder = BytecodeBuilder::new();

    // Load operands
    let left_val: i64 = tokens[0].text.parse().unwrap();
    let right_val: i64 = tokens[2].text.parse().unwrap();

    builder.emit_load_const(0, left_val);
    builder.emit_load_const(1, right_val);

    // Map token type to operator and emit
    let operator = match tokens[1].kind {
        TokenKind::Plus => BinaryOperator::Add,
        _ => panic!("Unexpected token"),
    };
    builder.emit_binary_op(2, 0, operator, 1);

    let bytecode = builder.build();

    // Verify bytecode was correctly generated
    assert_eq!(bytecode.constants[0], 10);
    assert_eq!(bytecode.constants[1], 5);
    assert!(matches!(
        bytecode.instructions[2],
        Instruction::BinaryOp {
            dest_reg: 2,
            left_reg: 0,
            op: BinaryOperator::Add,
            right_reg: 1
        }
    ));
}

#[test]
fn test_complete_pipeline_arithmetic_expression() {
    // Complete pipeline: Lexer -> (simulated parser) -> Bytecode -> Value execution

    // Step 1: Lexer tokenizes "2 + 3 * 4"
    let tokens = lex("2 + 3 * 4").unwrap();
    assert_eq!(tokens.len(), 6); // 2, +, 3, *, 4, EOF

    // Step 2: Build bytecode (simulating compiler)
    // Note: This simulates evaluation with correct precedence (* before +)
    let mut builder = BytecodeBuilder::new();

    builder.emit_load_const(0, 2); // Load 2
    builder.emit_load_const(1, 3); // Load 3
    builder.emit_load_const(2, 4); // Load 4

    // Execute 3 * 4 first (higher precedence)
    builder.emit_binary_op(3, 1, BinaryOperator::Mul, 2);
    // Then execute 2 + result
    builder.emit_binary_op(4, 0, BinaryOperator::Add, 3);

    let bytecode = builder.build();

    // Step 3: Verify constants are deduplicated correctly
    assert_eq!(bytecode.constants.len(), 3);

    // Step 4: Simulate VM execution using Value
    let val_2 = Value::Integer(2);
    let val_3 = Value::Integer(3);
    let val_4 = Value::Integer(4);

    // Execute: 3 * 4 = 12
    let mul_result = val_3.binary_op(BinaryOperator::Mul, &val_4).unwrap();
    assert_eq!(mul_result.as_integer(), 12);

    // Execute: 2 + 12 = 14
    let add_result = val_2.binary_op(BinaryOperator::Add, &mul_result).unwrap();
    assert_eq!(add_result.as_integer(), 14);
}

#[test]
fn test_floor_division_end_to_end() {
    // Test floor division operator through all layers
    // This is critical because // is a two-character token in lexer

    // Lexer: tokenize "10 // 3"
    let tokens = lex("10 // 3").unwrap();
    assert_eq!(tokens.len(), 4); // 10, //, 3, EOF
    assert_eq!(tokens[1].kind, TokenKind::DoubleSlash);
    assert_eq!(tokens[1].text, "//");

    // Bytecode: emit floor division instruction
    let mut builder = BytecodeBuilder::new();
    builder.emit_load_const(0, 10);
    builder.emit_load_const(1, 3);
    builder.emit_binary_op(2, 0, BinaryOperator::FloorDiv, 1);
    let bytecode = builder.build();

    // Verify instruction
    if let Instruction::BinaryOp { op, .. } = bytecode.instructions[2] {
        assert_eq!(op, BinaryOperator::FloorDiv);
    } else {
        panic!("Expected BinaryOp instruction");
    }

    // Value: execute floor division
    let left = Value::Integer(10);
    let right = Value::Integer(3);
    let result = left.binary_op(BinaryOperator::FloorDiv, &right).unwrap();
    assert_eq!(result.as_integer(), 3);
}

#[test]
fn test_unary_operators_cross_module() {
    // Test unary operators across all modules

    // Lexer: tokenize "-42"
    let tokens = lex("-42").unwrap();
    assert_eq!(tokens[0].kind, TokenKind::Minus);
    assert_eq!(tokens[1].kind, TokenKind::Integer);

    // Bytecode: emit unary operation
    let mut builder = BytecodeBuilder::new();
    builder.emit_load_const(0, 42);
    builder.emit_unary_op(1, UnaryOperator::Neg, 0);
    let bytecode = builder.build();

    assert!(matches!(
        bytecode.instructions[1],
        Instruction::UnaryOp {
            op: UnaryOperator::Neg,
            ..
        }
    ));

    // Value: execute unary operation
    let val = Value::Integer(42);
    let result = val.unary_op(UnaryOperator::Neg).unwrap();
    assert_eq!(result.as_integer(), -42);
}

#[test]
fn test_variable_assignment_flow() {
    // Test variable flow: Lexer -> Bytecode (variable names)

    // Lexer: tokenize "x = 42"
    let tokens = lex("x = 42").unwrap();
    assert_eq!(tokens[0].kind, TokenKind::Identifier);
    assert_eq!(tokens[0].text, "x");
    assert_eq!(tokens[1].kind, TokenKind::Equals);
    assert_eq!(tokens[2].kind, TokenKind::Integer);

    // Bytecode: emit store variable
    let mut builder = BytecodeBuilder::new();
    builder.emit_load_const(0, 42);
    builder.emit_store_var(tokens[0].text, 1, 0);
    builder.emit_load_var(1, "x", 1);

    let bytecode = builder.build();

    // Verify variable name pool
    assert_eq!(bytecode.var_names.len(), 1);
    assert_eq!(bytecode.var_names[0], "x");

    // Verify instructions reference correct index
    assert!(matches!(
        bytecode.instructions[1],
        Instruction::StoreVar {
            var_name_index: 0,
            var_id: 1,
            src_reg: 0
        }
    ));
    assert!(matches!(
        bytecode.instructions[2],
        Instruction::LoadVar {
            dest_reg: 1,
            var_name_index: 0,
            var_id: 1
        }
    ));
}

#[test]
fn test_print_statement_flow() {
    // Test print statement through lexer and bytecode

    // Lexer: tokenize "print(42)"
    let tokens = lex("print(42)").unwrap();
    assert_eq!(tokens[0].kind, TokenKind::Print);
    assert_eq!(tokens[1].kind, TokenKind::LeftParen);
    assert_eq!(tokens[2].kind, TokenKind::Integer);
    assert_eq!(tokens[3].kind, TokenKind::RightParen);

    // Bytecode: emit print instruction
    let mut builder = BytecodeBuilder::new();
    builder.emit_load_const(0, 42);
    builder.emit_print(0);

    let bytecode = builder.build();

    assert!(matches!(
        bytecode.instructions[1],
        Instruction::Print { src_reg: 0 }
    ));
}

/// PRIORITY 3: Shared Type Usage Tests
/// Tests that AST types (BinaryOperator, UnaryOperator) work correctly
/// across value and bytecode modules

#[test]
fn test_binary_operator_consistency() {
    // Verify BinaryOperator is consistently used in both bytecode and value modules

    let operators = vec![
        BinaryOperator::Add,
        BinaryOperator::Sub,
        BinaryOperator::Mul,
        BinaryOperator::Div,
        BinaryOperator::FloorDiv,
        BinaryOperator::Mod,
    ];

    for op in operators {
        // Test in bytecode module
        let mut builder = BytecodeBuilder::new();
        builder.emit_load_const(0, 10);
        builder.emit_load_const(1, 5);
        builder.emit_binary_op(2, 0, op, 1);
        let bytecode = builder.build();

        // Verify instruction was created
        if let Instruction::BinaryOp { op: inst_op, .. } = bytecode.instructions[2] {
            assert_eq!(inst_op, op);
        } else {
            panic!("Expected BinaryOp instruction");
        }

        // Test in value module (skip division by zero for Div, FloorDiv, Mod)
        let left = Value::Integer(10);
        let right = Value::Integer(5);
        let result = left.binary_op(op, &right);
        assert!(result.is_ok(), "Operation {:?} failed", op);
    }
}

#[test]
fn test_unary_operator_consistency() {
    // Verify UnaryOperator is consistently used in both bytecode and value modules

    let operators = vec![UnaryOperator::Neg, UnaryOperator::Pos];

    for op in operators {
        // Test in bytecode module
        let mut builder = BytecodeBuilder::new();
        builder.emit_load_const(0, 42);
        builder.emit_unary_op(1, op, 0);
        let bytecode = builder.build();

        // Verify instruction was created
        if let Instruction::UnaryOp { op: inst_op, .. } = bytecode.instructions[1] {
            assert_eq!(inst_op, op);
        } else {
            panic!("Expected UnaryOp instruction");
        }

        // Test in value module
        let val = Value::Integer(42);
        let result = val.unary_op(op);
        assert!(result.is_ok(), "Operation {:?} failed", op);
    }
}

#[test]
fn test_operator_precedence_from_ast() {
    // Verify BinaryOperator precedence is accessible and correct
    // This is used by parser but defined in AST

    assert_eq!(BinaryOperator::Add.precedence(), 1);
    assert_eq!(BinaryOperator::Sub.precedence(), 1);
    assert_eq!(BinaryOperator::Mul.precedence(), 2);
    assert_eq!(BinaryOperator::Div.precedence(), 2);
    assert_eq!(BinaryOperator::FloorDiv.precedence(), 2);
    assert_eq!(BinaryOperator::Mod.precedence(), 2);

    // Verify precedence ordering
    assert!(BinaryOperator::Mul.precedence() > BinaryOperator::Add.precedence());
}

#[test]
fn test_error_propagation_across_modules() {
    // Test that errors from value module can be handled
    // This tests the integration with error module

    // Division by zero error from value module
    let left = Value::Integer(10);
    let right = Value::Integer(0);
    let result = left.binary_op(BinaryOperator::Div, &right);

    assert!(result.is_err());
    let err = result.unwrap_err();
    assert_eq!(err.message, "Division by zero");

    // Integer overflow error
    let max_val = Value::Integer(i64::MAX);
    let one = Value::Integer(1);
    let result = max_val.binary_op(BinaryOperator::Add, &one);

    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.message.contains("Integer overflow"));
}

#[test]
fn test_complex_multi_module_interaction() {
    // Complex test involving all three merged modules
    // Simulates: "result = 10 + 20 * 3" with variable storage

    // Lexer phase
    let tokens = lex("result = 10 + 20 * 3").unwrap();
    assert_eq!(tokens[0].text, "result");
    assert_eq!(tokens[1].kind, TokenKind::Equals);

    // Bytecode compilation phase (with correct precedence)
    let mut builder = BytecodeBuilder::new();

    // Load constants
    builder.emit_load_const(0, 10);
    builder.emit_load_const(1, 20);
    builder.emit_load_const(2, 3);

    // Execute 20 * 3 first (precedence)
    builder.emit_binary_op(3, 1, BinaryOperator::Mul, 2);

    // Then 10 + result
    builder.emit_binary_op(4, 0, BinaryOperator::Add, 3);

    // Store in variable
    builder.emit_store_var("result", 1, 4);

    let bytecode = builder.build();

    // Verify constant pool deduplication
    assert_eq!(bytecode.constants.len(), 3);

    // Verify variable pool
    assert_eq!(bytecode.var_names.len(), 1);
    assert_eq!(bytecode.var_names[0], "result");

    // Value execution phase (simulating VM)
    let v10 = Value::Integer(10);
    let v20 = Value::Integer(20);
    let v3 = Value::Integer(3);

    // 20 * 3 = 60
    let mul_result = v20.binary_op(BinaryOperator::Mul, &v3).unwrap();
    assert_eq!(mul_result.as_integer(), 60);

    // 10 + 60 = 70
    let final_result = v10.binary_op(BinaryOperator::Add, &mul_result).unwrap();
    assert_eq!(final_result.as_integer(), 70);
}

#[test]
fn test_constant_pool_with_parsed_integers() {
    // Test that integers from lexer are correctly added to bytecode constant pool

    let source = "42 100 42 200 100"; // Duplicates to test deduplication
    let tokens = lex(source).unwrap();

    let mut builder = BytecodeBuilder::new();
    let mut reg = 0;

    for token in tokens {
        if token.kind == TokenKind::Integer {
            let value: i64 = token.text.parse().unwrap();
            builder.emit_load_const(reg, value);
            reg += 1;
        }
    }

    let bytecode = builder.build();

    // Should have only 3 unique constants: 42, 100, 200
    assert_eq!(bytecode.constants.len(), 3);
    assert!(bytecode.constants.contains(&42));
    assert!(bytecode.constants.contains(&100));
    assert!(bytecode.constants.contains(&200));
}

#[test]
fn test_all_arithmetic_operators_integration() {
    // Comprehensive test of all arithmetic operators through all modules

    let test_cases = vec![
        ("10 + 5", TokenKind::Plus, BinaryOperator::Add, 10, 5, 15),
        ("10 - 5", TokenKind::Minus, BinaryOperator::Sub, 10, 5, 5),
        ("10 * 5", TokenKind::Star, BinaryOperator::Mul, 10, 5, 50),
        ("10 / 5", TokenKind::Slash, BinaryOperator::Div, 10, 5, 2),
        (
            "10 // 3",
            TokenKind::DoubleSlash,
            BinaryOperator::FloorDiv,
            10,
            3,
            3,
        ),
        ("10 % 3", TokenKind::Percent, BinaryOperator::Mod, 10, 3, 1),
    ];

    for (source, expected_token, expected_op, left_val, right_val, expected_result) in test_cases {
        // Lexer phase
        let tokens = lex(source).unwrap();
        assert_eq!(
            tokens[1].kind, expected_token,
            "Failed for source: {}",
            source
        );

        // Bytecode phase
        let mut builder = BytecodeBuilder::new();
        builder.emit_load_const(0, left_val);
        builder.emit_load_const(1, right_val);
        builder.emit_binary_op(2, 0, expected_op, 1);
        let bytecode = builder.build();

        if let Instruction::BinaryOp { op, .. } = bytecode.instructions[2] {
            assert_eq!(op, expected_op, "Failed for source: {}", source);
        }

        // Value phase
        let left = Value::Integer(left_val);
        let right = Value::Integer(right_val);
        let result = left.binary_op(expected_op, &right).unwrap();
        assert_eq!(
            result.as_integer(),
            expected_result,
            "Failed for source: {}",
            source
        );
    }
}
