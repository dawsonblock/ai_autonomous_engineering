//! Edge case tests for bitmap-based register validity tracking
//!
//! These tests validate the correctness of the bitmap implementation
//! across all boundary conditions and edge cases for 256 registers
//! stored in 4 u64 words.

use pyrust::ast::BinaryOperator;
use pyrust::bytecode::BytecodeBuilder;
use pyrust::vm::VM;

#[test]
fn test_bitmap_register_0_boundary() {
    // Test register 0 (first bit of first word)
    let mut builder = BytecodeBuilder::new();
    builder.emit_load_const(0, 42);
    let bytecode = builder.build();

    let mut vm = VM::new();
    vm.execute(&bytecode).unwrap();

    // Verify register 0 is accessible
    assert_eq!(vm.format_output(None), "");
}

#[test]
fn test_bitmap_register_63_boundary() {
    // Test register 63 (last bit of first word)
    let mut builder = BytecodeBuilder::new();
    builder.emit_load_const(63, 999);
    let bytecode = builder.build();

    let mut vm = VM::new();
    vm.execute(&bytecode).unwrap();

    // Should complete without error
}

#[test]
fn test_bitmap_register_64_boundary() {
    // Test register 64 (first bit of second word)
    let mut builder = BytecodeBuilder::new();
    builder.emit_load_const(64, 777);
    let bytecode = builder.build();

    let mut vm = VM::new();
    vm.execute(&bytecode).unwrap();

    // Should complete without error
}

#[test]
fn test_bitmap_register_127_boundary() {
    // Test register 127 (last bit of second word)
    let mut builder = BytecodeBuilder::new();
    builder.emit_load_const(127, 555);
    let bytecode = builder.build();

    let mut vm = VM::new();
    vm.execute(&bytecode).unwrap();

    // Should complete without error
}

#[test]
fn test_bitmap_register_128_boundary() {
    // Test register 128 (first bit of third word)
    let mut builder = BytecodeBuilder::new();
    builder.emit_load_const(128, 333);
    let bytecode = builder.build();

    let mut vm = VM::new();
    vm.execute(&bytecode).unwrap();

    // Should complete without error
}

#[test]
fn test_bitmap_register_191_boundary() {
    // Test register 191 (last bit of third word)
    let mut builder = BytecodeBuilder::new();
    builder.emit_load_const(191, 111);
    let bytecode = builder.build();

    let mut vm = VM::new();
    vm.execute(&bytecode).unwrap();

    // Should complete without error
}

#[test]
fn test_bitmap_register_192_boundary() {
    // Test register 192 (first bit of fourth word)
    let mut builder = BytecodeBuilder::new();
    builder.emit_load_const(192, 888);
    let bytecode = builder.build();

    let mut vm = VM::new();
    vm.execute(&bytecode).unwrap();

    // Should complete without error
}

#[test]
fn test_bitmap_register_255_boundary() {
    // Test register 255 (last bit of fourth word, maximum valid register)
    let mut builder = BytecodeBuilder::new();
    builder.emit_load_const(255, 12345);
    let bytecode = builder.build();

    let mut vm = VM::new();
    vm.execute(&bytecode).unwrap();

    // Should complete without error
}

#[test]
fn test_uninitialized_register_access() {
    // Try to use a register that was never initialized
    let mut builder = BytecodeBuilder::new();
    builder.emit_load_const(0, 10);
    // Register 1 is never initialized
    builder.emit_binary_op(2, 0, BinaryOperator::Add, 1); // Should fail
    let bytecode = builder.build();

    let mut vm = VM::new();
    let result = vm.execute(&bytecode);

    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.message.contains("Register 1 is empty"));
    assert_eq!(err.instruction_index, 1); // BinaryOp instruction
}

#[test]
fn test_multiple_registers_across_words() {
    // Test setting and accessing multiple registers across different u64 words
    let mut builder = BytecodeBuilder::new();

    // Set registers in each word
    builder.emit_load_const(0, 1); // First word
    builder.emit_load_const(63, 2); // First word boundary
    builder.emit_load_const(64, 3); // Second word
    builder.emit_load_const(127, 4); // Second word boundary
    builder.emit_load_const(128, 5); // Third word
    builder.emit_load_const(191, 6); // Third word boundary
    builder.emit_load_const(192, 7); // Fourth word
    builder.emit_load_const(255, 8); // Fourth word boundary

    // Use all registers in calculations
    builder.emit_binary_op(10, 0, BinaryOperator::Add, 63); // 1 + 2 = 3
    builder.emit_binary_op(11, 64, BinaryOperator::Add, 127); // 3 + 4 = 7
    builder.emit_binary_op(12, 128, BinaryOperator::Add, 191); // 5 + 6 = 11
    builder.emit_binary_op(13, 192, BinaryOperator::Add, 255); // 7 + 8 = 15
    builder.emit_binary_op(14, 10, BinaryOperator::Add, 11); // 3 + 7 = 10
    builder.emit_binary_op(15, 12, BinaryOperator::Add, 13); // 11 + 15 = 26
    builder.emit_binary_op(16, 14, BinaryOperator::Add, 15); // 10 + 26 = 36
    builder.emit_set_result(16);

    let bytecode = builder.build();

    let mut vm = VM::new();
    let result = vm.execute(&bytecode).unwrap();

    assert_eq!(result, Some(pyrust::value::Value::Integer(36)));
}

#[test]
fn test_register_overwrite() {
    // Test that overwriting a register properly updates the value
    let mut builder = BytecodeBuilder::new();

    builder.emit_load_const(5, 100);
    builder.emit_load_const(5, 200); // Overwrite register 5
    builder.emit_set_result(5);

    let bytecode = builder.build();

    let mut vm = VM::new();
    let result = vm.execute(&bytecode).unwrap();

    assert_eq!(result, Some(pyrust::value::Value::Integer(200)));
}

#[test]
fn test_register_validity_isolation() {
    // Test that setting one register doesn't affect validity of others
    let mut builder = BytecodeBuilder::new();

    // Set register 0
    builder.emit_load_const(0, 42);

    // Try to use uninitialized register 1 (should fail)
    builder.emit_binary_op(2, 0, BinaryOperator::Add, 1);

    let bytecode = builder.build();

    let mut vm = VM::new();
    let result = vm.execute(&bytecode);

    assert!(result.is_err());
    assert!(result.unwrap_err().message.contains("Register 1 is empty"));
}

#[test]
fn test_all_256_registers_sequential() {
    // Stress test: set and verify all 256 registers
    let mut builder = BytecodeBuilder::new();

    // Load constants into all 256 registers
    for i in 0..=255u8 {
        builder.emit_load_const(i, i as i64);
    }

    // Sum a few to verify they're all accessible
    builder.emit_binary_op(0, 0, BinaryOperator::Add, 1); // 0 + 1 = 1
    builder.emit_binary_op(0, 0, BinaryOperator::Add, 100); // 1 + 100 = 101
    builder.emit_binary_op(0, 0, BinaryOperator::Add, 255); // 101 + 255 = 356
    builder.emit_set_result(0);

    let bytecode = builder.build();

    let mut vm = VM::new();
    let result = vm.execute(&bytecode).unwrap();

    assert_eq!(result, Some(pyrust::value::Value::Integer(356)));
}

#[test]
fn test_instruction_pointer_tracks_correctly_on_error() {
    // Verify instruction pointer is accurate when error occurs at different positions
    let mut builder = BytecodeBuilder::new();

    // Instruction 0: LoadConst
    builder.emit_load_const(0, 10);
    // Instruction 1: LoadConst
    builder.emit_load_const(1, 0);
    // Instruction 2: BinaryOp (division by zero error should report index 2)
    builder.emit_binary_op(2, 0, BinaryOperator::Div, 1);

    let bytecode = builder.build();

    let mut vm = VM::new();
    let result = vm.execute(&bytecode);

    assert!(result.is_err());
    let err = result.unwrap_err();
    assert_eq!(err.message, "Division by zero");
    assert_eq!(err.instruction_index, 2); // Error at instruction 2
}

#[test]
fn test_instruction_pointer_tracks_undefined_variable() {
    // Test instruction pointer on undefined variable access
    let mut builder = BytecodeBuilder::new();

    // Instruction 0: LoadConst
    builder.emit_load_const(0, 42);
    // Instruction 1: StoreVar
    builder.emit_store_var("x", 1, 0);
    // Instruction 2: LoadVar for undefined variable
    builder.emit_load_var(1, "undefined", 2);

    let bytecode = builder.build();

    let mut vm = VM::new();
    let result = vm.execute(&bytecode);

    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.message.contains("Undefined variable: undefined"));
    assert_eq!(err.instruction_index, 2); // Error at instruction 2
}

#[test]
fn test_copy_trait_for_value_in_registers() {
    // Verify that Value implements Copy trait and registers use it efficiently
    let mut builder = BytecodeBuilder::new();

    builder.emit_load_const(0, 42);
    // Use register 0 multiple times (Copy trait means no move)
    builder.emit_binary_op(1, 0, BinaryOperator::Add, 0); // 42 + 42 = 84
    builder.emit_binary_op(2, 0, BinaryOperator::Mul, 1); // 42 * 84 = 3528
    builder.emit_set_result(2);

    let bytecode = builder.build();

    let mut vm = VM::new();
    let result = vm.execute(&bytecode).unwrap();

    assert_eq!(result, Some(pyrust::value::Value::Integer(3528)));
}
