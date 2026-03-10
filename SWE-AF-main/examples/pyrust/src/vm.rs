//! Virtual Machine for bytecode execution
//!
//! Register-based VM with 256 preallocated registers for efficient execution.
//! Executes bytecode instructions and tracks program state including variables,
//! stdout output, and expression results.

use crate::bytecode::{Bytecode, Instruction};
use crate::error::RuntimeError;
use crate::value::Value;
use std::collections::HashMap;

/// Small string optimization for stdout buffer
///
/// Provides inline storage for strings ≤23 bytes to eliminate heap allocation
/// for simple print statements. Automatically promotes to heap for larger strings.
#[derive(Debug, Clone)]
enum SmallString {
    /// Inline storage for strings up to 23 bytes
    Inline { len: u8, data: [u8; 23] },
    /// Heap storage for strings > 23 bytes
    Heap(String),
}

impl SmallString {
    /// Create a new empty SmallString with inline storage
    #[inline]
    fn new() -> Self {
        SmallString::Inline {
            len: 0,
            data: [0; 23],
        }
    }

    /// Append a string slice to the SmallString
    ///
    /// Automatically promotes to heap storage if total size exceeds 23 bytes.
    fn push_str(&mut self, s: &str) {
        match self {
            SmallString::Inline { len, data } => {
                let current_len = *len as usize;
                let new_len = current_len + s.len();

                if new_len <= 23 {
                    // Can still fit inline
                    data[current_len..new_len].copy_from_slice(s.as_bytes());
                    *len = new_len as u8;
                } else {
                    // Promote to heap
                    let mut heap_string = String::with_capacity(new_len);
                    heap_string.push_str(
                        std::str::from_utf8(&data[..current_len])
                            .expect("Invalid UTF-8 in inline data"),
                    );
                    heap_string.push_str(s);
                    *self = SmallString::Heap(heap_string);
                }
            }
            SmallString::Heap(string) => {
                string.push_str(s);
            }
        }
    }

    /// Get a string slice view of the SmallString
    #[inline]
    fn as_str(&self) -> &str {
        match self {
            SmallString::Inline { len, data } => {
                std::str::from_utf8(&data[..*len as usize]).expect("Invalid UTF-8 in inline data")
            }
            SmallString::Heap(string) => string.as_str(),
        }
    }

    /// Check if the SmallString is empty
    #[inline]
    fn is_empty(&self) -> bool {
        match self {
            SmallString::Inline { len, .. } => *len == 0,
            SmallString::Heap(string) => string.is_empty(),
        }
    }
}

/// Function metadata stored in the VM
#[derive(Debug, Clone)]
struct FunctionMetadata {
    /// Parameter count
    param_count: u8,
    /// Start index of function body in bytecode
    body_start: usize,
    /// Maximum register used in this function (optional for backward compat)
    max_register_used: Option<u8>,
}

/// Call frame for function execution
#[derive(Debug, Clone)]
struct CallFrame {
    /// Return address (instruction pointer to resume after return)
    return_address: usize,
    /// Local variables for this function scope using interned IDs
    local_vars: HashMap<u32, Value>,
    /// Saved registers state (only used registers)
    saved_registers: Vec<Value>,
    /// Saved register validity bitmap
    saved_register_valid: [u64; 4],
    /// Maximum register that was saved
    max_saved_reg: u8,
    /// Register where return value should be stored
    dest_reg: u8,
}

/// Virtual Machine for bytecode execution
///
/// Provides a register-based execution environment with:
/// - 256 preallocated registers for fast value manipulation
/// - Bitmap-based register validity tracking for optimal performance
/// - Variable storage using HashMap
/// - stdout capture for print statements
/// - Result tracking for expression statements
/// - Function call stack for nested function calls
pub struct VM {
    /// Preallocated register file (256 registers)
    registers: Vec<Value>,

    /// Register validity bitmap (4 x u64 = 256 bits for 256 registers)
    register_valid: [u64; 4],

    /// Current instruction pointer for accurate error reporting
    ip: usize,

    /// Variable storage (interned ID -> value) - global scope
    variables: HashMap<u32, Value>,

    /// Accumulated stdout output from print statements
    stdout: SmallString,

    /// Result from last SetResult instruction
    result: Option<Value>,

    /// Function storage (name -> metadata)
    functions: HashMap<String, FunctionMetadata>,

    /// Call stack for function calls
    call_stack: Vec<CallFrame>,
}

impl VM {
    /// Create a new VM with preallocated 256-register file
    ///
    /// All registers are initialized to Value::Integer(0) with validity bits cleared.
    /// Variables HashMap starts empty.
    /// stdout buffer and result are empty/None.
    pub fn new() -> Self {
        Self {
            registers: vec![Value::Integer(0); 256],
            register_valid: [0; 4],
            ip: 0,
            variables: HashMap::new(),
            stdout: SmallString::new(),
            result: None,
            functions: HashMap::new(),
            call_stack: Vec::new(),
        }
    }

    /// Check if a register is valid (has been set)
    #[inline]
    fn is_register_valid(&self, reg: u8) -> bool {
        let word_idx = (reg as usize) / 64;
        let bit_idx = (reg as usize) % 64;
        (self.register_valid[word_idx] & (1u64 << bit_idx)) != 0
    }

    /// Mark a register as valid
    #[inline]
    fn set_register_valid(&mut self, reg: u8) {
        let word_idx = (reg as usize) / 64;
        let bit_idx = (reg as usize) % 64;
        self.register_valid[word_idx] |= 1u64 << bit_idx;
    }

    /// Get a register value, returning error if invalid
    #[inline]
    fn get_register(&self, reg: u8) -> Result<Value, RuntimeError> {
        if self.is_register_valid(reg) {
            Ok(self.registers[reg as usize])
        } else {
            Err(RuntimeError {
                message: format!("Register {} is empty", reg),
                instruction_index: self.ip,
            })
        }
    }

    /// Set a register value and mark it as valid
    #[inline]
    fn set_register(&mut self, reg: u8, value: Value) {
        self.registers[reg as usize] = value;
        self.set_register_valid(reg);
    }

    /// Save register state for function call (only saves registers [0..=max_reg])
    fn save_register_state(&self, max_reg: u8) -> Vec<Value> {
        let count = (max_reg as usize) + 1;
        self.registers[0..count].to_vec()
    }

    /// Restore register state after function return
    fn restore_register_state(
        &mut self,
        saved: Vec<Value>,
        saved_valid: [u64; 4],
        max_saved_reg: u8,
    ) {
        // Restore saved registers
        let count = (max_saved_reg as usize) + 1;
        self.registers[0..count].copy_from_slice(&saved[0..count]);

        // Restore validity bitmap
        self.register_valid = saved_valid;
    }

    /// Execute bytecode program
    ///
    /// Returns:
    /// - `Ok(None)` - No expression statements executed (only assignments/prints)
    /// - `Ok(Some(value))` - Expression statement executed, returning its value
    /// - `Err(RuntimeError)` - Execution failed with runtime error
    ///
    /// # Errors
    /// - Division by zero during BinaryOp execution
    /// - Undefined variable access during LoadVar
    /// - Integer overflow during arithmetic operations
    pub fn execute(&mut self, bytecode: &Bytecode) -> Result<Option<Value>, RuntimeError> {
        self.ip = 0; // Instruction pointer

        loop {
            if self.ip >= bytecode.instructions.len() {
                return Err(RuntimeError {
                    message: "Instruction pointer out of bounds".to_string(),
                    instruction_index: self.ip,
                });
            }

            let instruction = &bytecode.instructions[self.ip];

            match instruction {
                Instruction::LoadConst {
                    dest_reg,
                    const_index,
                } => {
                    if *const_index >= bytecode.constants.len() {
                        return Err(RuntimeError {
                            message: format!("Constant index {} out of bounds", const_index),
                            instruction_index: self.ip,
                        });
                    }
                    let value = bytecode.constants[*const_index];
                    self.set_register(*dest_reg, Value::Integer(value));
                }

                Instruction::LoadVar {
                    dest_reg,
                    var_name_index,
                    var_id,
                } => {
                    if *var_name_index >= bytecode.var_names.len() {
                        return Err(RuntimeError {
                            message: format!(
                                "Variable name index {} out of bounds",
                                var_name_index
                            ),
                            instruction_index: self.ip,
                        });
                    }
                    let var_name = &bytecode.var_names[*var_name_index];

                    // Check local scope first if we're in a function, then global scope
                    let value = if let Some(frame) = self.call_stack.last() {
                        frame
                            .local_vars
                            .get(var_id)
                            .or_else(|| self.variables.get(var_id))
                    } else {
                        self.variables.get(var_id)
                    };

                    match value {
                        Some(val) => {
                            self.set_register(*dest_reg, *val);
                        }
                        None => {
                            return Err(RuntimeError {
                                message: format!("Undefined variable: {}", var_name),
                                instruction_index: self.ip,
                            });
                        }
                    }
                }

                Instruction::StoreVar {
                    var_name_index,
                    var_id,
                    src_reg,
                } => {
                    if *var_name_index >= bytecode.var_names.len() {
                        return Err(RuntimeError {
                            message: format!(
                                "Variable name index {} out of bounds",
                                var_name_index
                            ),
                            instruction_index: self.ip,
                        });
                    }
                    let value = self.get_register(*src_reg)?;

                    // Store in local scope if we're in a function, otherwise in global scope
                    if let Some(frame) = self.call_stack.last_mut() {
                        frame.local_vars.insert(*var_id, value);
                    } else {
                        self.variables.insert(*var_id, value);
                    }
                }

                Instruction::BinaryOp {
                    dest_reg,
                    left_reg,
                    op,
                    right_reg,
                } => {
                    let left = self.get_register(*left_reg)?;
                    let right = self.get_register(*right_reg)?;

                    let result = left.binary_op(*op, &right).map_err(|mut e| {
                        e.instruction_index = self.ip;
                        e
                    })?;

                    self.set_register(*dest_reg, result);
                }

                Instruction::UnaryOp {
                    dest_reg,
                    op,
                    operand_reg,
                } => {
                    let operand = self.get_register(*operand_reg)?;

                    let result = operand.unary_op(*op).map_err(|mut e| {
                        e.instruction_index = self.ip;
                        e
                    })?;

                    self.set_register(*dest_reg, result);
                }

                Instruction::Print { src_reg } => {
                    let value = self.get_register(*src_reg)?;
                    self.stdout.push_str(&format!("{}\n", value));
                }

                Instruction::SetResult { src_reg } => {
                    let value = self.get_register(*src_reg)?;
                    self.result = Some(value);
                }

                Instruction::Halt => {
                    break;
                }

                Instruction::DefineFunction {
                    name_index,
                    param_count,
                    body_start,
                    body_len: _,
                    max_register_used,
                } => {
                    // Store function metadata
                    if *name_index >= bytecode.var_names.len() {
                        return Err(RuntimeError {
                            message: format!("Function name index {} out of bounds", name_index),
                            instruction_index: self.ip,
                        });
                    }
                    let func_name = bytecode.var_names[*name_index].clone();
                    self.functions.insert(
                        func_name,
                        FunctionMetadata {
                            param_count: *param_count,
                            body_start: *body_start,
                            max_register_used: Some(*max_register_used),
                        },
                    );
                    // Don't skip - just register the function and continue
                }

                Instruction::Call {
                    name_index,
                    arg_count,
                    first_arg_reg,
                    dest_reg,
                } => {
                    // Look up function
                    if *name_index >= bytecode.var_names.len() {
                        return Err(RuntimeError {
                            message: format!("Function name index {} out of bounds", name_index),
                            instruction_index: self.ip,
                        });
                    }
                    let func_name = &bytecode.var_names[*name_index];

                    let func_meta = self
                        .functions
                        .get(func_name)
                        .ok_or_else(|| RuntimeError {
                            message: format!("Undefined function: {}", func_name),
                            instruction_index: self.ip,
                        })?
                        .clone();

                    // Check argument count
                    if *arg_count != func_meta.param_count {
                        return Err(RuntimeError {
                            message: format!(
                                "Function {} expects {} arguments, got {}",
                                func_name, func_meta.param_count, arg_count
                            ),
                            instruction_index: self.ip,
                        });
                    }

                    // Create new call frame
                    let mut local_vars = HashMap::new();

                    // Pass arguments as local variables (param_0, param_1, ...)
                    // IMPORTANT: Parameters are stored in local_vars HashMap, NOT in registers.
                    // This prevents register allocation collisions when parameters are used
                    // in multiple operations (e.g., x+1, x*2, x-3 all use the same parameter x).
                    // The compiler allocates fresh registers for each LoadVar instruction,
                    // ensuring that intermediate values don't overwrite parameter values.
                    for i in 0..*arg_count {
                        let arg_reg = (*first_arg_reg as usize + i as usize) as u8;
                        let arg_value = self.get_register(arg_reg)?;

                        // Find the var_id for param_i by looking up the name in bytecode
                        let param_name = format!("param_{}", i);
                        let param_var_id = bytecode
                            .var_names
                            .iter()
                            .position(|n| n == &param_name)
                            .and_then(|idx| bytecode.var_ids.get(idx).copied())
                            .ok_or_else(|| RuntimeError {
                                message: format!("Parameter {} not found in bytecode", param_name),
                                instruction_index: self.ip,
                            })?;

                        local_vars.insert(param_var_id, arg_value);
                    }

                    // Determine how many registers to save
                    // Use metadata if available, otherwise save all (backward compat)
                    let max_reg_to_save = func_meta.max_register_used.unwrap_or(255);
                    let saved_registers = self.save_register_state(max_reg_to_save);
                    let saved_register_valid = self.register_valid;

                    let call_frame = CallFrame {
                        return_address: self.ip + 1,
                        local_vars,
                        saved_registers,
                        saved_register_valid,
                        max_saved_reg: max_reg_to_save,
                        dest_reg: *dest_reg,
                    };

                    self.call_stack.push(call_frame);

                    // Jump to function body
                    self.ip = func_meta.body_start;
                    continue; // Skip ip increment at end of loop
                }

                Instruction::Return { has_value, src_reg } => {
                    // CAPTURE return value BEFORE popping frame
                    // This ensures parameters are still accessible if needed
                    let return_value = if *has_value {
                        let return_reg = src_reg.ok_or_else(|| RuntimeError {
                            message: "Return with value but no register specified".to_string(),
                            instruction_index: self.ip,
                        })?;
                        self.get_register(return_reg)?
                    } else {
                        Value::None
                    };

                    // NOW safe to pop call frame
                    let call_frame = self.call_stack.pop().ok_or_else(|| RuntimeError {
                        message: "Return outside of function".to_string(),
                        instruction_index: self.ip,
                    })?;

                    // Restore registers using optimized method
                    self.restore_register_state(
                        call_frame.saved_registers,
                        call_frame.saved_register_valid,
                        call_frame.max_saved_reg,
                    );

                    // Set return value in destination register
                    self.set_register(call_frame.dest_reg, return_value);

                    // Jump back to return address
                    self.ip = call_frame.return_address;
                    continue; // Skip ip increment at end of loop
                }
            }

            self.ip += 1;
        }

        Ok(self.result)
    }

    /// Format output according to output specification
    ///
    /// Returns formatted string combining stdout and result:
    /// - If only stdout: returns stdout as-is
    /// - If only result: returns result value as string
    /// - If both: returns stdout followed by result value
    /// - If neither: returns empty string
    ///
    /// # Arguments
    /// * `result` - The result value from execute()
    pub fn format_output(&self, result: Option<Value>) -> String {
        let has_stdout = !self.stdout.is_empty();
        let has_result = result.is_some();

        match (has_stdout, has_result) {
            (true, true) => {
                // Both stdout and result: stdout + result value
                format!("{}{}", self.stdout.as_str(), result.unwrap())
            }
            (true, false) => {
                // Only stdout: return as-is
                self.stdout.as_str().to_string()
            }
            (false, true) => {
                // Only result: return result value as string
                format!("{}", result.unwrap())
            }
            (false, false) => {
                // Neither: return empty string
                String::new()
            }
        }
    }
}

impl Default for VM {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::{BinaryOperator, UnaryOperator};
    use crate::bytecode::BytecodeBuilder;

    #[test]
    fn test_vm_new() {
        let vm = VM::new();
        assert_eq!(vm.registers.len(), 256);
        assert!(vm.variables.is_empty());
        assert!(vm.stdout.is_empty());
        assert!(vm.result.is_none());
    }

    #[test]
    fn test_execute_load_const() {
        let mut builder = BytecodeBuilder::new();
        builder.emit_load_const(0, 42);
        let bytecode = builder.build();

        let mut vm = VM::new();
        let result = vm.execute(&bytecode).unwrap();

        assert_eq!(result, None);
        assert_eq!(vm.registers[0], Value::Integer(42));
    }

    #[test]
    fn test_execute_store_and_load_var() {
        let mut builder = BytecodeBuilder::new();
        builder.emit_load_const(0, 100);
        builder.emit_store_var("x", 1, 0);
        builder.emit_load_var(1, "x", 1);
        let bytecode = builder.build();

        let mut vm = VM::new();
        let result = vm.execute(&bytecode).unwrap();

        assert_eq!(result, None);
        assert_eq!(vm.registers[1], Value::Integer(100));
        assert_eq!(vm.variables.get(&1), Some(&Value::Integer(100)));
    }

    #[test]
    fn test_execute_binary_op_add() {
        let mut builder = BytecodeBuilder::new();
        builder.emit_load_const(0, 10);
        builder.emit_load_const(1, 20);
        builder.emit_binary_op(2, 0, BinaryOperator::Add, 1);
        let bytecode = builder.build();

        let mut vm = VM::new();
        vm.execute(&bytecode).unwrap();

        assert_eq!(vm.registers[2], Value::Integer(30));
    }

    #[test]
    fn test_execute_binary_op_all_operators() {
        // Test Add
        let mut builder = BytecodeBuilder::new();
        builder.emit_load_const(0, 10);
        builder.emit_load_const(1, 3);
        builder.emit_binary_op(2, 0, BinaryOperator::Add, 1);
        let bytecode = builder.build();
        let mut vm = VM::new();
        vm.execute(&bytecode).unwrap();
        assert_eq!(vm.registers[2], Value::Integer(13));

        // Test Sub
        let mut builder = BytecodeBuilder::new();
        builder.emit_load_const(0, 10);
        builder.emit_load_const(1, 3);
        builder.emit_binary_op(2, 0, BinaryOperator::Sub, 1);
        let bytecode = builder.build();
        let mut vm = VM::new();
        vm.execute(&bytecode).unwrap();
        assert_eq!(vm.registers[2], Value::Integer(7));

        // Test Mul
        let mut builder = BytecodeBuilder::new();
        builder.emit_load_const(0, 10);
        builder.emit_load_const(1, 3);
        builder.emit_binary_op(2, 0, BinaryOperator::Mul, 1);
        let bytecode = builder.build();
        let mut vm = VM::new();
        vm.execute(&bytecode).unwrap();
        assert_eq!(vm.registers[2], Value::Integer(30));

        // Test Div
        let mut builder = BytecodeBuilder::new();
        builder.emit_load_const(0, 10);
        builder.emit_load_const(1, 3);
        builder.emit_binary_op(2, 0, BinaryOperator::Div, 1);
        let bytecode = builder.build();
        let mut vm = VM::new();
        vm.execute(&bytecode).unwrap();
        assert_eq!(vm.registers[2], Value::Integer(3));

        // Test FloorDiv
        let mut builder = BytecodeBuilder::new();
        builder.emit_load_const(0, 10);
        builder.emit_load_const(1, 3);
        builder.emit_binary_op(2, 0, BinaryOperator::FloorDiv, 1);
        let bytecode = builder.build();
        let mut vm = VM::new();
        vm.execute(&bytecode).unwrap();
        assert_eq!(vm.registers[2], Value::Integer(3));

        // Test Mod
        let mut builder = BytecodeBuilder::new();
        builder.emit_load_const(0, 10);
        builder.emit_load_const(1, 3);
        builder.emit_binary_op(2, 0, BinaryOperator::Mod, 1);
        let bytecode = builder.build();
        let mut vm = VM::new();
        vm.execute(&bytecode).unwrap();
        assert_eq!(vm.registers[2], Value::Integer(1));
    }

    #[test]
    fn test_execute_unary_op() {
        // Test Neg
        let mut builder = BytecodeBuilder::new();
        builder.emit_load_const(0, 42);
        builder.emit_unary_op(1, UnaryOperator::Neg, 0);
        let bytecode = builder.build();

        let mut vm = VM::new();
        vm.execute(&bytecode).unwrap();

        assert_eq!(vm.registers[1], Value::Integer(-42));

        // Test Pos
        let mut builder = BytecodeBuilder::new();
        builder.emit_load_const(0, 42);
        builder.emit_unary_op(1, UnaryOperator::Pos, 0);
        let bytecode = builder.build();

        let mut vm = VM::new();
        vm.execute(&bytecode).unwrap();

        assert_eq!(vm.registers[1], Value::Integer(42));
    }

    #[test]
    fn test_execute_print() {
        let mut builder = BytecodeBuilder::new();
        builder.emit_load_const(0, 42);
        builder.emit_print(0);
        builder.emit_load_const(1, 100);
        builder.emit_print(1);
        let bytecode = builder.build();

        let mut vm = VM::new();
        vm.execute(&bytecode).unwrap();

        assert_eq!(vm.stdout.as_str(), "42\n100\n");
    }

    #[test]
    fn test_execute_set_result() {
        let mut builder = BytecodeBuilder::new();
        builder.emit_load_const(0, 42);
        builder.emit_set_result(0);
        let bytecode = builder.build();

        let mut vm = VM::new();
        let result = vm.execute(&bytecode).unwrap();

        assert_eq!(result, Some(Value::Integer(42)));
        assert_eq!(vm.result, Some(Value::Integer(42)));
    }

    #[test]
    fn test_execute_returns_none_for_assignments() {
        let mut builder = BytecodeBuilder::new();
        builder.emit_load_const(0, 42);
        builder.emit_store_var("x", 1, 0);
        let bytecode = builder.build();

        let mut vm = VM::new();
        let result = vm.execute(&bytecode).unwrap();

        assert_eq!(result, None);
    }

    #[test]
    fn test_execute_returns_some_for_expression_statement() {
        let mut builder = BytecodeBuilder::new();
        builder.emit_load_const(0, 42);
        builder.emit_set_result(0);
        let bytecode = builder.build();

        let mut vm = VM::new();
        let result = vm.execute(&bytecode).unwrap();

        assert_eq!(result, Some(Value::Integer(42)));
    }

    #[test]
    fn test_division_by_zero_error() {
        let mut builder = BytecodeBuilder::new();
        builder.emit_load_const(0, 10);
        builder.emit_load_const(1, 0);
        builder.emit_binary_op(2, 0, BinaryOperator::Div, 1);
        let bytecode = builder.build();

        let mut vm = VM::new();
        let result = vm.execute(&bytecode);

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.message, "Division by zero");
        assert_eq!(err.instruction_index, 2);
    }

    #[test]
    fn test_undefined_variable_error() {
        let mut builder = BytecodeBuilder::new();
        builder.emit_load_var(0, "undefined_var", 1);
        let bytecode = builder.build();

        let mut vm = VM::new();
        let result = vm.execute(&bytecode);

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.message.contains("Undefined variable"));
        assert!(err.message.contains("undefined_var"));
        assert_eq!(err.instruction_index, 0);
    }

    #[test]
    fn test_format_output_only_stdout() {
        let mut vm = VM::new();
        vm.stdout.push_str("42\n100\n");

        let output = vm.format_output(None);
        assert_eq!(output, "42\n100\n");
    }

    #[test]
    fn test_format_output_only_result() {
        let vm = VM::new();

        let output = vm.format_output(Some(Value::Integer(42)));
        assert_eq!(output, "42");
    }

    #[test]
    fn test_format_output_both() {
        let mut vm = VM::new();
        vm.stdout.push_str("100\n");

        let output = vm.format_output(Some(Value::Integer(42)));
        assert_eq!(output, "100\n42");
    }

    #[test]
    fn test_format_output_neither() {
        let vm = VM::new();

        let output = vm.format_output(None);
        assert_eq!(output, "");
    }

    #[test]
    fn test_complex_program() {
        // Simulate: x = 10 + 20; y = x * 2; print(y); y
        let mut builder = BytecodeBuilder::new();

        // x = 10 + 20
        builder.emit_load_const(0, 10);
        builder.emit_load_const(1, 20);
        builder.emit_binary_op(2, 0, BinaryOperator::Add, 1);
        builder.emit_store_var("x", 1, 2);

        // y = x * 2
        builder.emit_load_var(3, "x", 1);
        builder.emit_load_const(4, 2);
        builder.emit_binary_op(5, 3, BinaryOperator::Mul, 4);
        builder.emit_store_var("y", 2, 5);

        // print(y)
        builder.emit_load_var(6, "y", 2);
        builder.emit_print(6);

        // y (expression statement)
        builder.emit_load_var(7, "y", 2);
        builder.emit_set_result(7);

        let bytecode = builder.build();

        let mut vm = VM::new();
        let result = vm.execute(&bytecode).unwrap();

        assert_eq!(result, Some(Value::Integer(60)));
        assert_eq!(vm.stdout.as_str(), "60\n");
        assert_eq!(vm.variables.get(&1), Some(&Value::Integer(30)));
        assert_eq!(vm.variables.get(&2), Some(&Value::Integer(60)));

        let output = vm.format_output(result);
        assert_eq!(output, "60\n60");
    }

    #[test]
    fn test_halt_instruction() {
        let mut builder = BytecodeBuilder::new();
        builder.emit_load_const(0, 42);
        let bytecode = builder.build();

        let mut vm = VM::new();
        let result = vm.execute(&bytecode);

        assert!(result.is_ok());
    }

    #[test]
    fn test_vm_default() {
        let vm = VM::default();
        assert_eq!(vm.registers.len(), 256);
    }

    #[test]
    fn test_register_bounds() {
        let mut builder = BytecodeBuilder::new();
        // Test register 255 (max valid register)
        builder.emit_load_const(255, 42);
        let bytecode = builder.build();

        let mut vm = VM::new();
        let result = vm.execute(&bytecode);

        assert!(result.is_ok());
        assert_eq!(vm.registers[255], Value::Integer(42));
    }

    #[test]
    fn test_multiple_set_result() {
        // Test that multiple SetResult instructions overwrite the result
        let mut builder = BytecodeBuilder::new();
        builder.emit_load_const(0, 10);
        builder.emit_set_result(0);
        builder.emit_load_const(1, 20);
        builder.emit_set_result(1);
        let bytecode = builder.build();

        let mut vm = VM::new();
        let result = vm.execute(&bytecode).unwrap();

        // Should return the last SetResult value
        assert_eq!(result, Some(Value::Integer(20)));
    }

    #[test]
    fn test_expression_with_negatives() {
        // Test: -5 + -3
        let mut builder = BytecodeBuilder::new();
        builder.emit_load_const(0, 5);
        builder.emit_unary_op(1, UnaryOperator::Neg, 0);
        builder.emit_load_const(2, 3);
        builder.emit_unary_op(3, UnaryOperator::Neg, 2);
        builder.emit_binary_op(4, 1, BinaryOperator::Add, 3);
        let bytecode = builder.build();

        let mut vm = VM::new();
        vm.execute(&bytecode).unwrap();

        assert_eq!(vm.registers[4], Value::Integer(-8));
    }

    #[test]
    fn test_modulo_operation() {
        let mut builder = BytecodeBuilder::new();
        builder.emit_load_const(0, 10);
        builder.emit_load_const(1, 3);
        builder.emit_binary_op(2, 0, BinaryOperator::Mod, 1);
        let bytecode = builder.build();

        let mut vm = VM::new();
        vm.execute(&bytecode).unwrap();

        assert_eq!(vm.registers[2], Value::Integer(1));
    }

    #[test]
    fn test_floor_division_by_zero() {
        let mut builder = BytecodeBuilder::new();
        builder.emit_load_const(0, 10);
        builder.emit_load_const(1, 0);
        builder.emit_binary_op(2, 0, BinaryOperator::FloorDiv, 1);
        let bytecode = builder.build();

        let mut vm = VM::new();
        let result = vm.execute(&bytecode);

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.message, "Division by zero");
        assert_eq!(err.instruction_index, 2);
    }

    #[test]
    fn test_modulo_by_zero() {
        let mut builder = BytecodeBuilder::new();
        builder.emit_load_const(0, 10);
        builder.emit_load_const(1, 0);
        builder.emit_binary_op(2, 0, BinaryOperator::Mod, 1);
        let bytecode = builder.build();

        let mut vm = VM::new();
        let result = vm.execute(&bytecode);

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.message, "Division by zero");
        assert_eq!(err.instruction_index, 2);
    }

    // ========== Function Execution Tests ==========

    #[test]
    fn test_define_function_stores_metadata() {
        let instructions = vec![
            Instruction::DefineFunction {
                name_index: 0,
                param_count: 2,
                body_start: 2,
                body_len: 1,
                max_register_used: 1,
            },
            Instruction::Halt,
            Instruction::Return {
                has_value: false,
                src_reg: None,
            },
        ];

        let bytecode = Bytecode {
            instructions,
            constants: vec![],
            var_names: vec!["foo".to_string()],
            var_ids: vec![1],
            metadata: crate::bytecode::CompilerMetadata {
                max_register_used: 255,
            },
        };

        let mut vm = VM::new();
        vm.execute(&bytecode).unwrap();

        // Verify function was stored
        assert!(vm.functions.contains_key("foo"));
        let func = &vm.functions["foo"];
        assert_eq!(func.param_count, 2);
        assert_eq!(func.body_start, 2);
    }

    #[test]
    fn test_zero_param_function_call() {
        // def foo(): return 42
        // result = foo()

        // Manually build bytecode with proper layout:
        // 0: DefineFunction foo
        // 1: Call foo
        // 2: SetResult
        // 3: Halt
        // 4: LoadConst 42
        // 5: Return

        let instructions = vec![
            Instruction::DefineFunction {
                name_index: 0,
                param_count: 0,
                body_start: 4,
                body_len: 2,
                max_register_used: 0,
            },
            Instruction::Call {
                name_index: 0,
                arg_count: 0,
                first_arg_reg: 0,
                dest_reg: 5,
            },
            Instruction::SetResult { src_reg: 5 },
            Instruction::Halt,
            Instruction::LoadConst {
                dest_reg: 10,
                const_index: 0,
            },
            Instruction::Return {
                has_value: true,
                src_reg: Some(10),
            },
        ];

        let bytecode = Bytecode {
            instructions,
            constants: vec![42],
            var_names: vec!["foo".to_string()],
            var_ids: vec![1],
            metadata: crate::bytecode::CompilerMetadata {
                max_register_used: 255,
            },
        };

        let mut vm = VM::new();
        let result = vm.execute(&bytecode).unwrap();

        assert_eq!(result, Some(Value::Integer(42)));
    }

    #[test]
    fn test_function_with_one_parameter() {
        // def double(x): return x * 2
        // result = double(21)

        let instructions = vec![
            Instruction::DefineFunction {
                name_index: 0,
                param_count: 1,
                body_start: 5,
                body_len: 4,
                max_register_used: 2,
            },
            Instruction::LoadConst {
                dest_reg: 0,
                const_index: 0,
            },
            Instruction::Call {
                name_index: 0,
                arg_count: 1,
                first_arg_reg: 0,
                dest_reg: 5,
            },
            Instruction::SetResult { src_reg: 5 },
            Instruction::Halt,
            Instruction::LoadVar {
                dest_reg: 10,
                var_name_index: 1,
                var_id: 2,
            },
            Instruction::LoadConst {
                dest_reg: 11,
                const_index: 1,
            },
            Instruction::BinaryOp {
                dest_reg: 12,
                left_reg: 10,
                op: BinaryOperator::Mul,
                right_reg: 11,
            },
            Instruction::Return {
                has_value: true,
                src_reg: Some(12),
            },
        ];

        let bytecode = Bytecode {
            instructions,
            constants: vec![21, 2],
            var_names: vec!["double".to_string(), "param_0".to_string()],
            var_ids: vec![1, 2],
            metadata: crate::bytecode::CompilerMetadata {
                max_register_used: 255,
            },
        };

        let mut vm = VM::new();
        let result = vm.execute(&bytecode).unwrap();

        assert_eq!(result, Some(Value::Integer(42)));
    }

    #[test]
    fn test_function_with_two_parameters() {
        // def add(a, b): return a + b
        // result = add(10, 20)

        let instructions = vec![
            Instruction::DefineFunction {
                name_index: 0,
                param_count: 2,
                body_start: 6,
                body_len: 4,
                max_register_used: 3,
            },
            Instruction::LoadConst {
                dest_reg: 0,
                const_index: 0,
            },
            Instruction::LoadConst {
                dest_reg: 1,
                const_index: 1,
            },
            Instruction::Call {
                name_index: 0,
                arg_count: 2,
                first_arg_reg: 0,
                dest_reg: 5,
            },
            Instruction::SetResult { src_reg: 5 },
            Instruction::Halt,
            Instruction::LoadVar {
                dest_reg: 10,
                var_name_index: 1,
                var_id: 2,
            },
            Instruction::LoadVar {
                dest_reg: 11,
                var_name_index: 2,
                var_id: 3,
            },
            Instruction::BinaryOp {
                dest_reg: 12,
                left_reg: 10,
                op: BinaryOperator::Add,
                right_reg: 11,
            },
            Instruction::Return {
                has_value: true,
                src_reg: Some(12),
            },
        ];

        let bytecode = Bytecode {
            instructions,
            constants: vec![10, 20],
            var_names: vec![
                "add".to_string(),
                "param_0".to_string(),
                "param_1".to_string(),
            ],
            var_ids: vec![1, 2, 3],
            metadata: crate::bytecode::CompilerMetadata {
                max_register_used: 255,
            },
        };

        let mut vm = VM::new();
        let result = vm.execute(&bytecode).unwrap();

        assert_eq!(result, Some(Value::Integer(30)));
    }

    #[test]
    fn test_function_return_without_value() {
        // def no_return(): return
        // result = no_return()

        let instructions = vec![
            Instruction::DefineFunction {
                name_index: 0,
                param_count: 0,
                body_start: 4,
                body_len: 1,
                max_register_used: 0,
            },
            Instruction::Call {
                name_index: 0,
                arg_count: 0,
                first_arg_reg: 0,
                dest_reg: 5,
            },
            Instruction::SetResult { src_reg: 5 },
            Instruction::Halt,
            Instruction::Return {
                has_value: false,
                src_reg: None,
            },
        ];

        let bytecode = Bytecode {
            instructions,
            constants: vec![],
            var_names: vec!["no_return".to_string()],
            var_ids: vec![1],
            metadata: crate::bytecode::CompilerMetadata {
                max_register_used: 255,
            },
        };

        let mut vm = VM::new();
        let result = vm.execute(&bytecode).unwrap();

        // Should return None value
        assert_eq!(result, Some(Value::None));
    }

    #[test]
    fn test_local_scope_isolation() {
        // x = 5
        // def foo(): x = 10; return x
        // result = foo()
        // After call, global x should still be 5

        let instructions = vec![
            Instruction::LoadConst {
                dest_reg: 0,
                const_index: 0,
            },
            Instruction::StoreVar {
                var_name_index: 0,
                var_id: 1,
                src_reg: 0,
            },
            Instruction::DefineFunction {
                name_index: 1,
                param_count: 0,
                body_start: 6,
                body_len: 4,
                max_register_used: 2,
            },
            Instruction::Call {
                name_index: 1,
                arg_count: 0,
                first_arg_reg: 0,
                dest_reg: 5,
            },
            Instruction::SetResult { src_reg: 5 },
            Instruction::Halt,
            Instruction::LoadConst {
                dest_reg: 10,
                const_index: 1,
            },
            Instruction::StoreVar {
                var_name_index: 0,
                var_id: 1,
                src_reg: 10,
            },
            Instruction::LoadVar {
                dest_reg: 11,
                var_name_index: 0,
                var_id: 1,
            },
            Instruction::Return {
                has_value: true,
                src_reg: Some(11),
            },
        ];

        let bytecode = Bytecode {
            instructions,
            constants: vec![5, 10],
            var_names: vec!["x".to_string(), "foo".to_string()],
            var_ids: vec![1, 2],
            metadata: crate::bytecode::CompilerMetadata {
                max_register_used: 255,
            },
        };

        let mut vm = VM::new();
        let result = vm.execute(&bytecode).unwrap();

        // Function should return 10
        assert_eq!(result, Some(Value::Integer(10)));
        // Global x should still be 5
        assert_eq!(vm.variables.get(&1), Some(&Value::Integer(5)));
    }

    #[test]
    fn test_nested_function_calls() {
        // def inner(): return 10
        // def outer(): return inner() + 5
        // result = outer()

        let instructions = vec![
            Instruction::DefineFunction {
                name_index: 0,
                param_count: 0,
                body_start: 5,
                body_len: 2,
                max_register_used: 0,
            },
            Instruction::DefineFunction {
                name_index: 1,
                param_count: 0,
                body_start: 7,
                body_len: 4,
                max_register_used: 1,
            },
            Instruction::Call {
                name_index: 1,
                arg_count: 0,
                first_arg_reg: 0,
                dest_reg: 5,
            },
            Instruction::SetResult { src_reg: 5 },
            Instruction::Halt,
            Instruction::LoadConst {
                dest_reg: 10,
                const_index: 0,
            },
            Instruction::Return {
                has_value: true,
                src_reg: Some(10),
            },
            Instruction::Call {
                name_index: 0,
                arg_count: 0,
                first_arg_reg: 0,
                dest_reg: 15,
            },
            Instruction::LoadConst {
                dest_reg: 16,
                const_index: 1,
            },
            Instruction::BinaryOp {
                dest_reg: 17,
                left_reg: 15,
                op: BinaryOperator::Add,
                right_reg: 16,
            },
            Instruction::Return {
                has_value: true,
                src_reg: Some(17),
            },
        ];

        let bytecode = Bytecode {
            instructions,
            constants: vec![10, 5],
            var_names: vec!["inner".to_string(), "outer".to_string()],
            var_ids: vec![1, 2],
            metadata: crate::bytecode::CompilerMetadata {
                max_register_used: 255,
            },
        };

        let mut vm = VM::new();
        let result = vm.execute(&bytecode).unwrap();

        assert_eq!(result, Some(Value::Integer(15)));
    }

    #[test]
    fn test_recursive_function_countdown() {
        // def countdown(n): return n (simplified version)
        // result = countdown(3)

        let instructions = vec![
            Instruction::DefineFunction {
                name_index: 0,
                param_count: 1,
                body_start: 5,
                body_len: 2,
                max_register_used: 1,
            },
            Instruction::LoadConst {
                dest_reg: 0,
                const_index: 0,
            },
            Instruction::Call {
                name_index: 0,
                arg_count: 1,
                first_arg_reg: 0,
                dest_reg: 5,
            },
            Instruction::SetResult { src_reg: 5 },
            Instruction::Halt,
            Instruction::LoadVar {
                dest_reg: 10,
                var_name_index: 1,
                var_id: 2,
            },
            Instruction::Return {
                has_value: true,
                src_reg: Some(10),
            },
        ];

        let bytecode = Bytecode {
            instructions,
            constants: vec![3],
            var_names: vec!["countdown".to_string(), "param_0".to_string()],
            var_ids: vec![1, 2],
            metadata: crate::bytecode::CompilerMetadata {
                max_register_used: 255,
            },
        };

        let mut vm = VM::new();
        let result = vm.execute(&bytecode).unwrap();

        assert_eq!(result, Some(Value::Integer(3)));
    }

    #[test]
    fn test_undefined_function_error() {
        let mut builder = BytecodeBuilder::new();
        // Call undefined function
        builder.emit_call("undefined", 1, 0, 0, 5);

        let bytecode = builder.build();

        let mut vm = VM::new();
        let result = vm.execute(&bytecode);

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.message.contains("Undefined function"));
        assert!(err.message.contains("undefined"));
    }

    #[test]
    fn test_wrong_argument_count_error() {
        let instructions = vec![
            Instruction::DefineFunction {
                name_index: 0,
                param_count: 2,
                body_start: 4,
                body_len: 1,
                max_register_used: 1,
            },
            Instruction::LoadConst {
                dest_reg: 0,
                const_index: 0,
            },
            Instruction::Call {
                name_index: 0,
                arg_count: 1,
                first_arg_reg: 0,
                dest_reg: 5,
            },
            Instruction::Halt,
            Instruction::Return {
                has_value: false,
                src_reg: None,
            },
        ];

        let bytecode = Bytecode {
            instructions,
            constants: vec![10],
            var_names: vec!["add".to_string()],
            var_ids: vec![1],
            metadata: crate::bytecode::CompilerMetadata {
                max_register_used: 255,
            },
        };

        let mut vm = VM::new();
        let result = vm.execute(&bytecode);

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.message.contains("expects 2 arguments"));
        assert!(err.message.contains("got 1"));
    }

    #[test]
    fn test_return_outside_function_error() {
        let mut builder = BytecodeBuilder::new();
        // Return at top level (not in a function)
        builder.emit_return(false, None);

        let bytecode = builder.build();

        let mut vm = VM::new();
        let result = vm.execute(&bytecode);

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.message, "Return outside of function");
    }

    #[test]
    fn test_function_can_access_global_variables() {
        // global_var = 100
        // def foo(): return global_var
        // result = foo()

        let instructions = vec![
            Instruction::LoadConst {
                dest_reg: 0,
                const_index: 0,
            },
            Instruction::StoreVar {
                var_name_index: 0,
                var_id: 1,
                src_reg: 0,
            },
            Instruction::DefineFunction {
                name_index: 1,
                param_count: 0,
                body_start: 6,
                body_len: 2,
                max_register_used: 0,
            },
            Instruction::Call {
                name_index: 1,
                arg_count: 0,
                first_arg_reg: 0,
                dest_reg: 5,
            },
            Instruction::SetResult { src_reg: 5 },
            Instruction::Halt,
            Instruction::LoadVar {
                dest_reg: 10,
                var_name_index: 0,
                var_id: 1,
            },
            Instruction::Return {
                has_value: true,
                src_reg: Some(10),
            },
        ];

        let bytecode = Bytecode {
            instructions,
            constants: vec![100],
            var_names: vec!["global_var".to_string(), "foo".to_string()],
            var_ids: vec![1, 2],
            metadata: crate::bytecode::CompilerMetadata {
                max_register_used: 255,
            },
        };

        let mut vm = VM::new();
        let result = vm.execute(&bytecode).unwrap();

        assert_eq!(result, Some(Value::Integer(100)));
    }

    #[test]
    fn test_local_variable_shadows_global() {
        // x = 5
        // def foo(): x = 10; return x
        // foo() returns 10, global x is still 5

        let instructions = vec![
            Instruction::LoadConst {
                dest_reg: 0,
                const_index: 0,
            },
            Instruction::StoreVar {
                var_name_index: 0,
                var_id: 1,
                src_reg: 0,
            },
            Instruction::DefineFunction {
                name_index: 1,
                param_count: 0,
                body_start: 5,
                body_len: 4,
                max_register_used: 2,
            },
            Instruction::Call {
                name_index: 1,
                arg_count: 0,
                first_arg_reg: 0,
                dest_reg: 5,
            },
            Instruction::Halt,
            Instruction::LoadConst {
                dest_reg: 10,
                const_index: 1,
            },
            Instruction::StoreVar {
                var_name_index: 0,
                var_id: 1,
                src_reg: 10,
            },
            Instruction::LoadVar {
                dest_reg: 11,
                var_name_index: 0,
                var_id: 1,
            },
            Instruction::Return {
                has_value: true,
                src_reg: Some(11),
            },
        ];

        let bytecode = Bytecode {
            instructions,
            constants: vec![5, 10],
            var_names: vec!["x".to_string(), "foo".to_string()],
            var_ids: vec![1, 2],
            metadata: crate::bytecode::CompilerMetadata {
                max_register_used: 255,
            },
        };

        let mut vm = VM::new();
        vm.execute(&bytecode).unwrap();

        // Global x unchanged
        assert_eq!(vm.variables.get(&1), Some(&Value::Integer(5)));
    }

    #[test]
    fn test_multiple_function_definitions() {
        let instructions = vec![
            Instruction::DefineFunction {
                name_index: 0,
                param_count: 0,
                body_start: 4,
                body_len: 1,
                max_register_used: 0,
            },
            Instruction::DefineFunction {
                name_index: 1,
                param_count: 1,
                body_start: 5,
                body_len: 1,
                max_register_used: 0,
            },
            Instruction::DefineFunction {
                name_index: 2,
                param_count: 2,
                body_start: 6,
                body_len: 1,
                max_register_used: 1,
            },
            Instruction::Halt,
            Instruction::Return {
                has_value: false,
                src_reg: None,
            },
            Instruction::Return {
                has_value: false,
                src_reg: None,
            },
            Instruction::Return {
                has_value: false,
                src_reg: None,
            },
        ];

        let bytecode = Bytecode {
            instructions,
            constants: vec![],
            var_names: vec![
                "func1".to_string(),
                "func2".to_string(),
                "func3".to_string(),
            ],
            var_ids: vec![1, 2, 3],
            metadata: crate::bytecode::CompilerMetadata {
                max_register_used: 255,
            },
        };

        let mut vm = VM::new();
        vm.execute(&bytecode).unwrap();

        assert_eq!(vm.functions.len(), 3);
        assert!(vm.functions.contains_key("func1"));
        assert!(vm.functions.contains_key("func2"));
        assert!(vm.functions.contains_key("func3"));
    }

    #[test]
    fn test_function_returns_arithmetic_result() {
        // def calc(): return 10 + 20 * 2
        // result = calc()

        let instructions = vec![
            Instruction::DefineFunction {
                name_index: 0,
                param_count: 0,
                body_start: 4,
                body_len: 6,
                max_register_used: 6,
            },
            Instruction::Call {
                name_index: 0,
                arg_count: 0,
                first_arg_reg: 0,
                dest_reg: 5,
            },
            Instruction::SetResult { src_reg: 5 },
            Instruction::Halt,
            Instruction::LoadConst {
                dest_reg: 10,
                const_index: 0,
            },
            Instruction::LoadConst {
                dest_reg: 11,
                const_index: 1,
            },
            Instruction::LoadConst {
                dest_reg: 12,
                const_index: 2,
            },
            Instruction::BinaryOp {
                dest_reg: 13,
                left_reg: 11,
                op: BinaryOperator::Mul,
                right_reg: 12,
            },
            Instruction::BinaryOp {
                dest_reg: 14,
                left_reg: 10,
                op: BinaryOperator::Add,
                right_reg: 13,
            },
            Instruction::Return {
                has_value: true,
                src_reg: Some(14),
            },
        ];

        let bytecode = Bytecode {
            instructions,
            constants: vec![10, 20, 2],
            var_names: vec!["calc".to_string()],
            var_ids: vec![1],
            metadata: crate::bytecode::CompilerMetadata {
                max_register_used: 255,
            },
        };

        let mut vm = VM::new();
        let result = vm.execute(&bytecode).unwrap();

        assert_eq!(result, Some(Value::Integer(50)));
    }

    #[test]
    fn test_function_with_print_statement() {
        // def greet(): print(42)
        // greet()

        let instructions = vec![
            Instruction::DefineFunction {
                name_index: 0,
                param_count: 0,
                body_start: 3,
                body_len: 3,
                max_register_used: 0,
            },
            Instruction::Call {
                name_index: 0,
                arg_count: 0,
                first_arg_reg: 0,
                dest_reg: 5,
            },
            Instruction::Halt,
            Instruction::LoadConst {
                dest_reg: 10,
                const_index: 0,
            },
            Instruction::Print { src_reg: 10 },
            Instruction::Return {
                has_value: false,
                src_reg: None,
            },
        ];

        let bytecode = Bytecode {
            instructions,
            constants: vec![42],
            var_names: vec!["greet".to_string()],
            var_ids: vec![1],
            metadata: crate::bytecode::CompilerMetadata {
                max_register_used: 255,
            },
        };

        let mut vm = VM::new();
        vm.execute(&bytecode).unwrap();

        assert_eq!(vm.stdout.as_str(), "42\n");
    }

    #[test]
    fn test_function_call_multiple_times() {
        // def get_ten(): return 10
        // a = get_ten()
        // b = get_ten()

        let instructions = vec![
            Instruction::DefineFunction {
                name_index: 0,
                param_count: 0,
                body_start: 6,
                body_len: 2,
                max_register_used: 1,
            },
            Instruction::Call {
                name_index: 0,
                arg_count: 0,
                first_arg_reg: 0,
                dest_reg: 5,
            },
            Instruction::StoreVar {
                var_name_index: 1,
                var_id: 2,
                src_reg: 5,
            },
            Instruction::Call {
                name_index: 0,
                arg_count: 0,
                first_arg_reg: 0,
                dest_reg: 6,
            },
            Instruction::StoreVar {
                var_name_index: 2,
                var_id: 3,
                src_reg: 6,
            },
            Instruction::Halt,
            Instruction::LoadConst {
                dest_reg: 10,
                const_index: 0,
            },
            Instruction::Return {
                has_value: true,
                src_reg: Some(10),
            },
        ];

        let bytecode = Bytecode {
            instructions,
            constants: vec![10],
            var_names: vec!["get_ten".to_string(), "a".to_string(), "b".to_string()],
            var_ids: vec![1, 2, 3],
            metadata: crate::bytecode::CompilerMetadata {
                max_register_used: 255,
            },
        };

        let mut vm = VM::new();
        vm.execute(&bytecode).unwrap();

        assert_eq!(vm.variables.get(&2), Some(&Value::Integer(10)));
        assert_eq!(vm.variables.get(&3), Some(&Value::Integer(10)));
    }

    #[test]
    fn test_function_parameter_names_dont_leak() {
        // def foo(x): return x
        // foo(42)
        // param_0 should not be in global scope

        let instructions = vec![
            Instruction::DefineFunction {
                name_index: 0,
                param_count: 1,
                body_start: 4,
                body_len: 2,
                max_register_used: 0,
            },
            Instruction::LoadConst {
                dest_reg: 0,
                const_index: 0,
            },
            Instruction::Call {
                name_index: 0,
                arg_count: 1,
                first_arg_reg: 0,
                dest_reg: 5,
            },
            Instruction::Halt,
            Instruction::LoadVar {
                dest_reg: 10,
                var_name_index: 1,
                var_id: 2,
            },
            Instruction::Return {
                has_value: true,
                src_reg: Some(10),
            },
        ];

        let bytecode = Bytecode {
            instructions,
            constants: vec![42],
            var_names: vec!["foo".to_string(), "param_0".to_string()],
            var_ids: vec![1, 2],
            metadata: crate::bytecode::CompilerMetadata {
                max_register_used: 255,
            },
        };

        let mut vm = VM::new();
        vm.execute(&bytecode).unwrap();

        // param_0 should not exist in global scope
        assert!(!vm.variables.contains_key(&2));
    }

    #[test]
    fn test_registers_restored_after_function_call() {
        // Set some registers, call function, verify registers restored

        let instructions = vec![
            Instruction::LoadConst {
                dest_reg: 0,
                const_index: 0,
            },
            Instruction::DefineFunction {
                name_index: 0,
                param_count: 0,
                body_start: 4,
                body_len: 2,
                max_register_used: 0,
            },
            Instruction::Call {
                name_index: 0,
                arg_count: 0,
                first_arg_reg: 0,
                dest_reg: 5,
            },
            Instruction::Halt,
            Instruction::LoadConst {
                dest_reg: 0,
                const_index: 1,
            },
            Instruction::Return {
                has_value: true,
                src_reg: Some(0),
            },
        ];

        let bytecode = Bytecode {
            instructions,
            constants: vec![999, 42],
            var_names: vec!["foo".to_string()],
            var_ids: vec![1],
            metadata: crate::bytecode::CompilerMetadata {
                max_register_used: 255,
            },
        };

        let mut vm = VM::new();
        vm.execute(&bytecode).unwrap();

        // Register 0 should be restored to 999
        assert_eq!(vm.registers[0], Value::Integer(999));
        // Register 5 should have the return value
        assert_eq!(vm.registers[5], Value::Integer(42));
    }

    #[test]
    fn test_function_with_three_parameters() {
        // def sum3(a, b, c): return a + b + c
        // result = sum3(10, 20, 30)

        let instructions = vec![
            Instruction::DefineFunction {
                name_index: 0,
                param_count: 3,
                body_start: 7,
                body_len: 6,
                max_register_used: 5,
            },
            Instruction::LoadConst {
                dest_reg: 0,
                const_index: 0,
            },
            Instruction::LoadConst {
                dest_reg: 1,
                const_index: 1,
            },
            Instruction::LoadConst {
                dest_reg: 2,
                const_index: 2,
            },
            Instruction::Call {
                name_index: 0,
                arg_count: 3,
                first_arg_reg: 0,
                dest_reg: 5,
            },
            Instruction::SetResult { src_reg: 5 },
            Instruction::Halt,
            Instruction::LoadVar {
                dest_reg: 10,
                var_name_index: 1,
                var_id: 2,
            },
            Instruction::LoadVar {
                dest_reg: 11,
                var_name_index: 2,
                var_id: 3,
            },
            Instruction::LoadVar {
                dest_reg: 12,
                var_name_index: 3,
                var_id: 4,
            },
            Instruction::BinaryOp {
                dest_reg: 13,
                left_reg: 10,
                op: BinaryOperator::Add,
                right_reg: 11,
            },
            Instruction::BinaryOp {
                dest_reg: 14,
                left_reg: 13,
                op: BinaryOperator::Add,
                right_reg: 12,
            },
            Instruction::Return {
                has_value: true,
                src_reg: Some(14),
            },
        ];

        let bytecode = Bytecode {
            instructions,
            constants: vec![10, 20, 30],
            var_names: vec![
                "sum3".to_string(),
                "param_0".to_string(),
                "param_1".to_string(),
                "param_2".to_string(),
            ],
            var_ids: vec![1, 2, 3, 4],
            metadata: crate::bytecode::CompilerMetadata {
                max_register_used: 255,
            },
        };

        let mut vm = VM::new();
        let result = vm.execute(&bytecode).unwrap();

        assert_eq!(result, Some(Value::Integer(60)));
    }

    #[test]
    fn test_empty_function_body_returns_none() {
        // def empty(): return
        // result = empty()

        let instructions = vec![
            Instruction::DefineFunction {
                name_index: 0,
                param_count: 0,
                body_start: 4,
                body_len: 1,
                max_register_used: 0,
            },
            Instruction::Call {
                name_index: 0,
                arg_count: 0,
                first_arg_reg: 0,
                dest_reg: 5,
            },
            Instruction::SetResult { src_reg: 5 },
            Instruction::Halt,
            Instruction::Return {
                has_value: false,
                src_reg: None,
            },
        ];

        let bytecode = Bytecode {
            instructions,
            constants: vec![],
            var_names: vec!["empty".to_string()],
            var_ids: vec![1],
            metadata: crate::bytecode::CompilerMetadata {
                max_register_used: 255,
            },
        };

        let mut vm = VM::new();
        let result = vm.execute(&bytecode).unwrap();

        assert_eq!(result, Some(Value::None));
    }

    #[test]
    fn test_function_result_used_in_expression() {
        // def get_five(): return 5
        // result = get_five() + 10

        let instructions = vec![
            Instruction::DefineFunction {
                name_index: 0,
                param_count: 0,
                body_start: 6,
                body_len: 2,
                max_register_used: 1,
            },
            Instruction::Call {
                name_index: 0,
                arg_count: 0,
                first_arg_reg: 0,
                dest_reg: 5,
            },
            Instruction::LoadConst {
                dest_reg: 6,
                const_index: 1,
            },
            Instruction::BinaryOp {
                dest_reg: 7,
                left_reg: 5,
                op: BinaryOperator::Add,
                right_reg: 6,
            },
            Instruction::SetResult { src_reg: 7 },
            Instruction::Halt,
            Instruction::LoadConst {
                dest_reg: 10,
                const_index: 0,
            },
            Instruction::Return {
                has_value: true,
                src_reg: Some(10),
            },
        ];

        let bytecode = Bytecode {
            instructions,
            constants: vec![5, 10],
            var_names: vec!["get_five".to_string()],
            var_ids: vec![1],
            metadata: crate::bytecode::CompilerMetadata {
                max_register_used: 255,
            },
        };

        let mut vm = VM::new();
        let result = vm.execute(&bytecode).unwrap();

        assert_eq!(result, Some(Value::Integer(15)));
    }

    #[test]
    fn test_function_redefine_overwrites() {
        // Define function twice, second definition should win

        let instructions = vec![
            Instruction::DefineFunction {
                name_index: 0,
                param_count: 0,
                body_start: 3,
                body_len: 1,
                max_register_used: 0,
            },
            Instruction::DefineFunction {
                name_index: 0,
                param_count: 1,
                body_start: 4,
                body_len: 1,
                max_register_used: 0,
            },
            Instruction::Halt,
            Instruction::Return {
                has_value: false,
                src_reg: None,
            },
            Instruction::Return {
                has_value: false,
                src_reg: None,
            },
        ];

        let bytecode = Bytecode {
            instructions,
            constants: vec![],
            var_names: vec!["foo".to_string()],
            var_ids: vec![1],
            metadata: crate::bytecode::CompilerMetadata {
                max_register_used: 255,
            },
        };

        let mut vm = VM::new();
        vm.execute(&bytecode).unwrap();

        let func = &vm.functions["foo"];
        assert_eq!(func.param_count, 1);
        assert_eq!(func.body_start, 4);
    }

    #[test]
    fn test_deeply_nested_calls() {
        // def f1(): return 1
        // def f2(): return f1() + 1
        // def f3(): return f2() + 1
        // result = f3()  # Should be 3

        let instructions = vec![
            Instruction::DefineFunction {
                name_index: 0,
                param_count: 0,
                body_start: 6,
                body_len: 2,
                max_register_used: 1,
            },
            Instruction::DefineFunction {
                name_index: 1,
                param_count: 0,
                body_start: 8,
                body_len: 4,
                max_register_used: 1,
            },
            Instruction::DefineFunction {
                name_index: 2,
                param_count: 0,
                body_start: 12,
                body_len: 4,
                max_register_used: 1,
            },
            Instruction::Call {
                name_index: 2,
                arg_count: 0,
                first_arg_reg: 0,
                dest_reg: 5,
            },
            Instruction::SetResult { src_reg: 5 },
            Instruction::Halt,
            Instruction::LoadConst {
                dest_reg: 10,
                const_index: 0,
            },
            Instruction::Return {
                has_value: true,
                src_reg: Some(10),
            },
            Instruction::Call {
                name_index: 0,
                arg_count: 0,
                first_arg_reg: 0,
                dest_reg: 15,
            },
            Instruction::LoadConst {
                dest_reg: 16,
                const_index: 0,
            },
            Instruction::BinaryOp {
                dest_reg: 17,
                left_reg: 15,
                op: BinaryOperator::Add,
                right_reg: 16,
            },
            Instruction::Return {
                has_value: true,
                src_reg: Some(17),
            },
            Instruction::Call {
                name_index: 1,
                arg_count: 0,
                first_arg_reg: 0,
                dest_reg: 20,
            },
            Instruction::LoadConst {
                dest_reg: 21,
                const_index: 0,
            },
            Instruction::BinaryOp {
                dest_reg: 22,
                left_reg: 20,
                op: BinaryOperator::Add,
                right_reg: 21,
            },
            Instruction::Return {
                has_value: true,
                src_reg: Some(22),
            },
        ];

        let bytecode = Bytecode {
            instructions,
            constants: vec![1],
            var_names: vec!["f1".to_string(), "f2".to_string(), "f3".to_string()],
            var_ids: vec![1, 2, 3],
            metadata: crate::bytecode::CompilerMetadata {
                max_register_used: 255,
            },
        };

        let mut vm = VM::new();
        let result = vm.execute(&bytecode).unwrap();

        assert_eq!(result, Some(Value::Integer(3)));
    }

    #[test]
    fn test_function_with_none_return_value() {
        // def empty(): return
        // x = empty()  # x should be Value::None
        // Testing that None value is properly stored

        let instructions = vec![
            Instruction::DefineFunction {
                name_index: 0,
                param_count: 0,
                body_start: 4,
                body_len: 1,
                max_register_used: 0,
            },
            Instruction::Call {
                name_index: 0,
                arg_count: 0,
                first_arg_reg: 0,
                dest_reg: 5,
            },
            Instruction::StoreVar {
                var_name_index: 1,
                var_id: 2,
                src_reg: 5,
            },
            Instruction::Halt,
            Instruction::Return {
                has_value: false,
                src_reg: None,
            },
        ];

        let bytecode = Bytecode {
            instructions,
            constants: vec![],
            var_names: vec!["empty".to_string(), "x".to_string()],
            var_ids: vec![1, 2],
            metadata: crate::bytecode::CompilerMetadata {
                max_register_used: 255,
            },
        };

        let mut vm = VM::new();
        vm.execute(&bytecode).unwrap();

        // Verify x holds None value
        assert_eq!(vm.variables.get(&2), Some(&Value::None));
    }

    #[test]
    fn test_multiple_function_calls_in_sequence() {
        // def get_one(): return 1
        // def get_two(): return 2
        // a = get_one()
        // b = get_two()
        // result = a + b

        let instructions = vec![
            Instruction::DefineFunction {
                name_index: 0,
                param_count: 0,
                body_start: 11,
                body_len: 2,
                max_register_used: 0,
            },
            Instruction::DefineFunction {
                name_index: 1,
                param_count: 0,
                body_start: 13,
                body_len: 2,
                max_register_used: 0,
            },
            Instruction::Call {
                name_index: 0,
                arg_count: 0,
                first_arg_reg: 0,
                dest_reg: 5,
            },
            Instruction::StoreVar {
                var_name_index: 2,
                var_id: 3,
                src_reg: 5,
            },
            Instruction::Call {
                name_index: 1,
                arg_count: 0,
                first_arg_reg: 0,
                dest_reg: 6,
            },
            Instruction::StoreVar {
                var_name_index: 3,
                var_id: 4,
                src_reg: 6,
            },
            Instruction::LoadVar {
                dest_reg: 7,
                var_name_index: 2,
                var_id: 3,
            },
            Instruction::LoadVar {
                dest_reg: 8,
                var_name_index: 3,
                var_id: 4,
            },
            Instruction::BinaryOp {
                dest_reg: 9,
                left_reg: 7,
                op: BinaryOperator::Add,
                right_reg: 8,
            },
            Instruction::SetResult { src_reg: 9 },
            Instruction::Halt,
            Instruction::LoadConst {
                dest_reg: 10,
                const_index: 0,
            },
            Instruction::Return {
                has_value: true,
                src_reg: Some(10),
            },
            Instruction::LoadConst {
                dest_reg: 11,
                const_index: 1,
            },
            Instruction::Return {
                has_value: true,
                src_reg: Some(11),
            },
        ];

        let bytecode = Bytecode {
            instructions,
            constants: vec![1, 2],
            var_names: vec![
                "get_one".to_string(),
                "get_two".to_string(),
                "a".to_string(),
                "b".to_string(),
            ],
            var_ids: vec![1, 2, 3, 4],
            metadata: crate::bytecode::CompilerMetadata {
                max_register_used: 255,
            },
        };

        let mut vm = VM::new();
        let result = vm.execute(&bytecode).unwrap();

        assert_eq!(result, Some(Value::Integer(3)));
        assert_eq!(vm.variables.get(&3), Some(&Value::Integer(1)));
        assert_eq!(vm.variables.get(&4), Some(&Value::Integer(2)));
    }

    #[test]
    fn test_function_using_arithmetic_on_parameters() {
        // def complex_calc(a, b, c): return (a + b) * c
        // result = complex_calc(2, 3, 4)  # Should be 20

        let instructions = vec![
            Instruction::DefineFunction {
                name_index: 0,
                param_count: 3,
                body_start: 7,
                body_len: 6,
                max_register_used: 5,
            },
            Instruction::LoadConst {
                dest_reg: 0,
                const_index: 0,
            },
            Instruction::LoadConst {
                dest_reg: 1,
                const_index: 1,
            },
            Instruction::LoadConst {
                dest_reg: 2,
                const_index: 2,
            },
            Instruction::Call {
                name_index: 0,
                arg_count: 3,
                first_arg_reg: 0,
                dest_reg: 5,
            },
            Instruction::SetResult { src_reg: 5 },
            Instruction::Halt,
            Instruction::LoadVar {
                dest_reg: 10,
                var_name_index: 1,
                var_id: 2,
            },
            Instruction::LoadVar {
                dest_reg: 11,
                var_name_index: 2,
                var_id: 3,
            },
            Instruction::BinaryOp {
                dest_reg: 12,
                left_reg: 10,
                op: BinaryOperator::Add,
                right_reg: 11,
            },
            Instruction::LoadVar {
                dest_reg: 13,
                var_name_index: 3,
                var_id: 4,
            },
            Instruction::BinaryOp {
                dest_reg: 14,
                left_reg: 12,
                op: BinaryOperator::Mul,
                right_reg: 13,
            },
            Instruction::Return {
                has_value: true,
                src_reg: Some(14),
            },
        ];

        let bytecode = Bytecode {
            instructions,
            constants: vec![2, 3, 4],
            var_names: vec![
                "complex_calc".to_string(),
                "param_0".to_string(),
                "param_1".to_string(),
                "param_2".to_string(),
            ],
            var_ids: vec![1, 2, 3, 4],
            metadata: crate::bytecode::CompilerMetadata {
                max_register_used: 255,
            },
        };

        let mut vm = VM::new();
        let result = vm.execute(&bytecode).unwrap();

        assert_eq!(result, Some(Value::Integer(20)));
    }

    #[test]
    fn test_function_with_negative_parameters() {
        // def subtract(a, b): return a - b
        // result = subtract(-10, -5)  # Should be -5

        let instructions = vec![
            Instruction::DefineFunction {
                name_index: 0,
                param_count: 2,
                body_start: 6,
                body_len: 4,
                max_register_used: 3,
            },
            Instruction::LoadConst {
                dest_reg: 0,
                const_index: 0,
            },
            Instruction::LoadConst {
                dest_reg: 1,
                const_index: 1,
            },
            Instruction::Call {
                name_index: 0,
                arg_count: 2,
                first_arg_reg: 0,
                dest_reg: 5,
            },
            Instruction::SetResult { src_reg: 5 },
            Instruction::Halt,
            Instruction::LoadVar {
                dest_reg: 10,
                var_name_index: 1,
                var_id: 2,
            },
            Instruction::LoadVar {
                dest_reg: 11,
                var_name_index: 2,
                var_id: 3,
            },
            Instruction::BinaryOp {
                dest_reg: 12,
                left_reg: 10,
                op: BinaryOperator::Sub,
                right_reg: 11,
            },
            Instruction::Return {
                has_value: true,
                src_reg: Some(12),
            },
        ];

        let bytecode = Bytecode {
            instructions,
            constants: vec![-10, -5],
            var_names: vec![
                "subtract".to_string(),
                "param_0".to_string(),
                "param_1".to_string(),
            ],
            var_ids: vec![1, 2, 3],
            metadata: crate::bytecode::CompilerMetadata {
                max_register_used: 255,
            },
        };

        let mut vm = VM::new();
        let result = vm.execute(&bytecode).unwrap();

        assert_eq!(result, Some(Value::Integer(-5)));
    }

    #[test]
    fn test_call_stack_depth() {
        // Test that call stack can handle reasonable depth
        // def level1(): return 10
        // def level2(): return level1() + 1
        // def level3(): return level2() + 1
        // def level4(): return level3() + 1
        // def level5(): return level4() + 1
        // result = level5()  # Should be 14

        let instructions = vec![
            Instruction::DefineFunction {
                name_index: 0,
                param_count: 0,
                body_start: 8,
                body_len: 2,
                max_register_used: 0,
            },
            Instruction::DefineFunction {
                name_index: 1,
                param_count: 0,
                body_start: 10,
                body_len: 4,
                max_register_used: 1,
            },
            Instruction::DefineFunction {
                name_index: 2,
                param_count: 0,
                body_start: 14,
                body_len: 4,
                max_register_used: 1,
            },
            Instruction::DefineFunction {
                name_index: 3,
                param_count: 0,
                body_start: 18,
                body_len: 4,
                max_register_used: 1,
            },
            Instruction::DefineFunction {
                name_index: 4,
                param_count: 0,
                body_start: 22,
                body_len: 4,
                max_register_used: 0,
            },
            Instruction::Call {
                name_index: 4,
                arg_count: 0,
                first_arg_reg: 0,
                dest_reg: 5,
            },
            Instruction::SetResult { src_reg: 5 },
            Instruction::Halt,
            Instruction::LoadConst {
                dest_reg: 10,
                const_index: 0,
            },
            Instruction::Return {
                has_value: true,
                src_reg: Some(10),
            },
            Instruction::Call {
                name_index: 0,
                arg_count: 0,
                first_arg_reg: 0,
                dest_reg: 15,
            },
            Instruction::LoadConst {
                dest_reg: 16,
                const_index: 1,
            },
            Instruction::BinaryOp {
                dest_reg: 17,
                left_reg: 15,
                op: BinaryOperator::Add,
                right_reg: 16,
            },
            Instruction::Return {
                has_value: true,
                src_reg: Some(17),
            },
            Instruction::Call {
                name_index: 1,
                arg_count: 0,
                first_arg_reg: 0,
                dest_reg: 20,
            },
            Instruction::LoadConst {
                dest_reg: 21,
                const_index: 1,
            },
            Instruction::BinaryOp {
                dest_reg: 22,
                left_reg: 20,
                op: BinaryOperator::Add,
                right_reg: 21,
            },
            Instruction::Return {
                has_value: true,
                src_reg: Some(22),
            },
            Instruction::Call {
                name_index: 2,
                arg_count: 0,
                first_arg_reg: 0,
                dest_reg: 25,
            },
            Instruction::LoadConst {
                dest_reg: 26,
                const_index: 1,
            },
            Instruction::BinaryOp {
                dest_reg: 27,
                left_reg: 25,
                op: BinaryOperator::Add,
                right_reg: 26,
            },
            Instruction::Return {
                has_value: true,
                src_reg: Some(27),
            },
            Instruction::Call {
                name_index: 3,
                arg_count: 0,
                first_arg_reg: 0,
                dest_reg: 30,
            },
            Instruction::LoadConst {
                dest_reg: 31,
                const_index: 1,
            },
            Instruction::BinaryOp {
                dest_reg: 32,
                left_reg: 30,
                op: BinaryOperator::Add,
                right_reg: 31,
            },
            Instruction::Return {
                has_value: true,
                src_reg: Some(32),
            },
        ];

        let bytecode = Bytecode {
            instructions,
            constants: vec![10, 1],
            var_names: vec![
                "level1".to_string(),
                "level2".to_string(),
                "level3".to_string(),
                "level4".to_string(),
                "level5".to_string(),
            ],
            var_ids: vec![1, 2, 3, 4, 5],
            metadata: crate::bytecode::CompilerMetadata {
                max_register_used: 255,
            },
        };

        let mut vm = VM::new();
        let result = vm.execute(&bytecode).unwrap();

        assert_eq!(result, Some(Value::Integer(14)));
    }

    #[test]
    fn test_function_with_zero_result_printed() {
        // def return_zero(): return 0
        // result = return_zero()
        // Testing that zero is properly distinguished from None

        let instructions = vec![
            Instruction::DefineFunction {
                name_index: 0,
                param_count: 0,
                body_start: 4,
                body_len: 2,
                max_register_used: 0,
            },
            Instruction::Call {
                name_index: 0,
                arg_count: 0,
                first_arg_reg: 0,
                dest_reg: 5,
            },
            Instruction::SetResult { src_reg: 5 },
            Instruction::Halt,
            Instruction::LoadConst {
                dest_reg: 10,
                const_index: 0,
            },
            Instruction::Return {
                has_value: true,
                src_reg: Some(10),
            },
        ];

        let bytecode = Bytecode {
            instructions,
            constants: vec![0],
            var_names: vec!["return_zero".to_string()],
            var_ids: vec![1],
            metadata: crate::bytecode::CompilerMetadata {
                max_register_used: 255,
            },
        };

        let mut vm = VM::new();
        let result = vm.execute(&bytecode).unwrap();

        assert_eq!(result, Some(Value::Integer(0)));
    }

    // ========== SmallString Tests ==========

    #[test]
    fn test_smallstring_inline_storage() {
        // Test that SmallString correctly handles inline storage for ≤23 bytes
        let mut s = SmallString::new();
        assert!(s.is_empty());
        assert_eq!(s.as_str(), "");

        // Add string that fits inline (3 bytes: "42\n")
        s.push_str("42\n");
        assert_eq!(s.as_str(), "42\n");
        assert!(!s.is_empty());

        // Verify it's still inline
        match s {
            SmallString::Inline { len, .. } => assert_eq!(len, 3),
            SmallString::Heap(_) => panic!("Should still be inline"),
        }
    }

    #[test]
    fn test_smallstring_heap_promotion() {
        // Test that SmallString promotes to heap when size exceeds 23 bytes
        let mut s = SmallString::new();

        // Add string that fits inline (20 bytes)
        s.push_str("12345678901234567890");
        match s {
            SmallString::Inline { len, .. } => assert_eq!(len, 20),
            SmallString::Heap(_) => panic!("Should be inline"),
        }

        // Add more to exceed 23 bytes total (20 + 4 = 24 bytes)
        s.push_str("abcd");
        match s {
            SmallString::Inline { .. } => panic!("Should have promoted to heap"),
            SmallString::Heap(ref string) => assert_eq!(string, "12345678901234567890abcd"),
        }

        assert_eq!(s.as_str(), "12345678901234567890abcd");
    }

    #[test]
    fn test_smallstring_exactly_23_bytes() {
        // Test boundary case: exactly 23 bytes should stay inline
        let mut s = SmallString::new();
        s.push_str("12345678901234567890123"); // exactly 23 bytes

        match s {
            SmallString::Inline { len, .. } => assert_eq!(len, 23),
            SmallString::Heap(_) => panic!("Should be inline at 23 bytes"),
        }

        assert_eq!(s.as_str(), "12345678901234567890123");
    }

    #[test]
    fn test_smallstring_promotion_at_24_bytes() {
        // Test boundary case: 24 bytes should promote to heap
        let mut s = SmallString::new();
        s.push_str("123456789012345678901234"); // 24 bytes

        match s {
            SmallString::Inline { .. } => panic!("Should have promoted to heap at 24 bytes"),
            SmallString::Heap(ref string) => assert_eq!(string, "123456789012345678901234"),
        }
    }

    #[test]
    fn test_smallstring_heap_append() {
        // Test that append works correctly after promotion to heap
        let mut s = SmallString::new();
        s.push_str("12345678901234567890"); // 20 bytes, inline
        s.push_str("abcd"); // 24 bytes total, promotes to heap
        s.push_str("xyz"); // 27 bytes, stays on heap

        match s {
            SmallString::Inline { .. } => panic!("Should be on heap"),
            SmallString::Heap(ref string) => assert_eq!(string, "12345678901234567890abcdxyz"),
        }

        assert_eq!(s.as_str(), "12345678901234567890abcdxyz");
    }

    #[test]
    fn test_smallstring_print_simple() {
        // Test realistic use case: print(42) = "42\n" = 3 bytes (inline)
        let mut vm = VM::new();
        let mut builder = BytecodeBuilder::new();
        builder.emit_load_const(0, 42);
        builder.emit_print(0);
        let bytecode = builder.build();

        vm.execute(&bytecode).unwrap();

        // Should be inline
        match vm.stdout {
            SmallString::Inline { len, .. } => assert_eq!(len, 3),
            SmallString::Heap(_) => panic!("Should be inline for simple print"),
        }

        assert_eq!(vm.stdout.as_str(), "42\n");
    }

    #[test]
    fn test_smallstring_multiple_small_prints() {
        // Test multiple small prints that fit inline
        let mut vm = VM::new();
        let mut builder = BytecodeBuilder::new();
        builder.emit_load_const(0, 1);
        builder.emit_print(0);
        builder.emit_load_const(1, 2);
        builder.emit_print(1);
        builder.emit_load_const(2, 3);
        builder.emit_print(2);
        let bytecode = builder.build();

        vm.execute(&bytecode).unwrap();

        // "1\n2\n3\n" = 6 bytes, should be inline
        match vm.stdout {
            SmallString::Inline { len, .. } => assert_eq!(len, 6),
            SmallString::Heap(_) => panic!("Should be inline"),
        }

        assert_eq!(vm.stdout.as_str(), "1\n2\n3\n");
    }

    #[test]
    fn test_smallstring_large_print() {
        // Test print that exceeds inline capacity
        let mut vm = VM::new();

        // Create a large number that when printed exceeds 23 bytes
        // We'll simulate printing a very large integer
        vm.stdout.push_str("123456789012345678901234567890\n"); // 32 bytes

        match vm.stdout {
            SmallString::Inline { .. } => panic!("Should be on heap for large print"),
            SmallString::Heap(_) => {} // Expected
        }

        assert_eq!(vm.stdout.as_str(), "123456789012345678901234567890\n");
    }

    #[test]
    fn test_smallstring_clone() {
        // Test that SmallString clone works for both variants
        let mut s1 = SmallString::new();
        s1.push_str("test");
        let s2 = s1.clone();
        assert_eq!(s1.as_str(), s2.as_str());

        let mut s3 = SmallString::new();
        s3.push_str("12345678901234567890abcd"); // Heap variant
        let s4 = s3.clone();
        assert_eq!(s3.as_str(), s4.as_str());
    }

    #[test]
    fn test_vm_stdout_format_output_inline() {
        // Test format_output uses as_str() correctly with inline storage
        let mut vm = VM::new();
        vm.stdout.push_str("test\n");

        let output = vm.format_output(None);
        assert_eq!(output, "test\n");
    }

    #[test]
    fn test_vm_stdout_format_output_heap() {
        // Test format_output uses as_str() correctly with heap storage
        let mut vm = VM::new();
        vm.stdout.push_str("12345678901234567890abcd\n"); // Exceeds 23 bytes

        let output = vm.format_output(None);
        assert_eq!(output, "12345678901234567890abcd\n");
    }

    #[test]
    fn test_smallstring_empty_string() {
        // Edge case: empty string should be inline with len=0
        let s = SmallString::new();
        assert_eq!(s.as_str(), "");
        assert!(s.is_empty());
        match s {
            SmallString::Inline { len, .. } => assert_eq!(len, 0),
            SmallString::Heap(_) => panic!("Empty string should be inline"),
        }
    }

    #[test]
    fn test_smallstring_single_byte() {
        // Edge case: single byte should be inline
        let mut s = SmallString::new();
        s.push_str("x");
        assert_eq!(s.as_str(), "x");
        match s {
            SmallString::Inline { len, .. } => assert_eq!(len, 1),
            SmallString::Heap(_) => panic!("Single byte should be inline"),
        }
    }

    #[test]
    fn test_smallstring_incremental_growth() {
        // Edge case: gradually adding characters should promote at correct boundary
        let mut s = SmallString::new();

        // Add 23 bytes one at a time
        for i in 0..23 {
            s.push_str("a");
            match &s {
                SmallString::Inline { len, .. } => assert_eq!(*len as usize, i + 1),
                SmallString::Heap(_) => panic!("Should still be inline at {} bytes", i + 1),
            }
        }

        // Adding one more byte should promote to heap
        s.push_str("b");
        match s {
            SmallString::Inline { .. } => panic!("Should have promoted to heap at 24 bytes"),
            SmallString::Heap(ref string) => assert_eq!(string.len(), 24),
        }
    }

    #[test]
    fn test_smallstring_unicode_handling() {
        // Edge case: UTF-8 multibyte characters
        let mut s = SmallString::new();
        s.push_str("Hello 世界"); // "Hello 世界" = 12 bytes (Hello=5, space=1, 世界=6)
        assert_eq!(s.as_str(), "Hello 世界");
        match s {
            SmallString::Inline { len, .. } => assert_eq!(len, 12),
            SmallString::Heap(_) => panic!("Should be inline"),
        }
    }

    #[test]
    fn test_smallstring_newline_characters() {
        // Edge case: newlines are counted correctly
        let mut s = SmallString::new();
        s.push_str("line1\nline2\nline3\n"); // 18 bytes
        assert_eq!(s.as_str(), "line1\nline2\nline3\n");
        match s {
            SmallString::Inline { len, .. } => assert_eq!(len, 18),
            SmallString::Heap(_) => panic!("Should be inline"),
        }
    }

    #[test]
    fn test_smallstring_repeated_promotions() {
        // Edge case: once promoted to heap, stays on heap even for small additions
        let mut s = SmallString::new();
        s.push_str("12345678901234567890abcd"); // 24 bytes, promotes to heap

        match &s {
            SmallString::Heap(_) => {}
            SmallString::Inline { .. } => panic!("Should be on heap"),
        }

        // Add more small strings - should stay on heap
        s.push_str("x");
        s.push_str("y");

        match s {
            SmallString::Heap(ref string) => assert_eq!(string, "12345678901234567890abcdxy"),
            SmallString::Inline { .. } => panic!("Should remain on heap"),
        }
    }

    #[test]
    fn test_vm_multiple_prints_boundary() {
        // Edge case: multiple prints that cumulatively exceed 23 bytes
        let mut vm = VM::new();
        let mut builder = BytecodeBuilder::new();

        // Each print is "N\n" = 2 bytes
        // 12 prints = 24 bytes total, should promote to heap on 12th print
        for i in 0..12 {
            builder.emit_load_const(i, i as i64);
            builder.emit_print(i);
        }

        let bytecode = builder.build();
        vm.execute(&bytecode).unwrap();

        // Should have promoted to heap at 24 bytes
        match vm.stdout {
            SmallString::Heap(_) => {}
            SmallString::Inline { .. } => panic!("Should have promoted to heap"),
        }

        assert_eq!(vm.stdout.as_str(), "0\n1\n2\n3\n4\n5\n6\n7\n8\n9\n10\n11\n");
    }

    #[test]
    fn test_register_validity_bitmap_all_words() {
        // Edge case: test bitmap operations across all 4 u64 words (registers 0-63, 64-127, 128-191, 192-255)
        let mut vm = VM::new();

        // Test one register from each u64 word
        let test_regs = [0u8, 63, 64, 127, 128, 191, 192, 255];

        for &reg in &test_regs {
            // Initially invalid
            assert!(
                !vm.is_register_valid(reg),
                "Register {} should be invalid initially",
                reg
            );

            // Set and verify
            vm.set_register(reg, Value::Integer(reg as i64));
            assert!(
                vm.is_register_valid(reg),
                "Register {} should be valid after set",
                reg
            );

            // Verify value
            let value = vm.get_register(reg).unwrap();
            assert_eq!(value, Value::Integer(reg as i64));
        }
    }

    #[test]
    fn test_register_validity_error_message() {
        // Edge case: accessing invalid register should give clear error with instruction pointer
        let vm = VM::new();

        let result = vm.get_register(42);
        assert!(result.is_err());

        let err = result.unwrap_err();
        assert_eq!(err.message, "Register 42 is empty");
        assert_eq!(err.instruction_index, 0); // IP is 0 initially
    }
}
