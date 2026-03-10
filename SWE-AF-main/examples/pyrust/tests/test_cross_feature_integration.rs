//! Cross-Feature Integration Tests
//!
//! Tests the Parser → Compiler → VM pipeline integration
//! Focuses on:
//! 1. Module interaction boundaries (conflict resolution areas)
//! 2. Data flow between components
//! 3. Error propagation
//! 4. Feature compatibility

use pyrust::compiler::compile;
use pyrust::lexer::lex;
use pyrust::parser::parse;
use pyrust::value::Value;
use pyrust::vm::VM;

// ============================================================================
// PRIORITY 1: Conflict Resolution - Module Accessibility
// ============================================================================

#[test]
fn test_all_three_modules_accessible_from_lib() {
    // Tests that parser, compiler, and vm modules are all accessible
    // This verifies the conflict resolution in src/lib.rs

    let source = "42";
    let tokens = lex(source).expect("Lexer accessible");
    let program = parse(tokens).expect("Parser accessible");
    let bytecode = compile(&program).expect("Compiler accessible");
    let mut vm = VM::new();
    let result = vm.execute(&bytecode).expect("VM accessible");

    assert_eq!(result, Some(Value::Integer(42)));
}

#[test]
fn test_parser_compiler_boundary() {
    // Parser's AST output must match Compiler's expected input format
    let source = "x = 10\ny = 20\nx + y";
    let tokens = lex(source).unwrap();
    let program = parse(tokens).unwrap();

    let bytecode = compile(&program);
    assert!(bytecode.is_ok(), "Compiler must accept Parser AST format");
}

#[test]
fn test_compiler_vm_boundary() {
    // Compiler's bytecode must match VM's expected format
    let source = "2 + 3 * 4";
    let tokens = lex(source).unwrap();
    let program = parse(tokens).unwrap();
    let bytecode = compile(&program).unwrap();

    let mut vm = VM::new();
    let result = vm.execute(&bytecode);
    assert!(result.is_ok(), "VM must execute Compiler bytecode format");
    assert_eq!(result.unwrap(), Some(Value::Integer(14)));
}

// ============================================================================
// PRIORITY 2: Full Pipeline - Arithmetic Operations
// ============================================================================

#[test]
fn test_pipeline_basic_arithmetic() {
    let cases = vec![
        ("2 + 3", 5),
        ("10 - 5", 5),
        ("3 * 4", 12),
        ("20 / 4", 5),
        ("7 // 2", 3),
        ("10 % 3", 1),
    ];

    for (source, expected) in cases {
        let tokens = lex(source).unwrap();
        let program = parse(tokens).unwrap();
        let bytecode = compile(&program).unwrap();
        let mut vm = VM::new();
        let result = vm.execute(&bytecode).unwrap();

        assert_eq!(result, Some(Value::Integer(expected)), "Failed: {}", source);
    }
}

#[test]
fn test_pipeline_operator_precedence() {
    // Critical: Parser precedence → Compiler → VM execution
    let cases = vec![
        ("2 + 3 * 4", 14),
        ("10 - 5 / 5", 9),
        ("(2 + 3) * 4", 20),
        ("2 * 3 + 4 * 5", 26),
    ];

    for (source, expected) in cases {
        let tokens = lex(source).unwrap();
        let program = parse(tokens).unwrap();
        let bytecode = compile(&program).unwrap();
        let mut vm = VM::new();
        let result = vm.execute(&bytecode).unwrap();

        assert_eq!(result, Some(Value::Integer(expected)), "Failed: {}", source);
    }
}

#[test]
fn test_pipeline_left_associativity() {
    // Tests that Parser's associativity is preserved
    let cases = vec![
        ("10 - 5 - 2", 3), // (10 - 5) - 2
        ("20 / 4 / 2", 2), // (20 / 4) / 2
        ("2 * 3 * 4", 24),
    ];

    for (source, expected) in cases {
        let tokens = lex(source).unwrap();
        let program = parse(tokens).unwrap();
        let bytecode = compile(&program).unwrap();
        let mut vm = VM::new();
        let result = vm.execute(&bytecode).unwrap();

        assert_eq!(result, Some(Value::Integer(expected)), "Failed: {}", source);
    }
}

// ============================================================================
// PRIORITY 2: Full Pipeline - Variables
// ============================================================================

#[test]
fn test_pipeline_variables() {
    let source = "x = 10\ny = 20\nx + y";
    let tokens = lex(source).unwrap();
    let program = parse(tokens).unwrap();
    let bytecode = compile(&program).unwrap();
    let mut vm = VM::new();
    let result = vm.execute(&bytecode).unwrap();

    assert_eq!(result, Some(Value::Integer(30)));
}

#[test]
fn test_pipeline_variable_reassignment() {
    let source = "x = 10\nx = x + 5\nx = x * 2\nx";
    let tokens = lex(source).unwrap();
    let program = parse(tokens).unwrap();
    let bytecode = compile(&program).unwrap();
    let mut vm = VM::new();
    let result = vm.execute(&bytecode).unwrap();

    assert_eq!(result, Some(Value::Integer(30)));
}

#[test]
fn test_pipeline_many_variables() {
    let source = "a = 1\nb = 2\nc = 3\nd = 4\ne = 5\na + b + c + d + e";
    let tokens = lex(source).unwrap();
    let program = parse(tokens).unwrap();
    let bytecode = compile(&program).unwrap();
    let mut vm = VM::new();
    let result = vm.execute(&bytecode).unwrap();

    assert_eq!(result, Some(Value::Integer(15)));
}

// ============================================================================
// PRIORITY 2: Full Pipeline - Print Statement
// ============================================================================

#[test]
fn test_pipeline_print() {
    let source = "print(42)";
    let tokens = lex(source).unwrap();
    let program = parse(tokens).unwrap();
    let bytecode = compile(&program).unwrap();
    let mut vm = VM::new();
    let result = vm.execute(&bytecode).unwrap();

    assert_eq!(result, None, "Print should not set result");
    assert_eq!(vm.format_output(result), "42\n");
}

#[test]
fn test_pipeline_print_with_expression() {
    let source = "x = 10\ny = 20\nprint(x + y)\nx * y";
    let tokens = lex(source).unwrap();
    let program = parse(tokens).unwrap();
    let bytecode = compile(&program).unwrap();
    let mut vm = VM::new();
    let result = vm.execute(&bytecode).unwrap();

    assert_eq!(result, Some(Value::Integer(200)));
    assert!(vm.format_output(result).contains("30"));
}

// ============================================================================
// PRIORITY 2: SetResult Emission Rules (Critical Compiler-VM Integration)
// ============================================================================

#[test]
fn test_setresult_assignment_no_result() {
    // Assignment: NO SetResult
    let source = "x = 42";
    let tokens = lex(source).unwrap();
    let program = parse(tokens).unwrap();
    let bytecode = compile(&program).unwrap();
    let mut vm = VM::new();
    let result = vm.execute(&bytecode).unwrap();

    assert_eq!(result, None);
}

#[test]
fn test_setresult_print_no_result() {
    // Print: NO SetResult
    let source = "print(42)";
    let tokens = lex(source).unwrap();
    let program = parse(tokens).unwrap();
    let bytecode = compile(&program).unwrap();
    let mut vm = VM::new();
    let result = vm.execute(&bytecode).unwrap();

    assert_eq!(result, None);
}

#[test]
fn test_setresult_expression_has_result() {
    // Expression: YES SetResult
    let source = "42";
    let tokens = lex(source).unwrap();
    let program = parse(tokens).unwrap();
    let bytecode = compile(&program).unwrap();
    let mut vm = VM::new();
    let result = vm.execute(&bytecode).unwrap();

    assert_eq!(result, Some(Value::Integer(42)));
}

#[test]
fn test_setresult_last_expression_wins() {
    let source = "5\n10\n15";
    let tokens = lex(source).unwrap();
    let program = parse(tokens).unwrap();
    let bytecode = compile(&program).unwrap();
    let mut vm = VM::new();
    let result = vm.execute(&bytecode).unwrap();

    assert_eq!(result, Some(Value::Integer(15)));
}

#[test]
fn test_setresult_mixed_statements() {
    let source = "x = 10\nprint(x)\n20\ny = 30\n40";
    let tokens = lex(source).unwrap();
    let program = parse(tokens).unwrap();
    let bytecode = compile(&program).unwrap();
    let mut vm = VM::new();
    let result = vm.execute(&bytecode).unwrap();

    assert_eq!(result, Some(Value::Integer(40)));
}

// ============================================================================
// PRIORITY 3: Error Propagation
// ============================================================================

#[test]
fn test_parse_error_propagation() {
    // Changed from "2 + + 3" which is now valid (parses as 2 + (+3))
    // to a truly invalid expression
    let source = "2 + * 3";
    let tokens = lex(source).unwrap();
    let result = parse(tokens);

    assert!(result.is_err());
}

#[test]
fn test_runtime_error_division_by_zero() {
    let source = "10 / 0";
    let tokens = lex(source).unwrap();
    let program = parse(tokens).unwrap();
    let bytecode = compile(&program).unwrap();
    let mut vm = VM::new();
    let result = vm.execute(&bytecode);

    assert!(result.is_err());
    assert!(result.unwrap_err().message.contains("Division by zero"));
}

#[test]
fn test_runtime_error_undefined_variable() {
    let source = "x + 1";
    let tokens = lex(source).unwrap();
    let program = parse(tokens).unwrap();
    let bytecode = compile(&program).unwrap();
    let mut vm = VM::new();
    let result = vm.execute(&bytecode);

    assert!(result.is_err());
}

// ============================================================================
// Edge Cases
// ============================================================================

#[test]
fn test_empty_program() {
    let source = "";
    let tokens = lex(source).unwrap();
    let program = parse(tokens).unwrap();
    let bytecode = compile(&program).unwrap();
    let mut vm = VM::new();
    let result = vm.execute(&bytecode).unwrap();

    assert_eq!(result, None);
}

#[test]
fn test_deeply_nested_parentheses() {
    let source = "((((1 + 2) * 3) - 4) / 5)";
    let tokens = lex(source).unwrap();
    let program = parse(tokens).unwrap();
    let bytecode = compile(&program).unwrap();
    let mut vm = VM::new();
    let result = vm.execute(&bytecode).unwrap();

    assert_eq!(result, Some(Value::Integer(1)));
}

#[test]
fn test_many_operations() {
    let source = "1 + 2 + 3 + 4 + 5 + 6 + 7 + 8 + 9 + 10";
    let tokens = lex(source).unwrap();
    let program = parse(tokens).unwrap();
    let bytecode = compile(&program).unwrap();
    let mut vm = VM::new();
    let result = vm.execute(&bytecode).unwrap();

    assert_eq!(result, Some(Value::Integer(55)));
}

#[test]
fn test_large_integers() {
    let source = "1000000 + 2000000";
    let tokens = lex(source).unwrap();
    let program = parse(tokens).unwrap();
    let bytecode = compile(&program).unwrap();
    let mut vm = VM::new();
    let result = vm.execute(&bytecode).unwrap();

    assert_eq!(result, Some(Value::Integer(3000000)));
}

#[test]
fn test_floor_division_python_semantics() {
    // Using subtraction workaround since parser doesn't support unary minus
    let cases = vec![("7 // 2", 3), ("(0 - 7) // 2", -4), ("7 // (0 - 2)", -4)];

    for (source, expected) in cases {
        let tokens = lex(source).unwrap();
        let program = parse(tokens).unwrap();
        let bytecode = compile(&program).unwrap();
        let mut vm = VM::new();
        let result = vm.execute(&bytecode).unwrap();

        assert_eq!(result, Some(Value::Integer(expected)), "Failed: {}", source);
    }
}

#[test]
fn test_modulo_python_semantics() {
    let cases = vec![
        ("10 % 3", 1),
        ("10 % 5", 0),
        ("(0 - 10) % 3", 2), // Python: -10 % 3 = 2
    ];

    for (source, expected) in cases {
        let tokens = lex(source).unwrap();
        let program = parse(tokens).unwrap();
        let bytecode = compile(&program).unwrap();
        let mut vm = VM::new();
        let result = vm.execute(&bytecode).unwrap();

        assert_eq!(result, Some(Value::Integer(expected)), "Failed: {}", source);
    }
}

#[test]
fn test_complex_multi_statement_program() {
    let source = "a = 5\nb = 10\nc = a + b\nprint(c)\nc * 2";
    let tokens = lex(source).unwrap();
    let program = parse(tokens).unwrap();
    let bytecode = compile(&program).unwrap();
    let mut vm = VM::new();
    let result = vm.execute(&bytecode).unwrap();

    assert_eq!(result, Some(Value::Integer(30)));
    assert!(vm.format_output(result).contains("15"));
}

#[test]
fn test_all_operators_in_one_program() {
    let source = "add = 10 + 5\nsub = 10 - 5\nmul = 10 * 5\ndiv = 10 / 5\nfdiv = 10 // 3\nmod_op = 10 % 3\nadd + sub + mul + div + fdiv + mod_op";
    let tokens = lex(source).unwrap();
    let program = parse(tokens).unwrap();
    let bytecode = compile(&program).unwrap();
    let mut vm = VM::new();
    let result = vm.execute(&bytecode).unwrap();

    // 15 + 5 + 50 + 2 + 3 + 1 = 76
    assert_eq!(result, Some(Value::Integer(76)));
}

// ============================================================================
// INTEGRATION BUG DOCUMENTATION
// ============================================================================

#[test]
fn test_bug_unary_minus_not_supported_by_parser() {
    // FIXED: Parser now supports unary minus operator

    let source = "-42";
    let tokens = lex(source).unwrap();
    let program = parse(tokens).unwrap();
    assert_eq!(program.statements.len(), 1);
}

#[test]
fn test_bug_unary_plus_not_supported_by_parser() {
    // FIXED: Parser now supports unary plus operator

    let source = "+42";
    let tokens = lex(source).unwrap();
    let program = parse(tokens).unwrap();
    assert_eq!(program.statements.len(), 1);
}
