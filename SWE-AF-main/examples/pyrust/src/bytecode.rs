//! Bytecode format and builder for register-based VM
//!
//! Defines compact bytecode instruction format with 8 instruction types.
//! Target: 8-16 bytes per instruction.

use crate::ast::{BinaryOperator, UnaryOperator};

/// Compact bytecode instruction for register-based VM
#[derive(Debug, Clone, PartialEq)]
pub enum Instruction {
    /// Load constant from constant pool into register
    /// Args: dest_reg, const_index
    LoadConst { dest_reg: u8, const_index: usize },

    /// Load variable value into register
    /// Args: dest_reg, var_name_index, var_id
    LoadVar {
        dest_reg: u8,
        var_name_index: usize,
        var_id: u32,
    },

    /// Store register value into variable
    /// Args: var_name_index, var_id, src_reg
    StoreVar {
        var_name_index: usize,
        var_id: u32,
        src_reg: u8,
    },

    /// Binary operation: dest_reg = left_reg op right_reg
    /// Args: dest_reg, left_reg, op, right_reg
    BinaryOp {
        dest_reg: u8,
        left_reg: u8,
        op: BinaryOperator,
        right_reg: u8,
    },

    /// Unary operation: dest_reg = op operand_reg
    /// Args: dest_reg, op, operand_reg
    UnaryOp {
        dest_reg: u8,
        op: UnaryOperator,
        operand_reg: u8,
    },

    /// Print value from register
    /// Args: src_reg
    Print { src_reg: u8 },

    /// Set result register for expression statements
    /// Args: src_reg
    SetResult { src_reg: u8 },

    /// Halt execution
    Halt,

    /// Define a function
    /// Args: name_index, param_count, body_start, body_len, max_register_used
    DefineFunction {
        name_index: usize,
        param_count: u8,
        body_start: usize,
        body_len: usize,
        max_register_used: u8,
    },

    /// Call a function
    /// Args: name_index, arg_count, first_arg_reg, dest_reg
    Call {
        name_index: usize,
        arg_count: u8,
        first_arg_reg: u8,
        dest_reg: u8,
    },

    /// Return from a function
    /// Args: has_value, src_reg (None if has_value is false)
    Return {
        has_value: bool,
        src_reg: Option<u8>,
    },
}

/// Compiler metadata tracking register usage
#[derive(Debug, Clone, PartialEq)]
pub struct CompilerMetadata {
    /// Maximum register used during compilation
    pub max_register_used: u8,
}

/// Complete bytecode program with constant and variable pools
#[derive(Debug, Clone, PartialEq)]
pub struct Bytecode {
    /// Instruction sequence
    pub instructions: Vec<Instruction>,

    /// Constant pool for integer literals
    pub constants: Vec<i64>,

    /// Variable name pool for identifiers
    pub var_names: Vec<String>,

    /// Variable ID pool parallel to var_names for interned IDs
    pub var_ids: Vec<u32>,

    /// Compiler metadata
    pub metadata: CompilerMetadata,
}

/// Builder for constructing bytecode with automatic pooling
pub struct BytecodeBuilder {
    instructions: Vec<Instruction>,
    constants: Vec<i64>,
    var_names: Vec<String>,
    var_ids: Vec<u32>,
}

impl BytecodeBuilder {
    /// Create a new bytecode builder
    pub fn new() -> Self {
        Self {
            instructions: Vec::new(),
            constants: Vec::new(),
            var_names: Vec::new(),
            var_ids: Vec::new(),
        }
    }

    /// Add or reuse a constant in the pool, returning its index
    fn add_constant(&mut self, value: i64) -> usize {
        // Check if constant already exists
        if let Some(index) = self.constants.iter().position(|&c| c == value) {
            return index;
        }
        // Add new constant
        let index = self.constants.len();
        self.constants.push(value);
        index
    }

    /// Add or reuse a variable name in the pool, returning its index
    fn add_var_name(&mut self, name: &str, var_id: u32) -> usize {
        // Check if variable name already exists
        if let Some(index) = self.var_names.iter().position(|n| n == name) {
            return index;
        }
        // Add new variable name and ID
        let index = self.var_names.len();
        self.var_names.push(name.to_string());
        self.var_ids.push(var_id);
        index
    }

    /// Ensure a variable name is in the pool (for parameter registration)
    /// Returns the index of the var_name in the pool
    pub fn ensure_var_name(&mut self, name: &str, var_id: u32) -> usize {
        self.add_var_name(name, var_id)
    }

    /// Emit LoadConst instruction
    pub fn emit_load_const(&mut self, dest_reg: u8, value: i64) {
        let const_index = self.add_constant(value);
        self.instructions.push(Instruction::LoadConst {
            dest_reg,
            const_index,
        });
    }

    /// Emit LoadVar instruction
    pub fn emit_load_var(&mut self, dest_reg: u8, var_name: &str, var_id: u32) {
        let var_name_index = self.add_var_name(var_name, var_id);
        self.instructions.push(Instruction::LoadVar {
            dest_reg,
            var_name_index,
            var_id,
        });
    }

    /// Emit StoreVar instruction
    pub fn emit_store_var(&mut self, var_name: &str, var_id: u32, src_reg: u8) {
        let var_name_index = self.add_var_name(var_name, var_id);
        self.instructions.push(Instruction::StoreVar {
            var_name_index,
            var_id,
            src_reg,
        });
    }

    /// Emit BinaryOp instruction
    pub fn emit_binary_op(
        &mut self,
        dest_reg: u8,
        left_reg: u8,
        op: BinaryOperator,
        right_reg: u8,
    ) {
        self.instructions.push(Instruction::BinaryOp {
            dest_reg,
            left_reg,
            op,
            right_reg,
        });
    }

    /// Emit UnaryOp instruction
    pub fn emit_unary_op(&mut self, dest_reg: u8, op: UnaryOperator, operand_reg: u8) {
        self.instructions.push(Instruction::UnaryOp {
            dest_reg,
            op,
            operand_reg,
        });
    }

    /// Emit Print instruction
    pub fn emit_print(&mut self, src_reg: u8) {
        self.instructions.push(Instruction::Print { src_reg });
    }

    /// Emit SetResult instruction
    pub fn emit_set_result(&mut self, src_reg: u8) {
        self.instructions.push(Instruction::SetResult { src_reg });
    }

    /// Emit DefineFunction instruction
    pub fn emit_define_function(
        &mut self,
        name: &str,
        var_id: u32,
        param_count: u8,
        body_start: usize,
        body_len: usize,
        max_register_used: u8,
    ) {
        let name_index = self.add_var_name(name, var_id);
        self.instructions.push(Instruction::DefineFunction {
            name_index,
            param_count,
            body_start,
            body_len,
            max_register_used,
        });
    }

    /// Emit Call instruction
    pub fn emit_call(
        &mut self,
        name: &str,
        var_id: u32,
        arg_count: u8,
        first_arg_reg: u8,
        dest_reg: u8,
    ) {
        let name_index = self.add_var_name(name, var_id);
        self.instructions.push(Instruction::Call {
            name_index,
            arg_count,
            first_arg_reg,
            dest_reg,
        });
    }

    /// Emit Return instruction
    pub fn emit_return(&mut self, has_value: bool, src_reg: Option<u8>) {
        self.instructions
            .push(Instruction::Return { has_value, src_reg });
    }

    /// Build final bytecode, automatically appending Halt instruction
    pub fn build(mut self) -> Bytecode {
        // Automatically append Halt instruction
        self.instructions.push(Instruction::Halt);

        Bytecode {
            instructions: self.instructions,
            constants: self.constants,
            var_names: self.var_names,
            var_ids: self.var_ids,
            metadata: CompilerMetadata {
                max_register_used: 0, // Will be set by compiler
            },
        }
    }

    /// Get a reference to the current instructions (for compiler use)
    pub fn instructions(&self) -> &[Instruction] {
        &self.instructions
    }

    /// Append instructions from another builder (for compiler use)
    pub fn append_instructions(&mut self, instructions: Vec<Instruction>) {
        self.instructions.extend(instructions);
    }

    /// Get references to the constant and variable name pools (for compiler use)
    pub fn get_pools(&self) -> (&Vec<i64>, &Vec<String>, &Vec<u32>) {
        (&self.constants, &self.var_names, &self.var_ids)
    }

    /// Create a new builder with pre-populated pools (for compiler use)
    pub fn with_pools(constants: Vec<i64>, var_names: Vec<String>, var_ids: Vec<u32>) -> Self {
        Self {
            instructions: Vec::new(),
            constants,
            var_names,
            var_ids,
        }
    }
}

impl Default for BytecodeBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_instruction_creation() {
        // Test LoadConst instruction
        let load = Instruction::LoadConst {
            dest_reg: 0,
            const_index: 5,
        };
        assert_eq!(
            load,
            Instruction::LoadConst {
                dest_reg: 0,
                const_index: 5
            }
        );

        // Test LoadVar instruction
        let load_var = Instruction::LoadVar {
            dest_reg: 1,
            var_name_index: 2,
            var_id: 10,
        };
        assert_eq!(
            load_var,
            Instruction::LoadVar {
                dest_reg: 1,
                var_name_index: 2,
                var_id: 10
            }
        );

        // Test StoreVar instruction
        let store = Instruction::StoreVar {
            var_name_index: 0,
            var_id: 5,
            src_reg: 3,
        };
        assert_eq!(
            store,
            Instruction::StoreVar {
                var_name_index: 0,
                var_id: 5,
                src_reg: 3
            }
        );

        // Test BinaryOp instruction
        let binop = Instruction::BinaryOp {
            dest_reg: 0,
            left_reg: 1,
            op: BinaryOperator::Add,
            right_reg: 2,
        };
        assert_eq!(
            binop,
            Instruction::BinaryOp {
                dest_reg: 0,
                left_reg: 1,
                op: BinaryOperator::Add,
                right_reg: 2,
            }
        );

        // Test UnaryOp instruction
        let unop = Instruction::UnaryOp {
            dest_reg: 0,
            op: UnaryOperator::Neg,
            operand_reg: 1,
        };
        assert_eq!(
            unop,
            Instruction::UnaryOp {
                dest_reg: 0,
                op: UnaryOperator::Neg,
                operand_reg: 1,
            }
        );

        // Test Print instruction
        let print = Instruction::Print { src_reg: 5 };
        assert_eq!(print, Instruction::Print { src_reg: 5 });

        // Test SetResult instruction
        let set_result = Instruction::SetResult { src_reg: 7 };
        assert_eq!(set_result, Instruction::SetResult { src_reg: 7 });

        // Test Halt instruction
        let halt = Instruction::Halt;
        assert_eq!(halt, Instruction::Halt);

        // Test DefineFunction instruction
        let def_func = Instruction::DefineFunction {
            name_index: 0,
            param_count: 2,
            body_start: 10,
            body_len: 5,
            max_register_used: 3,
        };
        assert_eq!(
            def_func,
            Instruction::DefineFunction {
                name_index: 0,
                param_count: 2,
                body_start: 10,
                body_len: 5,
                max_register_used: 3,
            }
        );

        // Test Call instruction
        let call = Instruction::Call {
            name_index: 1,
            arg_count: 3,
            first_arg_reg: 0,
            dest_reg: 4,
        };
        assert_eq!(
            call,
            Instruction::Call {
                name_index: 1,
                arg_count: 3,
                first_arg_reg: 0,
                dest_reg: 4,
            }
        );

        // Test Return instruction with value
        let return_val = Instruction::Return {
            has_value: true,
            src_reg: Some(5),
        };
        assert_eq!(
            return_val,
            Instruction::Return {
                has_value: true,
                src_reg: Some(5),
            }
        );

        // Test Return instruction without value
        let return_none = Instruction::Return {
            has_value: false,
            src_reg: None,
        };
        assert_eq!(
            return_none,
            Instruction::Return {
                has_value: false,
                src_reg: None,
            }
        );
    }

    #[test]
    fn test_bytecode_builder_basic() {
        let mut builder = BytecodeBuilder::new();
        builder.emit_load_const(0, 42);
        builder.emit_print(0);

        let bytecode = builder.build();

        // Check instructions (including auto-appended Halt)
        assert_eq!(bytecode.instructions.len(), 3);
        assert_eq!(
            bytecode.instructions[0],
            Instruction::LoadConst {
                dest_reg: 0,
                const_index: 0
            }
        );
        assert_eq!(bytecode.instructions[1], Instruction::Print { src_reg: 0 });
        assert_eq!(bytecode.instructions[2], Instruction::Halt);

        // Check constant pool
        assert_eq!(bytecode.constants.len(), 1);
        assert_eq!(bytecode.constants[0], 42);
    }

    #[test]
    fn test_constant_pool_deduplication() {
        let mut builder = BytecodeBuilder::new();

        // Add same constant multiple times
        builder.emit_load_const(0, 100);
        builder.emit_load_const(1, 200);
        builder.emit_load_const(2, 100); // Duplicate
        builder.emit_load_const(3, 200); // Duplicate
        builder.emit_load_const(4, 100); // Duplicate again

        let bytecode = builder.build();

        // Only 2 unique constants should be in the pool
        assert_eq!(bytecode.constants.len(), 2);
        assert_eq!(bytecode.constants[0], 100);
        assert_eq!(bytecode.constants[1], 200);

        // Check that instructions reference correct indices
        assert_eq!(
            bytecode.instructions[0],
            Instruction::LoadConst {
                dest_reg: 0,
                const_index: 0
            }
        );
        assert_eq!(
            bytecode.instructions[1],
            Instruction::LoadConst {
                dest_reg: 1,
                const_index: 1
            }
        );
        assert_eq!(
            bytecode.instructions[2],
            Instruction::LoadConst {
                dest_reg: 2,
                const_index: 0
            }
        ); // Reuses index 0
        assert_eq!(
            bytecode.instructions[3],
            Instruction::LoadConst {
                dest_reg: 3,
                const_index: 1
            }
        ); // Reuses index 1
        assert_eq!(
            bytecode.instructions[4],
            Instruction::LoadConst {
                dest_reg: 4,
                const_index: 0
            }
        ); // Reuses index 0
    }

    #[test]
    fn test_variable_name_deduplication() {
        let mut builder = BytecodeBuilder::new();

        // Add same variable name multiple times (use unique IDs for testing)
        builder.emit_load_var(0, "x", 1);
        builder.emit_load_var(1, "y", 2);
        builder.emit_load_var(2, "x", 1); // Duplicate name, same ID
        builder.emit_store_var("y", 2, 3); // Duplicate name, same ID
        builder.emit_store_var("z", 3, 4); // New name and ID

        let bytecode = builder.build();

        // Only 3 unique variable names should be in the pool
        assert_eq!(bytecode.var_names.len(), 3);
        assert_eq!(bytecode.var_names[0], "x");
        assert_eq!(bytecode.var_names[1], "y");
        assert_eq!(bytecode.var_names[2], "z");

        // Check that instructions reference correct indices
        assert_eq!(
            bytecode.instructions[0],
            Instruction::LoadVar {
                dest_reg: 0,
                var_name_index: 0,
                var_id: 1
            }
        );
        assert_eq!(
            bytecode.instructions[1],
            Instruction::LoadVar {
                dest_reg: 1,
                var_name_index: 1,
                var_id: 2
            }
        );
        assert_eq!(
            bytecode.instructions[2],
            Instruction::LoadVar {
                dest_reg: 2,
                var_name_index: 0,
                var_id: 1
            }
        ); // Reuses index 0
        assert_eq!(
            bytecode.instructions[3],
            Instruction::StoreVar {
                var_name_index: 1,
                var_id: 2,
                src_reg: 3
            }
        ); // Reuses index 1
        assert_eq!(
            bytecode.instructions[4],
            Instruction::StoreVar {
                var_name_index: 2,
                var_id: 3,
                src_reg: 4
            }
        );
    }

    #[test]
    fn test_all_emit_methods() {
        let mut builder = BytecodeBuilder::new();

        builder.emit_load_const(0, 10);
        builder.emit_load_const(1, 20);
        builder.emit_binary_op(2, 0, BinaryOperator::Add, 1);
        builder.emit_store_var("result", 1, 2);
        builder.emit_load_var(3, "result", 1);
        builder.emit_unary_op(4, UnaryOperator::Neg, 3);
        builder.emit_print(4);
        builder.emit_set_result(4);

        let bytecode = builder.build();

        // 8 instructions + 1 Halt
        assert_eq!(bytecode.instructions.len(), 9);

        // Check all instruction types are present
        assert!(matches!(
            bytecode.instructions[0],
            Instruction::LoadConst { .. }
        ));
        assert!(matches!(
            bytecode.instructions[1],
            Instruction::LoadConst { .. }
        ));
        assert!(matches!(
            bytecode.instructions[2],
            Instruction::BinaryOp { .. }
        ));
        assert!(matches!(
            bytecode.instructions[3],
            Instruction::StoreVar { .. }
        ));
        assert!(matches!(
            bytecode.instructions[4],
            Instruction::LoadVar { .. }
        ));
        assert!(matches!(
            bytecode.instructions[5],
            Instruction::UnaryOp { .. }
        ));
        assert!(matches!(
            bytecode.instructions[6],
            Instruction::Print { .. }
        ));
        assert!(matches!(
            bytecode.instructions[7],
            Instruction::SetResult { .. }
        ));
        assert_eq!(bytecode.instructions[8], Instruction::Halt);
    }

    #[test]
    fn test_build_appends_halt() {
        let mut builder = BytecodeBuilder::new();
        builder.emit_load_const(0, 1);

        let bytecode = builder.build();

        // Should have LoadConst + Halt
        assert_eq!(bytecode.instructions.len(), 2);
        assert_eq!(bytecode.instructions[1], Instruction::Halt);
    }

    #[test]
    fn test_empty_builder() {
        let builder = BytecodeBuilder::new();
        let bytecode = builder.build();

        // Should have only Halt instruction
        assert_eq!(bytecode.instructions.len(), 1);
        assert_eq!(bytecode.instructions[0], Instruction::Halt);
        assert_eq!(bytecode.constants.len(), 0);
        assert_eq!(bytecode.var_names.len(), 0);
    }

    #[test]
    fn test_bytecode_clone() {
        let mut builder = BytecodeBuilder::new();
        builder.emit_load_const(0, 42);
        let bytecode = builder.build();

        let cloned = bytecode.clone();
        assert_eq!(bytecode, cloned);
    }

    #[test]
    fn test_instruction_clone() {
        let inst = Instruction::BinaryOp {
            dest_reg: 0,
            left_reg: 1,
            op: BinaryOperator::Mul,
            right_reg: 2,
        };

        let cloned = inst.clone();
        assert_eq!(inst, cloned);
    }

    #[test]
    fn test_builder_default() {
        let builder = BytecodeBuilder::default();
        let bytecode = builder.build();

        assert_eq!(bytecode.instructions.len(), 1);
        assert_eq!(bytecode.instructions[0], Instruction::Halt);
    }

    #[test]
    fn test_all_binary_operators() {
        let mut builder = BytecodeBuilder::new();

        builder.emit_binary_op(0, 1, BinaryOperator::Add, 2);
        builder.emit_binary_op(0, 1, BinaryOperator::Sub, 2);
        builder.emit_binary_op(0, 1, BinaryOperator::Mul, 2);
        builder.emit_binary_op(0, 1, BinaryOperator::Div, 2);
        builder.emit_binary_op(0, 1, BinaryOperator::FloorDiv, 2);
        builder.emit_binary_op(0, 1, BinaryOperator::Mod, 2);

        let bytecode = builder.build();

        // 6 binary ops + Halt
        assert_eq!(bytecode.instructions.len(), 7);
    }

    #[test]
    fn test_all_unary_operators() {
        let mut builder = BytecodeBuilder::new();

        builder.emit_unary_op(0, UnaryOperator::Neg, 1);
        builder.emit_unary_op(0, UnaryOperator::Pos, 1);

        let bytecode = builder.build();

        // 2 unary ops + Halt
        assert_eq!(bytecode.instructions.len(), 3);
    }

    #[test]
    fn test_complex_program() {
        // Simulate: x = 10 + 20; y = x * 2; print(y)
        let mut builder = BytecodeBuilder::new();

        // Load constants
        builder.emit_load_const(0, 10);
        builder.emit_load_const(1, 20);
        // x = 10 + 20
        builder.emit_binary_op(2, 0, BinaryOperator::Add, 1);
        builder.emit_store_var("x", 1, 2);
        // Load x and constant 2
        builder.emit_load_var(3, "x", 1);
        builder.emit_load_const(4, 2);
        // y = x * 2
        builder.emit_binary_op(5, 3, BinaryOperator::Mul, 4);
        builder.emit_store_var("y", 2, 5);
        // print(y)
        builder.emit_load_var(6, "y", 2);
        builder.emit_print(6);

        let bytecode = builder.build();

        // Verify constants pool has 3 unique values (10, 20, 2)
        assert_eq!(bytecode.constants.len(), 3);
        assert!(bytecode.constants.contains(&10));
        assert!(bytecode.constants.contains(&20));
        assert!(bytecode.constants.contains(&2));

        // Verify variable names pool has 2 unique names (x, y)
        assert_eq!(bytecode.var_names.len(), 2);
        assert!(bytecode.var_names.contains(&"x".to_string()));
        assert!(bytecode.var_names.contains(&"y".to_string()));

        // 10 instructions + Halt
        assert_eq!(bytecode.instructions.len(), 11);
        assert_eq!(bytecode.instructions[10], Instruction::Halt);
    }

    #[test]
    fn test_negative_constants() {
        let mut builder = BytecodeBuilder::new();

        builder.emit_load_const(0, -42);
        builder.emit_load_const(1, 0);
        builder.emit_load_const(2, -100);
        builder.emit_load_const(3, -42); // Duplicate

        let bytecode = builder.build();

        // 3 unique constants
        assert_eq!(bytecode.constants.len(), 3);
        assert!(bytecode.constants.contains(&-42));
        assert!(bytecode.constants.contains(&0));
        assert!(bytecode.constants.contains(&-100));

        // Check deduplication worked
        if let Instruction::LoadConst { const_index, .. } = bytecode.instructions[0] {
            if let Instruction::LoadConst {
                const_index: dup_index,
                ..
            } = bytecode.instructions[3]
            {
                assert_eq!(const_index, dup_index);
            }
        }
    }

    // ========== Function Instruction Tests ==========

    #[test]
    fn test_emit_define_function_basic() {
        let mut builder = BytecodeBuilder::new();
        builder.emit_define_function("foo", 1, 2, 0, 10, 0);

        let bytecode = builder.build();

        assert_eq!(bytecode.instructions.len(), 2); // DefineFunction + Halt
        match &bytecode.instructions[0] {
            Instruction::DefineFunction {
                name_index,
                param_count,
                body_start,
                body_len,
                max_register_used,
            } => {
                assert_eq!(*name_index, 0);
                assert_eq!(*param_count, 2);
                assert_eq!(*body_start, 0);
                assert_eq!(*body_len, 10);
                assert_eq!(*max_register_used, 0);
            }
            _ => panic!("Expected DefineFunction instruction"),
        }

        // Function name should be in var_names pool
        assert_eq!(bytecode.var_names.len(), 1);
        assert_eq!(bytecode.var_names[0], "foo");
    }

    #[test]
    fn test_emit_call_basic() {
        let mut builder = BytecodeBuilder::new();
        builder.emit_call("foo", 1, 2, 0, 5);

        let bytecode = builder.build();

        assert_eq!(bytecode.instructions.len(), 2); // Call + Halt
        match &bytecode.instructions[0] {
            Instruction::Call {
                name_index,
                arg_count,
                first_arg_reg,
                dest_reg,
            } => {
                assert_eq!(*name_index, 0);
                assert_eq!(*arg_count, 2);
                assert_eq!(*first_arg_reg, 0);
                assert_eq!(*dest_reg, 5);
            }
            _ => panic!("Expected Call instruction"),
        }

        // Function name should be in var_names pool
        assert_eq!(bytecode.var_names.len(), 1);
        assert_eq!(bytecode.var_names[0], "foo");
    }

    #[test]
    fn test_emit_return_with_value() {
        let mut builder = BytecodeBuilder::new();
        builder.emit_return(true, Some(3));

        let bytecode = builder.build();

        assert_eq!(bytecode.instructions.len(), 2); // Return + Halt
        match &bytecode.instructions[0] {
            Instruction::Return { has_value, src_reg } => {
                assert!(*has_value);
                assert_eq!(*src_reg, Some(3));
            }
            _ => panic!("Expected Return instruction"),
        }
    }

    #[test]
    fn test_emit_return_without_value() {
        let mut builder = BytecodeBuilder::new();
        builder.emit_return(false, None);

        let bytecode = builder.build();

        assert_eq!(bytecode.instructions.len(), 2); // Return + Halt
        match &bytecode.instructions[0] {
            Instruction::Return { has_value, src_reg } => {
                assert!(!(*has_value));
                assert_eq!(*src_reg, None);
            }
            _ => panic!("Expected Return instruction"),
        }
    }

    #[test]
    fn test_function_name_deduplication() {
        let mut builder = BytecodeBuilder::new();

        // Define and call same function multiple times
        builder.emit_define_function("foo", 1, 0, 0, 5, 0);
        builder.emit_call("foo", 1, 0, 0, 1);
        builder.emit_call("foo", 1, 0, 0, 2); // Duplicate name
        builder.emit_define_function("bar", 2, 1, 5, 10, 1);
        builder.emit_call("foo", 1, 0, 0, 3); // Another duplicate

        let bytecode = builder.build();

        // Only 2 unique function names
        assert_eq!(bytecode.var_names.len(), 2);
        assert_eq!(bytecode.var_names[0], "foo");
        assert_eq!(bytecode.var_names[1], "bar");

        // Verify deduplication worked
        match &bytecode.instructions[0] {
            Instruction::DefineFunction { name_index, .. } => assert_eq!(*name_index, 0),
            _ => panic!("Expected DefineFunction"),
        }
        match &bytecode.instructions[1] {
            Instruction::Call { name_index, .. } => assert_eq!(*name_index, 0),
            _ => panic!("Expected Call"),
        }
        match &bytecode.instructions[2] {
            Instruction::Call { name_index, .. } => assert_eq!(*name_index, 0), // Reused index
            _ => panic!("Expected Call"),
        }
        match &bytecode.instructions[3] {
            Instruction::DefineFunction { name_index, .. } => assert_eq!(*name_index, 1),
            _ => panic!("Expected DefineFunction"),
        }
        match &bytecode.instructions[4] {
            Instruction::Call { name_index, .. } => assert_eq!(*name_index, 0), // Reused index
            _ => panic!("Expected Call"),
        }
    }

    #[test]
    fn test_function_with_zero_params() {
        let mut builder = BytecodeBuilder::new();
        builder.emit_define_function("no_params", 1, 0, 0, 1, 0);

        let bytecode = builder.build();

        match &bytecode.instructions[0] {
            Instruction::DefineFunction { param_count, .. } => {
                assert_eq!(*param_count, 0);
            }
            _ => panic!("Expected DefineFunction"),
        }
    }

    #[test]
    fn test_function_with_max_params() {
        let mut builder = BytecodeBuilder::new();
        builder.emit_define_function("many_params", 1, 255, 0, 1, 254); // u8 max

        let bytecode = builder.build();

        match &bytecode.instructions[0] {
            Instruction::DefineFunction { param_count, .. } => {
                assert_eq!(*param_count, 255);
            }
            _ => panic!("Expected DefineFunction"),
        }
    }

    #[test]
    fn test_call_with_zero_args() {
        let mut builder = BytecodeBuilder::new();
        builder.emit_call("no_args", 1, 0, 0, 1);

        let bytecode = builder.build();

        match &bytecode.instructions[0] {
            Instruction::Call { arg_count, .. } => {
                assert_eq!(*arg_count, 0);
            }
            _ => panic!("Expected Call"),
        }
    }

    #[test]
    fn test_call_with_max_args() {
        let mut builder = BytecodeBuilder::new();
        builder.emit_call("many_args", 1, 255, 0, 1); // u8 max

        let bytecode = builder.build();

        match &bytecode.instructions[0] {
            Instruction::Call { arg_count, .. } => {
                assert_eq!(*arg_count, 255);
            }
            _ => panic!("Expected Call"),
        }
    }

    #[test]
    fn test_complex_function_program() {
        // Simulate: def add(a, b): return a + b; result = add(10, 20)
        let mut builder = BytecodeBuilder::new();

        // Define function
        builder.emit_define_function("add", 1, 2, 0, 5, 3);

        // Call function
        builder.emit_load_const(0, 10);
        builder.emit_load_const(1, 20);
        builder.emit_call("add", 1, 2, 0, 2);
        builder.emit_store_var("result", 2, 2);

        // Return from function (would be in body)
        builder.emit_return(true, Some(3));

        let bytecode = builder.build();

        // Verify structure
        assert!(matches!(
            bytecode.instructions[0],
            Instruction::DefineFunction { .. }
        ));
        assert!(matches!(
            bytecode.instructions[1],
            Instruction::LoadConst { .. }
        ));
        assert!(matches!(
            bytecode.instructions[2],
            Instruction::LoadConst { .. }
        ));
        assert!(matches!(bytecode.instructions[3], Instruction::Call { .. }));
        assert!(matches!(
            bytecode.instructions[4],
            Instruction::StoreVar { .. }
        ));
        assert!(matches!(
            bytecode.instructions[5],
            Instruction::Return { .. }
        ));
        assert_eq!(bytecode.instructions[6], Instruction::Halt);

        // Verify pools
        assert_eq!(bytecode.constants.len(), 2);
        assert!(bytecode.constants.contains(&10));
        assert!(bytecode.constants.contains(&20));

        assert_eq!(bytecode.var_names.len(), 2); // "add" and "result"
        assert!(bytecode.var_names.contains(&"add".to_string()));
        assert!(bytecode.var_names.contains(&"result".to_string()));
    }

    #[test]
    fn test_nested_function_definitions() {
        let mut builder = BytecodeBuilder::new();

        builder.emit_define_function("outer", 1, 1, 0, 10, 1);
        builder.emit_define_function("inner", 2, 2, 10, 5, 2);
        builder.emit_call("inner", 2, 2, 0, 1);
        builder.emit_return(true, Some(1));
        builder.emit_call("outer", 1, 1, 0, 0);

        let bytecode = builder.build();

        // Verify all function names are tracked
        assert_eq!(bytecode.var_names.len(), 2);
        assert!(bytecode.var_names.contains(&"outer".to_string()));
        assert!(bytecode.var_names.contains(&"inner".to_string()));

        // Verify instruction sequence
        assert!(matches!(
            bytecode.instructions[0],
            Instruction::DefineFunction { .. }
        ));
        assert!(matches!(
            bytecode.instructions[1],
            Instruction::DefineFunction { .. }
        ));
        assert!(matches!(bytecode.instructions[2], Instruction::Call { .. }));
        assert!(matches!(
            bytecode.instructions[3],
            Instruction::Return { .. }
        ));
        assert!(matches!(bytecode.instructions[4], Instruction::Call { .. }));
    }

    #[test]
    fn test_function_and_variable_names_share_pool() {
        let mut builder = BytecodeBuilder::new();

        // Both functions and variables use var_names pool (all use same ID 1 for "x")
        builder.emit_define_function("x", 1, 0, 0, 1, 3);
        builder.emit_store_var("x", 1, 1); // Same name as function
        builder.emit_call("x", 1, 0, 0, 2);
        builder.emit_load_var(3, "x", 1);

        let bytecode = builder.build();

        // Should only have one entry for "x" due to deduplication
        assert_eq!(bytecode.var_names.len(), 1);
        assert_eq!(bytecode.var_names[0], "x");

        // All instructions should reference same index
        match &bytecode.instructions[0] {
            Instruction::DefineFunction { name_index, .. } => assert_eq!(*name_index, 0),
            _ => panic!("Expected DefineFunction"),
        }
        match &bytecode.instructions[1] {
            Instruction::StoreVar { var_name_index, .. } => assert_eq!(*var_name_index, 0),
            _ => panic!("Expected StoreVar"),
        }
        match &bytecode.instructions[2] {
            Instruction::Call { name_index, .. } => assert_eq!(*name_index, 0),
            _ => panic!("Expected Call"),
        }
        match &bytecode.instructions[3] {
            Instruction::LoadVar { var_name_index, .. } => assert_eq!(*var_name_index, 0),
            _ => panic!("Expected LoadVar"),
        }
    }

    #[test]
    fn test_empty_function_body() {
        let mut builder = BytecodeBuilder::new();
        builder.emit_define_function("empty", 1, 0, 0, 0, 0); // Zero-length body

        let bytecode = builder.build();

        match &bytecode.instructions[0] {
            Instruction::DefineFunction { body_len, .. } => {
                assert_eq!(*body_len, 0);
            }
            _ => panic!("Expected DefineFunction"),
        }
    }

    #[test]
    fn test_function_instruction_clone() {
        let inst1 = Instruction::DefineFunction {
            name_index: 0,
            param_count: 2,
            body_start: 5,
            body_len: 10,
            max_register_used: 3,
        };
        let cloned1 = inst1.clone();
        assert_eq!(inst1, cloned1);

        let inst2 = Instruction::Call {
            name_index: 1,
            arg_count: 3,
            first_arg_reg: 0,
            dest_reg: 4,
        };
        let cloned2 = inst2.clone();
        assert_eq!(inst2, cloned2);

        let inst3 = Instruction::Return {
            has_value: true,
            src_reg: Some(5),
        };
        let cloned3 = inst3.clone();
        assert_eq!(inst3, cloned3);
    }
}
