//! AST to Bytecode Compiler
//!
//! Single-pass compiler that transforms AST into register-based bytecode.
//! Implements register allocation and critical SetResult emission rules.

use crate::ast::{Expression, Program, Statement, UnaryOperator};
use crate::bytecode::{Bytecode, BytecodeBuilder};
use crate::error::CompileError;
use std::collections::{HashMap, HashSet};

#[cfg(test)]
use crate::ast::BinaryOperator;

/// Variable name interner for eliminating String allocations at runtime
pub struct VariableInterner {
    /// Map from variable name to interned ID
    name_to_id: HashMap<String, u32>,
    /// Map from interned ID back to variable name
    id_to_name: HashMap<u32, String>,
    /// Next available ID
    next_id: u32,
}

impl VariableInterner {
    /// Create a new interner with pre-populated common variable names
    pub fn new() -> Self {
        let mut interner = Self {
            name_to_id: HashMap::new(),
            id_to_name: HashMap::new(),
            next_id: 0,
        };

        // Pre-intern single-letter variables a-z
        for c in b'a'..=b'z' {
            let name = (c as char).to_string();
            interner.intern(&name);
        }

        // Pre-intern common variable names
        for name in &["result", "value", "temp", "count", "index", "data"] {
            interner.intern(name);
        }

        interner
    }

    /// Intern a variable name and return its ID
    pub fn intern(&mut self, name: &str) -> u32 {
        if let Some(&id) = self.name_to_id.get(name) {
            return id;
        }

        let id = self.next_id;
        self.next_id += 1;
        self.name_to_id.insert(name.to_string(), id);
        self.id_to_name.insert(id, name.to_string());
        id
    }

    /// Get the name for a given ID (for debugging/error messages)
    pub fn get_name(&self, id: u32) -> Option<&str> {
        self.id_to_name.get(&id).map(|s| s.as_str())
    }

    /// Get all interned names in ID order
    pub fn get_all_names(&self) -> Vec<String> {
        let mut names: Vec<(u32, String)> = self
            .id_to_name
            .iter()
            .map(|(&id, name)| (id, name.clone()))
            .collect();
        names.sort_by_key(|(id, _)| *id);
        names.into_iter().map(|(_, name)| name).collect()
    }
}

impl Default for VariableInterner {
    fn default() -> Self {
        Self::new()
    }
}

/// Compiler state with register allocation
pub struct Compiler {
    /// Bytecode builder for emitting instructions
    builder: BytecodeBuilder,
    /// Next available register number
    next_register: u8,
    /// Maximum register used so far
    max_register_used: u8,
    /// Track current instruction count
    instruction_counter: usize,
    /// Parameter name mapping: actual_name -> param_N (when compiling function bodies)
    param_mapping: HashMap<String, String>,
    /// Variable name interner
    interner: VariableInterner,
}

impl Compiler {
    /// Create a new compiler instance
    pub fn new() -> Self {
        Self {
            builder: BytecodeBuilder::new(),
            next_register: 0,
            max_register_used: 0,
            instruction_counter: 0,
            param_mapping: HashMap::new(),
            interner: VariableInterner::new(),
        }
    }

    /// Allocate a new register and return its number
    ///
    /// # Errors
    /// Returns CompileError if register limit (256) is exceeded
    fn alloc_register(&mut self) -> Result<u8, CompileError> {
        let reg = self.next_register;
        if reg == u8::MAX {
            return Err(CompileError {
                message: "Register limit exceeded (max 256 registers)".to_string(),
            });
        }
        self.next_register += 1;

        // Track maximum register used
        if reg > self.max_register_used {
            self.max_register_used = reg;
        }

        Ok(reg)
    }

    /// Increment instruction counter (called after each emit)
    fn inc_instruction_counter(&mut self) {
        self.instruction_counter += 1;
    }

    /// Compile a statement
    ///
    /// Implements critical SetResult emission rules:
    /// - Assignment: NO SetResult
    /// - Print: NO SetResult
    /// - Expression: YES SetResult
    ///
    /// Returns true if this was a function definition (to be handled separately)
    fn compile_statement(
        &mut self,
        stmt: &Statement,
        is_function_body: bool,
    ) -> Result<bool, CompileError> {
        match stmt {
            Statement::Assignment { name, value } => {
                // Compile the expression and get the register containing its result
                let value_reg = self.compile_expression(value)?;
                // Check if this is a parameter reference that needs mapping
                let actual_name = self.param_mapping.get(name).unwrap_or(name);
                // Intern the variable name
                let var_id = self.interner.intern(actual_name);
                // Store the value in the variable
                self.builder.emit_store_var(actual_name, var_id, value_reg);
                self.inc_instruction_counter();
                // CRITICAL: Assignment does NOT emit SetResult
                Ok(false)
            }
            Statement::Print { value } => {
                // Compile the expression and get the register containing its result
                let value_reg = self.compile_expression(value)?;
                // Emit print instruction
                self.builder.emit_print(value_reg);
                self.inc_instruction_counter();
                // CRITICAL: Print does NOT emit SetResult
                Ok(false)
            }
            Statement::Expression { value } => {
                // Compile the expression and get the register containing its result
                let value_reg = self.compile_expression(value)?;
                // CRITICAL: Expression statements DO emit SetResult
                self.builder.emit_set_result(value_reg);
                self.inc_instruction_counter();
                Ok(false)
            }
            Statement::FunctionDef {
                name: _,
                params: _,
                body: _,
            } => {
                if is_function_body {
                    // Nested function definitions not supported
                    return Err(CompileError {
                        message: "Nested function definitions are not supported".to_string(),
                    });
                }
                // Return true to indicate this is a function definition
                // It will be handled in compile_program
                Ok(true)
            }
            Statement::Return { value } => {
                if let Some(expr) = value {
                    // Compile the return value expression
                    let value_reg = self.compile_expression(expr)?;
                    // Emit return instruction with value
                    self.builder.emit_return(true, Some(value_reg));
                    self.inc_instruction_counter();
                } else {
                    // Emit return instruction without value
                    self.builder.emit_return(false, None);
                    self.inc_instruction_counter();
                }
                Ok(false)
            }
        }
    }

    /// Compile an expression and return the register containing its result
    fn compile_expression(&mut self, expr: &Expression) -> Result<u8, CompileError> {
        match expr {
            Expression::Integer(value) => {
                // Allocate a register for the constant
                let dest_reg = self.alloc_register()?;
                // Load the constant into the register
                self.builder.emit_load_const(dest_reg, *value);
                self.inc_instruction_counter();
                Ok(dest_reg)
            }
            Expression::Variable(name) => {
                // Allocate a register for the variable value
                let dest_reg = self.alloc_register()?;
                // Check if this is a parameter reference that needs mapping
                let actual_name = self.param_mapping.get(name).unwrap_or(name);
                // Intern the variable name
                let var_id = self.interner.intern(actual_name);
                // Load the variable into the register
                self.builder.emit_load_var(dest_reg, actual_name, var_id);
                self.inc_instruction_counter();
                Ok(dest_reg)
            }
            Expression::BinaryOp { left, op, right } => {
                // Compile left operand
                let left_reg = self.compile_expression(left)?;
                // Compile right operand
                let right_reg = self.compile_expression(right)?;
                // Allocate a register for the result
                let dest_reg = self.alloc_register()?;
                // Emit the binary operation
                self.builder
                    .emit_binary_op(dest_reg, left_reg, *op, right_reg);
                self.inc_instruction_counter();
                Ok(dest_reg)
            }
            Expression::UnaryOp { op, operand } => {
                // Compile the operand
                let operand_reg = self.compile_expression(operand)?;
                // Allocate a register for the result
                let dest_reg = self.alloc_register()?;
                // Emit the unary operation
                self.builder.emit_unary_op(dest_reg, *op, operand_reg);
                self.inc_instruction_counter();
                Ok(dest_reg)
            }
            Expression::Call { name, args } => {
                // Compile all arguments and collect their result registers
                // Arguments are evaluated left-to-right for register-based VM
                let mut arg_regs = Vec::new();
                for arg in args.iter() {
                    let arg_reg = self.compile_expression(arg)?;
                    arg_regs.push(arg_reg);
                }

                // Ensure arguments are in consecutive registers
                // If they're not, move them to consecutive registers
                let first_arg_reg = if arg_regs.is_empty() {
                    0 // No arguments, use 0 as placeholder
                } else {
                    // Check if registers are already consecutive
                    let are_consecutive = arg_regs.windows(2).all(|w| w[1] == w[0] + 1);

                    if are_consecutive {
                        // Already consecutive, use first register
                        arg_regs[0]
                    } else {
                        // Not consecutive, need to copy to consecutive registers
                        let first_consecutive_reg = self.next_register;

                        for (i, &arg_reg) in arg_regs.iter().enumerate() {
                            let target_reg = first_consecutive_reg + i as u8;

                            // Skip if already in correct position
                            if arg_reg != target_reg {
                                // Allocate the target register
                                let allocated_reg = self.alloc_register()?;
                                debug_assert_eq!(allocated_reg, target_reg);

                                // Copy using UnaryOp::Pos (identity operation)
                                self.builder
                                    .emit_unary_op(target_reg, UnaryOperator::Pos, arg_reg);
                                self.inc_instruction_counter();
                            } else {
                                // Register already in correct position, just mark it as allocated
                                self.alloc_register()?;
                            }
                        }

                        first_consecutive_reg
                    }
                };

                // Allocate a register for the return value
                let dest_reg = self.alloc_register()?;

                // Intern the function name
                let var_id = self.interner.intern(name);

                // Emit call instruction with argument register information
                self.builder
                    .emit_call(name, var_id, args.len() as u8, first_arg_reg, dest_reg);
                self.inc_instruction_counter();

                Ok(dest_reg)
            }
        }
    }

    /// Validate that a statement doesn't contain forward references to functions
    /// Forward reference: calling a function that will be defined later in the program
    fn validate_no_forward_references(
        stmt: &Statement,
        defined_so_far: &HashSet<String>,
        all_defined_functions: &HashSet<String>,
    ) -> Result<(), CompileError> {
        match stmt {
            Statement::Expression { value } | Statement::Assignment { value, .. } => {
                Self::check_expression_for_forward_references(
                    value,
                    defined_so_far,
                    all_defined_functions,
                )
            }
            Statement::Print { value } => Self::check_expression_for_forward_references(
                value,
                defined_so_far,
                all_defined_functions,
            ),
            Statement::Return { value } => {
                if let Some(expr) = value {
                    Self::check_expression_for_forward_references(
                        expr,
                        defined_so_far,
                        all_defined_functions,
                    )
                } else {
                    Ok(())
                }
            }
            Statement::FunctionDef { .. } => Ok(()),
        }
    }

    /// Check if an expression contains forward references to functions
    fn check_expression_for_forward_references(
        expr: &Expression,
        defined_so_far: &HashSet<String>,
        all_defined_functions: &HashSet<String>,
    ) -> Result<(), CompileError> {
        match expr {
            Expression::Call { name, args } => {
                // Check if this is a forward reference:
                // - The function will be defined later (in all_defined_functions)
                // - But is NOT yet defined (not in defined_so_far)
                if all_defined_functions.contains(name) && !defined_so_far.contains(name) {
                    return Err(CompileError {
                        message: format!(
                            "Call to undefined function '{}' (function defined later in program)",
                            name
                        ),
                    });
                }
                // Recursively check arguments
                for arg in args {
                    Self::check_expression_for_forward_references(
                        arg,
                        defined_so_far,
                        all_defined_functions,
                    )?;
                }
                Ok(())
            }
            Expression::BinaryOp { left, right, .. } => {
                Self::check_expression_for_forward_references(
                    left,
                    defined_so_far,
                    all_defined_functions,
                )?;
                Self::check_expression_for_forward_references(
                    right,
                    defined_so_far,
                    all_defined_functions,
                )
            }
            Expression::UnaryOp { operand, .. } => Self::check_expression_for_forward_references(
                operand,
                defined_so_far,
                all_defined_functions,
            ),
            Expression::Integer(_) | Expression::Variable(_) => Ok(()),
        }
    }

    /// Compile a program and return the bytecode
    fn compile_program(mut self, program: &Program) -> Result<Bytecode, CompileError> {
        // First pass: collect all function names that will be defined
        let all_defined_functions: HashSet<String> = program
            .statements
            .iter()
            .filter_map(|stmt| {
                if let Statement::FunctionDef { name, .. } = stmt {
                    Some(name.clone())
                } else {
                    None
                }
            })
            .collect();

        // Separate function definitions from main code
        let mut function_defs = Vec::new();
        let mut main_statements = Vec::new();
        let mut defined_so_far = HashSet::new();

        // Process statements in order to detect forward references
        for stmt in &program.statements {
            if let Statement::FunctionDef { name, body, .. } = stmt {
                // Add function to defined set BEFORE validating body (allows recursion)
                defined_so_far.insert(name.clone());
                // Validate function body doesn't contain forward references
                for body_stmt in body {
                    Self::validate_no_forward_references(
                        body_stmt,
                        &defined_so_far,
                        &all_defined_functions,
                    )?;
                }
                function_defs.push(stmt);
            } else {
                // Validate that any function calls don't reference functions defined later
                Self::validate_no_forward_references(
                    stmt,
                    &defined_so_far,
                    &all_defined_functions,
                )?;
                main_statements.push(stmt);
            }
        }

        // Pass 1: Compile function bodies and track their locations
        // We need to know where function bodies will be AFTER main code + Halt
        // Calculate offset: DefineFunction instructions + main code + Halt
        let define_func_count = function_defs.len();

        // We'll emit DefineFunction instructions first, then main code, then Halt, then function bodies
        // So function bodies start at: define_func_count + main_code_length + 1 (for Halt)

        // First, we need to compile main code to know its length
        let saved_counter = self.instruction_counter;

        // Temporarily compile main code to measure length
        for stmt in &main_statements {
            self.compile_statement(stmt, false)?;
        }
        let main_code_length = self.instruction_counter - saved_counter;

        // Reset compiler state
        self.instruction_counter = 0;
        self.next_register = 0;
        self.builder = BytecodeBuilder::new();

        // Calculate where function bodies will start
        let function_bodies_start = define_func_count + main_code_length + 1; // +1 for Halt

        // Pass 2: Compile function bodies and emit DefineFunction instructions
        let mut function_metadata = Vec::new();
        let mut current_body_offset = function_bodies_start;

        for func_def in &function_defs {
            if let Statement::FunctionDef { name, params, body } = func_def {
                // Save compiler state
                let saved_reg = self.next_register;
                let saved_param_mapping = self.param_mapping.clone();
                let saved_max_reg = self.max_register_used;

                // Set instruction counter to where this function body will be
                let body_start = current_body_offset;
                self.instruction_counter = body_start;

                // Reset register allocation for function scope
                self.next_register = params.len() as u8;

                // Reset max_register_used for this function
                self.max_register_used = if !params.is_empty() {
                    params.len() as u8 - 1
                } else {
                    0
                };

                // Set up parameter mapping: param_name -> param_N
                self.param_mapping.clear();
                for (i, param_name) in params.iter().enumerate() {
                    self.param_mapping
                        .insert(param_name.clone(), format!("param_{}", i));
                }

                // Ensure all parameter names are interned in bytecode
                // even if they're not used in the function body
                for i in 0..params.len() {
                    let param_name = format!("param_{}", i);
                    let var_id = self.interner.intern(&param_name);
                    // Add to var_names pool (will be deduplicated if already exists)
                    self.builder.ensure_var_name(&param_name, var_id);
                }

                // Compile function body
                for stmt in body {
                    self.compile_statement(stmt, true)?;
                }

                // Calculate body length
                let body_len = self.instruction_counter - body_start;

                // Store metadata for later (including per-function max_register_used)
                function_metadata.push((
                    name.clone(),
                    params.len() as u8,
                    body_start,
                    body_len,
                    self.max_register_used,
                ));

                // Update offset for next function
                current_body_offset = self.instruction_counter;

                // Restore compiler state
                self.next_register = saved_reg;
                self.param_mapping = saved_param_mapping;
                self.max_register_used = saved_max_reg;
            }
        }

        // Now we need to rebuild bytecode in correct order:
        // 1. DefineFunction instructions
        // 2. Main code
        // 3. Halt (added by builder)
        // 4. Function bodies

        // Get the function body instructions we just compiled
        let function_body_instructions = self.builder.instructions().to_vec();

        // Save the constant and variable name pools from function compilation
        let (constants, var_names, var_ids) = self.builder.get_pools();
        let saved_constants = constants.clone();
        let saved_var_names = var_names.clone();
        let saved_var_ids = var_ids.clone();

        // Reset builder with saved pools and instruction counter
        self.builder = BytecodeBuilder::with_pools(saved_constants, saved_var_names, saved_var_ids);
        self.instruction_counter = 0;
        self.next_register = 0;

        // Emit DefineFunction instructions first
        for (name, param_count, body_start, body_len, max_reg_used) in &function_metadata {
            let var_id = self.interner.intern(name);
            self.builder.emit_define_function(
                name,
                var_id,
                *param_count,
                *body_start,
                *body_len,
                *max_reg_used,
            );
            self.inc_instruction_counter();
        }

        // Compile main code
        for stmt in &main_statements {
            self.compile_statement(stmt, false)?;
        }

        // Build bytecode (this adds Halt)
        let mut bytecode = self.builder.build();

        // Append function body instructions
        bytecode.instructions.extend(function_body_instructions);

        // Set the max_register_used in metadata
        bytecode.metadata.max_register_used = self.max_register_used;

        Ok(bytecode)
    }
}

impl Default for Compiler {
    fn default() -> Self {
        Self::new()
    }
}

/// Compile an AST Program into Bytecode
///
/// This is the main entry point for the compiler.
/// Performs single-pass compilation with register allocation.
///
/// # Arguments
/// * `program` - The AST program to compile
///
/// # Returns
/// * `Ok(Bytecode)` - The compiled bytecode
/// * `Err(CompileError)` - If compilation fails
///
/// # Examples
/// ```
/// use pyrust::ast::{Program, Statement, Expression};
/// use pyrust::compiler::compile;
///
/// let program = Program {
///     statements: vec![
///         Statement::Expression {
///             value: Expression::Integer(42),
///         },
///     ],
/// };
///
/// let bytecode = compile(&program).unwrap();
/// ```
pub fn compile(program: &Program) -> Result<Bytecode, CompileError> {
    let compiler = Compiler::new();
    compiler.compile_program(program)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bytecode::Instruction;

    #[test]
    fn test_compile_integer_literal() {
        let program = Program {
            statements: vec![Statement::Expression {
                value: Expression::Integer(42),
            }],
        };

        let bytecode = compile(&program).unwrap();

        // Should have: LoadConst, SetResult, Halt
        assert_eq!(bytecode.instructions.len(), 3);
        assert_eq!(
            bytecode.instructions[0],
            Instruction::LoadConst {
                dest_reg: 0,
                const_index: 0
            }
        );
        assert_eq!(
            bytecode.instructions[1],
            Instruction::SetResult { src_reg: 0 }
        );
        assert_eq!(bytecode.instructions[2], Instruction::Halt);

        // Check constant pool
        assert_eq!(bytecode.constants.len(), 1);
        assert_eq!(bytecode.constants[0], 42);
    }

    #[test]
    fn test_compile_assignment_no_setresult() {
        let program = Program {
            statements: vec![Statement::Assignment {
                name: "x".to_string(),
                value: Expression::Integer(10),
            }],
        };

        let bytecode = compile(&program).unwrap();

        // Should have: LoadConst, StoreVar, Halt
        // CRITICAL: NO SetResult for assignment
        assert_eq!(bytecode.instructions.len(), 3);
        assert_eq!(
            bytecode.instructions[0],
            Instruction::LoadConst {
                dest_reg: 0,
                const_index: 0
            }
        );
        assert!(matches!(
            bytecode.instructions[1],
            Instruction::StoreVar {
                var_name_index: 0,
                var_id: _,
                src_reg: 0
            }
        ));
        assert_eq!(bytecode.instructions[2], Instruction::Halt);
    }

    #[test]
    fn test_compile_print_no_setresult() {
        let program = Program {
            statements: vec![Statement::Print {
                value: Expression::Integer(42),
            }],
        };

        let bytecode = compile(&program).unwrap();

        // Should have: LoadConst, Print, Halt
        // CRITICAL: NO SetResult for print
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
    }

    #[test]
    fn test_compile_expression_statement_has_setresult() {
        let program = Program {
            statements: vec![Statement::Expression {
                value: Expression::Variable("x".to_string()),
            }],
        };

        let bytecode = compile(&program).unwrap();

        // Should have: LoadVar, SetResult, Halt
        // CRITICAL: Expression statements DO emit SetResult
        assert_eq!(bytecode.instructions.len(), 3);
        assert!(matches!(
            bytecode.instructions[0],
            Instruction::LoadVar {
                dest_reg: 0,
                var_name_index: 0,
                var_id: _
            }
        ));
        assert_eq!(
            bytecode.instructions[1],
            Instruction::SetResult { src_reg: 0 }
        );
        assert_eq!(bytecode.instructions[2], Instruction::Halt);
    }

    #[test]
    fn test_compile_binary_operation() {
        // Test: 1 + 2
        let program = Program {
            statements: vec![Statement::Expression {
                value: Expression::BinaryOp {
                    left: Box::new(Expression::Integer(1)),
                    op: BinaryOperator::Add,
                    right: Box::new(Expression::Integer(2)),
                },
            }],
        };

        let bytecode = compile(&program).unwrap();

        // Should have: LoadConst(1), LoadConst(2), BinaryOp, SetResult, Halt
        assert_eq!(bytecode.instructions.len(), 5);
        assert!(matches!(
            bytecode.instructions[0],
            Instruction::LoadConst { dest_reg: 0, .. }
        ));
        assert!(matches!(
            bytecode.instructions[1],
            Instruction::LoadConst { dest_reg: 1, .. }
        ));
        assert_eq!(
            bytecode.instructions[2],
            Instruction::BinaryOp {
                dest_reg: 2,
                left_reg: 0,
                op: BinaryOperator::Add,
                right_reg: 1
            }
        );
        assert_eq!(
            bytecode.instructions[3],
            Instruction::SetResult { src_reg: 2 }
        );
        assert_eq!(bytecode.instructions[4], Instruction::Halt);
    }

    #[test]
    fn test_compile_all_binary_operators() {
        let operators = vec![
            BinaryOperator::Add,
            BinaryOperator::Sub,
            BinaryOperator::Mul,
            BinaryOperator::Div,
            BinaryOperator::FloorDiv,
            BinaryOperator::Mod,
        ];

        for op in operators {
            let program = Program {
                statements: vec![Statement::Expression {
                    value: Expression::BinaryOp {
                        left: Box::new(Expression::Integer(10)),
                        op,
                        right: Box::new(Expression::Integer(5)),
                    },
                }],
            };

            let bytecode = compile(&program).unwrap();

            // Verify BinaryOp instruction is present with correct operator
            if let Instruction::BinaryOp {
                op: compiled_op, ..
            } = bytecode.instructions[2]
            {
                assert_eq!(compiled_op, op);
            } else {
                panic!("Expected BinaryOp instruction");
            }
        }
    }

    #[test]
    fn test_compile_unary_operation() {
        // Test: -42
        let program = Program {
            statements: vec![Statement::Expression {
                value: Expression::UnaryOp {
                    op: UnaryOperator::Neg,
                    operand: Box::new(Expression::Integer(42)),
                },
            }],
        };

        let bytecode = compile(&program).unwrap();

        // Should have: LoadConst, UnaryOp, SetResult, Halt
        assert_eq!(bytecode.instructions.len(), 4);
        assert!(matches!(
            bytecode.instructions[0],
            Instruction::LoadConst { dest_reg: 0, .. }
        ));
        assert_eq!(
            bytecode.instructions[1],
            Instruction::UnaryOp {
                dest_reg: 1,
                op: UnaryOperator::Neg,
                operand_reg: 0
            }
        );
        assert_eq!(
            bytecode.instructions[2],
            Instruction::SetResult { src_reg: 1 }
        );
        assert_eq!(bytecode.instructions[3], Instruction::Halt);
    }

    #[test]
    fn test_compile_nested_expression() {
        // Test: (1 + 2) * 3
        let program = Program {
            statements: vec![Statement::Expression {
                value: Expression::BinaryOp {
                    left: Box::new(Expression::BinaryOp {
                        left: Box::new(Expression::Integer(1)),
                        op: BinaryOperator::Add,
                        right: Box::new(Expression::Integer(2)),
                    }),
                    op: BinaryOperator::Mul,
                    right: Box::new(Expression::Integer(3)),
                },
            }],
        };

        let bytecode = compile(&program).unwrap();

        // LoadConst(1), LoadConst(2), BinaryOp(Add), LoadConst(3), BinaryOp(Mul), SetResult, Halt
        assert_eq!(bytecode.instructions.len(), 7);

        // Verify the structure
        assert!(matches!(
            bytecode.instructions[0],
            Instruction::LoadConst { dest_reg: 0, .. }
        ));
        assert!(matches!(
            bytecode.instructions[1],
            Instruction::LoadConst { dest_reg: 1, .. }
        ));
        assert!(matches!(
            bytecode.instructions[2],
            Instruction::BinaryOp {
                dest_reg: 2,
                left_reg: 0,
                op: BinaryOperator::Add,
                right_reg: 1
            }
        ));
        assert!(matches!(
            bytecode.instructions[3],
            Instruction::LoadConst { dest_reg: 3, .. }
        ));
        assert!(matches!(
            bytecode.instructions[4],
            Instruction::BinaryOp {
                dest_reg: 4,
                left_reg: 2,
                op: BinaryOperator::Mul,
                right_reg: 3
            }
        ));
        assert_eq!(
            bytecode.instructions[5],
            Instruction::SetResult { src_reg: 4 }
        );
    }

    #[test]
    fn test_compile_variable_reference() {
        let program = Program {
            statements: vec![Statement::Expression {
                value: Expression::Variable("x".to_string()),
            }],
        };

        let bytecode = compile(&program).unwrap();

        // Should have: LoadVar, SetResult, Halt
        assert_eq!(bytecode.instructions.len(), 3);
        assert!(matches!(
            bytecode.instructions[0],
            Instruction::LoadVar {
                dest_reg: 0,
                var_name_index: 0,
                var_id: _
            }
        ));
        assert_eq!(bytecode.var_names.len(), 1);
        assert_eq!(bytecode.var_names[0], "x");
    }

    #[test]
    fn test_compile_complex_program() {
        // Test: x = 10; y = x + 5; print(y)
        let program = Program {
            statements: vec![
                Statement::Assignment {
                    name: "x".to_string(),
                    value: Expression::Integer(10),
                },
                Statement::Assignment {
                    name: "y".to_string(),
                    value: Expression::BinaryOp {
                        left: Box::new(Expression::Variable("x".to_string())),
                        op: BinaryOperator::Add,
                        right: Box::new(Expression::Integer(5)),
                    },
                },
                Statement::Print {
                    value: Expression::Variable("y".to_string()),
                },
            ],
        };

        let bytecode = compile(&program).unwrap();

        // Verify variable names pool
        assert_eq!(bytecode.var_names.len(), 2);
        assert!(bytecode.var_names.contains(&"x".to_string()));
        assert!(bytecode.var_names.contains(&"y".to_string()));

        // Verify constants pool
        assert_eq!(bytecode.constants.len(), 2);
        assert!(bytecode.constants.contains(&10));
        assert!(bytecode.constants.contains(&5));

        // Verify no SetResult for assignments and print
        for instr in &bytecode.instructions {
            if matches!(instr, Instruction::SetResult { .. }) {
                panic!("Unexpected SetResult in assignment/print statements");
            }
        }
    }

    #[test]
    fn test_register_allocation_sequential() {
        // Test: 1 + 2 + 3 + 4
        let program = Program {
            statements: vec![Statement::Expression {
                value: Expression::BinaryOp {
                    left: Box::new(Expression::BinaryOp {
                        left: Box::new(Expression::BinaryOp {
                            left: Box::new(Expression::Integer(1)),
                            op: BinaryOperator::Add,
                            right: Box::new(Expression::Integer(2)),
                        }),
                        op: BinaryOperator::Add,
                        right: Box::new(Expression::Integer(3)),
                    }),
                    op: BinaryOperator::Add,
                    right: Box::new(Expression::Integer(4)),
                },
            }],
        };

        let bytecode = compile(&program).unwrap();

        // Verify that registers are allocated sequentially
        // Each expression allocates a new register
        let mut max_reg = 0;
        for instr in &bytecode.instructions {
            match instr {
                Instruction::LoadConst { dest_reg, .. } => {
                    max_reg = max_reg.max(*dest_reg);
                }
                Instruction::BinaryOp { dest_reg, .. } => {
                    max_reg = max_reg.max(*dest_reg);
                }
                _ => {}
            }
        }
        // Should use multiple registers
        assert!(max_reg > 0);
    }

    #[test]
    fn test_constant_pool_deduplication() {
        // Test: 42 + 42
        let program = Program {
            statements: vec![Statement::Expression {
                value: Expression::BinaryOp {
                    left: Box::new(Expression::Integer(42)),
                    op: BinaryOperator::Add,
                    right: Box::new(Expression::Integer(42)),
                },
            }],
        };

        let bytecode = compile(&program).unwrap();

        // Constant 42 should only appear once in the pool
        assert_eq!(bytecode.constants.len(), 1);
        assert_eq!(bytecode.constants[0], 42);
    }

    #[test]
    fn test_variable_name_deduplication() {
        // Test: x + x
        let program = Program {
            statements: vec![Statement::Expression {
                value: Expression::BinaryOp {
                    left: Box::new(Expression::Variable("x".to_string())),
                    op: BinaryOperator::Add,
                    right: Box::new(Expression::Variable("x".to_string())),
                },
            }],
        };

        let bytecode = compile(&program).unwrap();

        // Variable name "x" should only appear once in the pool
        assert_eq!(bytecode.var_names.len(), 1);
        assert_eq!(bytecode.var_names[0], "x");
    }

    #[test]
    fn test_empty_program() {
        let program = Program { statements: vec![] };

        let bytecode = compile(&program).unwrap();

        // Should only have Halt instruction
        assert_eq!(bytecode.instructions.len(), 1);
        assert_eq!(bytecode.instructions[0], Instruction::Halt);
    }

    #[test]
    fn test_multiple_expression_statements() {
        // Test: 1; 2; 3;
        let program = Program {
            statements: vec![
                Statement::Expression {
                    value: Expression::Integer(1),
                },
                Statement::Expression {
                    value: Expression::Integer(2),
                },
                Statement::Expression {
                    value: Expression::Integer(3),
                },
            ],
        };

        let bytecode = compile(&program).unwrap();

        // Each expression should have SetResult
        let mut setresult_count = 0;
        for instr in &bytecode.instructions {
            if matches!(instr, Instruction::SetResult { .. }) {
                setresult_count += 1;
            }
        }
        assert_eq!(setresult_count, 3);
    }

    #[test]
    fn test_mixed_statement_types() {
        // Test: x = 5; print(x); x
        let program = Program {
            statements: vec![
                Statement::Assignment {
                    name: "x".to_string(),
                    value: Expression::Integer(5),
                },
                Statement::Print {
                    value: Expression::Variable("x".to_string()),
                },
                Statement::Expression {
                    value: Expression::Variable("x".to_string()),
                },
            ],
        };

        let bytecode = compile(&program).unwrap();

        // Only the expression statement should have SetResult
        let mut setresult_count = 0;
        for instr in &bytecode.instructions {
            if matches!(instr, Instruction::SetResult { .. }) {
                setresult_count += 1;
            }
        }
        assert_eq!(
            setresult_count, 1,
            "Only expression statement should emit SetResult"
        );
    }

    #[test]
    fn test_compiler_default() {
        let compiler = Compiler::default();
        assert_eq!(compiler.next_register, 0);
    }

    #[test]
    fn test_all_unary_operators() {
        let operators = vec![UnaryOperator::Neg, UnaryOperator::Pos];

        for op in operators {
            let program = Program {
                statements: vec![Statement::Expression {
                    value: Expression::UnaryOp {
                        op,
                        operand: Box::new(Expression::Integer(42)),
                    },
                }],
            };

            let bytecode = compile(&program).unwrap();

            // Verify UnaryOp instruction is present with correct operator
            if let Instruction::UnaryOp {
                op: compiled_op, ..
            } = bytecode.instructions[1]
            {
                assert_eq!(compiled_op, op);
            } else {
                panic!("Expected UnaryOp instruction");
            }
        }
    }

    #[test]
    fn test_deeply_nested_expression() {
        // Test: ((1 + 2) * (3 - 4)) / 5
        let program = Program {
            statements: vec![Statement::Expression {
                value: Expression::BinaryOp {
                    left: Box::new(Expression::BinaryOp {
                        left: Box::new(Expression::BinaryOp {
                            left: Box::new(Expression::Integer(1)),
                            op: BinaryOperator::Add,
                            right: Box::new(Expression::Integer(2)),
                        }),
                        op: BinaryOperator::Mul,
                        right: Box::new(Expression::BinaryOp {
                            left: Box::new(Expression::Integer(3)),
                            op: BinaryOperator::Sub,
                            right: Box::new(Expression::Integer(4)),
                        }),
                    }),
                    op: BinaryOperator::Div,
                    right: Box::new(Expression::Integer(5)),
                },
            }],
        };

        let bytecode = compile(&program).unwrap();

        // Should compile successfully with correct structure
        // Verify we have 5 constants
        assert_eq!(bytecode.constants.len(), 5);

        // Verify we have SetResult for the expression statement
        assert!(bytecode
            .instructions
            .iter()
            .any(|i| matches!(i, Instruction::SetResult { .. })));

        // Verify we have multiple BinaryOp instructions
        let binop_count = bytecode
            .instructions
            .iter()
            .filter(|i| matches!(i, Instruction::BinaryOp { .. }))
            .count();
        assert_eq!(binop_count, 4); // 4 binary operations
    }

    // ========== Function Compilation Tests ==========

    #[test]
    fn test_compile_function_def_no_params() {
        // Test: def foo(): return 42
        let program = Program {
            statements: vec![Statement::FunctionDef {
                name: "foo".to_string(),
                params: vec![],
                body: vec![Statement::Return {
                    value: Some(Expression::Integer(42)),
                }],
            }],
        };

        let bytecode = compile(&program).unwrap();

        // Should have: LoadConst(42), Return, DefineFunction, Halt
        assert!(bytecode
            .instructions
            .iter()
            .any(|i| matches!(i, Instruction::DefineFunction { .. })));
        assert!(bytecode
            .instructions
            .iter()
            .any(|i| matches!(i, Instruction::Return { .. })));

        // Check DefineFunction metadata
        let define_func = bytecode
            .instructions
            .iter()
            .find(|i| matches!(i, Instruction::DefineFunction { .. }))
            .unwrap();

        if let Instruction::DefineFunction {
            param_count,
            name_index,
            ..
        } = define_func
        {
            assert_eq!(*param_count, 0);
            assert_eq!(bytecode.var_names[*name_index], "foo");
        } else {
            panic!("Expected DefineFunction instruction");
        }
    }

    #[test]
    fn test_compile_function_def_with_params() {
        // Test: def add(a, b): return a + b
        let program = Program {
            statements: vec![Statement::FunctionDef {
                name: "add".to_string(),
                params: vec!["a".to_string(), "b".to_string()],
                body: vec![Statement::Return {
                    value: Some(Expression::BinaryOp {
                        left: Box::new(Expression::Variable("a".to_string())),
                        op: BinaryOperator::Add,
                        right: Box::new(Expression::Variable("b".to_string())),
                    }),
                }],
            }],
        };

        let bytecode = compile(&program).unwrap();

        // Verify DefineFunction instruction exists with correct param_count
        let define_func = bytecode
            .instructions
            .iter()
            .find(|i| matches!(i, Instruction::DefineFunction { .. }))
            .unwrap();

        if let Instruction::DefineFunction {
            param_count,
            name_index,
            ..
        } = define_func
        {
            assert_eq!(*param_count, 2);
            assert_eq!(bytecode.var_names[*name_index], "add");
        }

        // Verify function body compiled correctly
        assert!(bytecode.instructions.iter().any(|i| matches!(
            i,
            Instruction::Return {
                has_value: true,
                ..
            }
        )));
    }

    #[test]
    fn test_compile_function_call_no_args() {
        // Test: foo()
        let program = Program {
            statements: vec![Statement::Expression {
                value: Expression::Call {
                    name: "foo".to_string(),
                    args: vec![],
                },
            }],
        };

        let bytecode = compile(&program).unwrap();

        // Should have: Call, SetResult, Halt
        assert!(bytecode
            .instructions
            .iter()
            .any(|i| matches!(i, Instruction::Call { .. })));

        // Check Call instruction
        let call_instr = bytecode
            .instructions
            .iter()
            .find(|i| matches!(i, Instruction::Call { .. }))
            .unwrap();

        if let Instruction::Call {
            arg_count,
            name_index,
            ..
        } = call_instr
        {
            assert_eq!(*arg_count, 0);
            assert_eq!(bytecode.var_names[*name_index], "foo");
        }
    }

    #[test]
    fn test_compile_function_call_with_args() {
        // Test: add(10, 20)
        let program = Program {
            statements: vec![Statement::Expression {
                value: Expression::Call {
                    name: "add".to_string(),
                    args: vec![Expression::Integer(10), Expression::Integer(20)],
                },
            }],
        };

        let bytecode = compile(&program).unwrap();

        // Should have: LoadConst(10), LoadConst(20), Call, SetResult, Halt
        assert!(bytecode
            .instructions
            .iter()
            .any(|i| matches!(i, Instruction::Call { .. })));

        // Check that arguments are compiled
        let loadconst_count = bytecode
            .instructions
            .iter()
            .filter(|i| matches!(i, Instruction::LoadConst { .. }))
            .count();
        assert_eq!(loadconst_count, 2);

        // Check Call instruction has correct arg_count
        let call_instr = bytecode
            .instructions
            .iter()
            .find(|i| matches!(i, Instruction::Call { .. }))
            .unwrap();

        if let Instruction::Call { arg_count, .. } = call_instr {
            assert_eq!(*arg_count, 2);
        }
    }

    #[test]
    fn test_compile_return_with_value() {
        // Test: def foo(): return 42
        let program = Program {
            statements: vec![Statement::FunctionDef {
                name: "foo".to_string(),
                params: vec![],
                body: vec![Statement::Return {
                    value: Some(Expression::Integer(42)),
                }],
            }],
        };

        let bytecode = compile(&program).unwrap();

        // Find Return instruction
        let return_instr = bytecode
            .instructions
            .iter()
            .find(|i| matches!(i, Instruction::Return { .. }))
            .unwrap();

        if let Instruction::Return { has_value, src_reg } = return_instr {
            assert!(*has_value);
            assert!(src_reg.is_some());
        }
    }

    #[test]
    fn test_compile_return_without_value() {
        // Test: def foo(): return
        let program = Program {
            statements: vec![Statement::FunctionDef {
                name: "foo".to_string(),
                params: vec![],
                body: vec![Statement::Return { value: None }],
            }],
        };

        let bytecode = compile(&program).unwrap();

        // Find Return instruction
        let return_instr = bytecode
            .instructions
            .iter()
            .find(|i| matches!(i, Instruction::Return { .. }))
            .unwrap();

        if let Instruction::Return { has_value, src_reg } = return_instr {
            assert!(!(*has_value));
            assert_eq!(*src_reg, None);
        }
    }

    #[test]
    fn test_compile_function_scope_isolation() {
        // Test that function local variables don't interfere with global scope
        // def foo(): x = 10; return x
        let program = Program {
            statements: vec![Statement::FunctionDef {
                name: "foo".to_string(),
                params: vec![],
                body: vec![
                    Statement::Assignment {
                        name: "x".to_string(),
                        value: Expression::Integer(10),
                    },
                    Statement::Return {
                        value: Some(Expression::Variable("x".to_string())),
                    },
                ],
            }],
        };

        let bytecode = compile(&program).unwrap();

        // Verify compilation succeeds and function body is present
        assert!(bytecode
            .instructions
            .iter()
            .any(|i| matches!(i, Instruction::DefineFunction { .. })));
        assert!(bytecode
            .instructions
            .iter()
            .any(|i| matches!(i, Instruction::StoreVar { .. })));
    }

    #[test]
    fn test_compile_multiple_functions() {
        // Test: def foo(): return 1; def bar(): return 2
        let program = Program {
            statements: vec![
                Statement::FunctionDef {
                    name: "foo".to_string(),
                    params: vec![],
                    body: vec![Statement::Return {
                        value: Some(Expression::Integer(1)),
                    }],
                },
                Statement::FunctionDef {
                    name: "bar".to_string(),
                    params: vec![],
                    body: vec![Statement::Return {
                        value: Some(Expression::Integer(2)),
                    }],
                },
            ],
        };

        let bytecode = compile(&program).unwrap();

        // Should have two DefineFunction instructions
        let define_count = bytecode
            .instructions
            .iter()
            .filter(|i| matches!(i, Instruction::DefineFunction { .. }))
            .count();
        assert_eq!(define_count, 2);

        // Verify both function names are in var_names pool
        assert!(bytecode.var_names.contains(&"foo".to_string()));
        assert!(bytecode.var_names.contains(&"bar".to_string()));
    }

    #[test]
    fn test_compile_nested_call() {
        // Test: foo(bar())
        let program = Program {
            statements: vec![Statement::Expression {
                value: Expression::Call {
                    name: "foo".to_string(),
                    args: vec![Expression::Call {
                        name: "bar".to_string(),
                        args: vec![],
                    }],
                },
            }],
        };

        let bytecode = compile(&program).unwrap();

        // Should have two Call instructions
        let call_count = bytecode
            .instructions
            .iter()
            .filter(|i| matches!(i, Instruction::Call { .. }))
            .count();
        assert_eq!(call_count, 2);
    }

    #[test]
    fn test_compile_function_with_complex_body() {
        // Test: def calc(x): y = x + 1; print(y); return y * 2
        let program = Program {
            statements: vec![Statement::FunctionDef {
                name: "calc".to_string(),
                params: vec!["x".to_string()],
                body: vec![
                    Statement::Assignment {
                        name: "y".to_string(),
                        value: Expression::BinaryOp {
                            left: Box::new(Expression::Variable("x".to_string())),
                            op: BinaryOperator::Add,
                            right: Box::new(Expression::Integer(1)),
                        },
                    },
                    Statement::Print {
                        value: Expression::Variable("y".to_string()),
                    },
                    Statement::Return {
                        value: Some(Expression::BinaryOp {
                            left: Box::new(Expression::Variable("y".to_string())),
                            op: BinaryOperator::Mul,
                            right: Box::new(Expression::Integer(2)),
                        }),
                    },
                ],
            }],
        };

        let bytecode = compile(&program).unwrap();

        // Verify function compiled with all statement types
        assert!(bytecode
            .instructions
            .iter()
            .any(|i| matches!(i, Instruction::DefineFunction { .. })));
        assert!(bytecode
            .instructions
            .iter()
            .any(|i| matches!(i, Instruction::StoreVar { .. })));
        assert!(bytecode
            .instructions
            .iter()
            .any(|i| matches!(i, Instruction::Print { .. })));
        assert!(bytecode
            .instructions
            .iter()
            .any(|i| matches!(i, Instruction::Return { .. })));
    }

    #[test]
    fn test_compile_function_call_with_expression_args() {
        // Test: add(1 + 2, 3 * 4)
        let program = Program {
            statements: vec![Statement::Expression {
                value: Expression::Call {
                    name: "add".to_string(),
                    args: vec![
                        Expression::BinaryOp {
                            left: Box::new(Expression::Integer(1)),
                            op: BinaryOperator::Add,
                            right: Box::new(Expression::Integer(2)),
                        },
                        Expression::BinaryOp {
                            left: Box::new(Expression::Integer(3)),
                            op: BinaryOperator::Mul,
                            right: Box::new(Expression::Integer(4)),
                        },
                    ],
                },
            }],
        };

        let bytecode = compile(&program).unwrap();

        // Verify arguments are compiled as expressions
        assert!(bytecode
            .instructions
            .iter()
            .any(|i| matches!(i, Instruction::BinaryOp { .. })));
        assert!(bytecode
            .instructions
            .iter()
            .any(|i| matches!(i, Instruction::Call { .. })));
    }

    #[test]
    fn test_compile_function_register_allocation() {
        // Test that parameters use registers 0..N
        // def add(a, b, c): return a + b + c
        let program = Program {
            statements: vec![Statement::FunctionDef {
                name: "add".to_string(),
                params: vec!["a".to_string(), "b".to_string(), "c".to_string()],
                body: vec![Statement::Return {
                    value: Some(Expression::BinaryOp {
                        left: Box::new(Expression::BinaryOp {
                            left: Box::new(Expression::Variable("a".to_string())),
                            op: BinaryOperator::Add,
                            right: Box::new(Expression::Variable("b".to_string())),
                        }),
                        op: BinaryOperator::Add,
                        right: Box::new(Expression::Variable("c".to_string())),
                    }),
                }],
            }],
        };

        let bytecode = compile(&program).unwrap();

        // Verify DefineFunction has correct param_count
        let define_func = bytecode
            .instructions
            .iter()
            .find(|i| matches!(i, Instruction::DefineFunction { .. }))
            .unwrap();

        if let Instruction::DefineFunction { param_count, .. } = define_func {
            assert_eq!(*param_count, 3);
        }

        // Function body should compile successfully
        assert!(bytecode
            .instructions
            .iter()
            .any(|i| matches!(i, Instruction::Return { .. })));
    }

    #[test]
    fn test_compile_call_tracks_argument_registers() {
        // Test: add(10, 20) - verify first_arg_reg is tracked correctly
        let program = Program {
            statements: vec![Statement::Expression {
                value: Expression::Call {
                    name: "add".to_string(),
                    args: vec![Expression::Integer(10), Expression::Integer(20)],
                },
            }],
        };

        let bytecode = compile(&program).unwrap();

        // Find Call instruction
        let call_instr = bytecode
            .instructions
            .iter()
            .find(|i| matches!(i, Instruction::Call { .. }))
            .unwrap();

        // Verify Call instruction has correct first_arg_reg
        if let Instruction::Call {
            arg_count,
            first_arg_reg,
            dest_reg,
            ..
        } = call_instr
        {
            assert_eq!(*arg_count, 2);
            // With right-to-left evaluation and consecutive register allocation,
            // arguments end up in consecutive registers (after potential copying)
            // Just verify arg_count is correct and dest_reg comes after arguments
            assert!(*dest_reg >= *first_arg_reg + 2);
        } else {
            panic!("Expected Call instruction");
        }

        // Verify that arguments are compiled (exact register/const assignments may vary
        // with right-to-left evaluation, but we should have LoadConst instructions)
        assert!(matches!(
            bytecode.instructions[0],
            Instruction::LoadConst { .. }
        ));
        assert!(matches!(
            bytecode.instructions[1],
            Instruction::LoadConst { .. }
        ));
    }

    #[test]
    fn test_compile_call_no_args_first_arg_reg() {
        // Test: foo() - verify first_arg_reg when no arguments
        let program = Program {
            statements: vec![Statement::Expression {
                value: Expression::Call {
                    name: "foo".to_string(),
                    args: vec![],
                },
            }],
        };

        let bytecode = compile(&program).unwrap();

        // Find Call instruction
        let call_instr = bytecode
            .instructions
            .iter()
            .find(|i| matches!(i, Instruction::Call { .. }))
            .unwrap();

        // Verify Call instruction
        if let Instruction::Call {
            arg_count,
            first_arg_reg,
            ..
        } = call_instr
        {
            assert_eq!(*arg_count, 0);
            // When no arguments, first_arg_reg should be 0 (placeholder)
            assert_eq!(*first_arg_reg, 0);
        } else {
            panic!("Expected Call instruction");
        }
    }

    #[test]
    fn test_compile_nested_calls_register_tracking() {
        // Test: foo(bar(1, 2), 3) - verify register tracking with nested calls
        // With right-to-left evaluation: 3 is evaluated first, then bar(1,2)
        let program = Program {
            statements: vec![Statement::Expression {
                value: Expression::Call {
                    name: "foo".to_string(),
                    args: vec![
                        Expression::Call {
                            name: "bar".to_string(),
                            args: vec![Expression::Integer(1), Expression::Integer(2)],
                        },
                        Expression::Integer(3),
                    ],
                },
            }],
        };

        let bytecode = compile(&program).unwrap();

        // Find both Call instructions
        let call_instrs: Vec<_> = bytecode
            .instructions
            .iter()
            .filter(|i| matches!(i, Instruction::Call { .. }))
            .collect();

        assert_eq!(call_instrs.len(), 2);

        // With right-to-left evaluation, bar(1,2) is evaluated after 3
        // So first Call we encounter is bar(1, 2)
        if let Instruction::Call {
            arg_count,
            first_arg_reg: _,
            name_index,
            ..
        } = call_instrs[0]
        {
            assert_eq!(bytecode.var_names[*name_index], "bar");
            assert_eq!(*arg_count, 2);
            // bar's args are in registers starting from wherever they were allocated
            // Just verify arg_count is correct
        }

        // Second call is foo(<result of bar>, 3)
        if let Instruction::Call {
            arg_count,
            first_arg_reg: _,
            name_index,
            ..
        } = call_instrs[1]
        {
            assert_eq!(bytecode.var_names[*name_index], "foo");
            assert_eq!(*arg_count, 2);
            // Just verify arg_count is correct, register allocation may vary
        }
    }

    #[test]
    fn test_compile_function_metadata_tracking() {
        // Test: Verify that function metadata is tracked separately
        // def foo(a, b): return a + b
        // def bar(): return 42
        let program = Program {
            statements: vec![
                Statement::FunctionDef {
                    name: "foo".to_string(),
                    params: vec!["a".to_string(), "b".to_string()],
                    body: vec![Statement::Return {
                        value: Some(Expression::BinaryOp {
                            left: Box::new(Expression::Variable("a".to_string())),
                            op: BinaryOperator::Add,
                            right: Box::new(Expression::Variable("b".to_string())),
                        }),
                    }],
                },
                Statement::FunctionDef {
                    name: "bar".to_string(),
                    params: vec![],
                    body: vec![Statement::Return {
                        value: Some(Expression::Integer(42)),
                    }],
                },
            ],
        };

        let bytecode = compile(&program).unwrap();

        // Find both DefineFunction instructions
        let define_funcs: Vec<_> = bytecode
            .instructions
            .iter()
            .filter(|i| matches!(i, Instruction::DefineFunction { .. }))
            .collect();

        assert_eq!(define_funcs.len(), 2, "Should have 2 function definitions");

        // Verify first function metadata
        if let Instruction::DefineFunction {
            name_index,
            param_count,
            body_start: _,
            body_len,
            ..
        } = define_funcs[0]
        {
            assert_eq!(bytecode.var_names[*name_index], "foo");
            assert_eq!(*param_count, 2);
            assert!(*body_len > 0, "Function body should have instructions");
        }

        // Verify second function metadata
        if let Instruction::DefineFunction {
            name_index,
            param_count,
            body_start: _,
            body_len,
            ..
        } = define_funcs[1]
        {
            assert_eq!(bytecode.var_names[*name_index], "bar");
            assert_eq!(*param_count, 0);
            assert!(*body_len > 0, "Function body should have instructions");
        }
    }

    #[test]
    fn test_compile_function_without_explicit_return() {
        // Test: Function without explicit return (should still compile)
        // def foo(): x = 5
        let program = Program {
            statements: vec![Statement::FunctionDef {
                name: "foo".to_string(),
                params: vec![],
                body: vec![Statement::Assignment {
                    name: "x".to_string(),
                    value: Expression::Integer(5),
                }],
            }],
        };

        let bytecode = compile(&program).unwrap();

        // Should compile successfully even without explicit return
        assert!(bytecode
            .instructions
            .iter()
            .any(|i| matches!(i, Instruction::DefineFunction { .. })));

        // Should NOT have a Return instruction (function has implicit None return)
        let has_return = bytecode
            .instructions
            .iter()
            .any(|i| matches!(i, Instruction::Return { .. }));
        assert!(
            !has_return,
            "Function without explicit return should not have Return instruction in body"
        );
    }

    #[test]
    fn test_compile_call_argument_consecutive_registers() {
        // Test: Verify arguments are compiled into consecutive registers
        // add(1 + 2, 3 * 4, 5)
        let program = Program {
            statements: vec![Statement::Expression {
                value: Expression::Call {
                    name: "add".to_string(),
                    args: vec![
                        Expression::BinaryOp {
                            left: Box::new(Expression::Integer(1)),
                            op: BinaryOperator::Add,
                            right: Box::new(Expression::Integer(2)),
                        },
                        Expression::BinaryOp {
                            left: Box::new(Expression::Integer(3)),
                            op: BinaryOperator::Mul,
                            right: Box::new(Expression::Integer(4)),
                        },
                        Expression::Integer(5),
                    ],
                },
            }],
        };

        let bytecode = compile(&program).unwrap();

        // Find Call instruction
        let call_instr = bytecode
            .instructions
            .iter()
            .find(|i| matches!(i, Instruction::Call { .. }))
            .unwrap();

        // Verify Call has correct arg_count and first_arg_reg
        if let Instruction::Call {
            arg_count,
            first_arg_reg,
            ..
        } = call_instr
        {
            assert_eq!(*arg_count, 3);
            // With consecutive register allocation fix:
            // Arg1 (1+2): compiles to regs 0, 1, result in 2
            // Arg2 (3*4): compiles to regs 3, 4, result in 5
            // Arg3 (5): compiles to reg 6
            // Since results (2, 5, 6) are not consecutive, they're copied to consecutive registers starting at 7
            // So first_arg_reg = 7, and args are in regs 7, 8, 9
            assert_eq!(*first_arg_reg, 7);
        }
    }

    #[test]
    fn test_compile_function_with_many_params() {
        // Test: Function with many parameters (not 255, but a reasonable large number)
        let params: Vec<String> = (0..20).map(|i| format!("p{}", i)).collect();

        let program = Program {
            statements: vec![Statement::FunctionDef {
                name: "many_params".to_string(),
                params: params.clone(),
                body: vec![Statement::Return {
                    value: Some(Expression::Variable("p0".to_string())),
                }],
            }],
        };

        let bytecode = compile(&program).unwrap();

        // Verify DefineFunction has correct param_count
        let define_func = bytecode
            .instructions
            .iter()
            .find(|i| matches!(i, Instruction::DefineFunction { .. }))
            .unwrap();

        if let Instruction::DefineFunction { param_count, .. } = define_func {
            assert_eq!(*param_count, 20);
        }
    }

    #[test]
    fn test_compile_recursive_function_call() {
        // Test: Function that calls itself (recursive)
        // def factorial(n): return factorial(n)
        let program = Program {
            statements: vec![Statement::FunctionDef {
                name: "factorial".to_string(),
                params: vec!["n".to_string()],
                body: vec![Statement::Return {
                    value: Some(Expression::Call {
                        name: "factorial".to_string(),
                        args: vec![Expression::Variable("n".to_string())],
                    }),
                }],
            }],
        };

        let bytecode = compile(&program).unwrap();

        // Should compile successfully (recursion detection is runtime, not compile-time)
        assert!(bytecode
            .instructions
            .iter()
            .any(|i| matches!(i, Instruction::DefineFunction { .. })));
        assert!(bytecode
            .instructions
            .iter()
            .any(|i| matches!(i, Instruction::Call { .. })));
    }

    #[test]
    fn test_compile_function_call_in_assignment() {
        // Test: result = add(1, 2)
        let program = Program {
            statements: vec![Statement::Assignment {
                name: "result".to_string(),
                value: Expression::Call {
                    name: "add".to_string(),
                    args: vec![Expression::Integer(1), Expression::Integer(2)],
                },
            }],
        };

        let bytecode = compile(&program).unwrap();

        // Should have Call and StoreVar, but NO SetResult
        assert!(bytecode
            .instructions
            .iter()
            .any(|i| matches!(i, Instruction::Call { .. })));
        assert!(bytecode
            .instructions
            .iter()
            .any(|i| matches!(i, Instruction::StoreVar { .. })));

        // CRITICAL: Assignment should NOT have SetResult
        let has_setresult = bytecode
            .instructions
            .iter()
            .any(|i| matches!(i, Instruction::SetResult { .. }));
        assert!(
            !has_setresult,
            "Assignment to function call should not emit SetResult"
        );
    }

    #[test]
    fn test_compile_function_body_metadata_offsets() {
        // Test: Verify body_start points to correct location
        // def foo(): return 1
        // def bar(): return 2
        let program = Program {
            statements: vec![
                Statement::FunctionDef {
                    name: "foo".to_string(),
                    params: vec![],
                    body: vec![Statement::Return {
                        value: Some(Expression::Integer(1)),
                    }],
                },
                Statement::FunctionDef {
                    name: "bar".to_string(),
                    params: vec![],
                    body: vec![Statement::Return {
                        value: Some(Expression::Integer(2)),
                    }],
                },
            ],
        };

        let bytecode = compile(&program).unwrap();

        // Expected layout:
        // 0: DefineFunction foo (body_start points after Halt)
        // 1: DefineFunction bar (body_start points after foo's body)
        // 2: Halt
        // 3+: foo body
        // N+: bar body

        // Find both DefineFunction instructions
        let define_funcs: Vec<_> = bytecode
            .instructions
            .iter()
            .enumerate()
            .filter(|(_, i)| matches!(i, Instruction::DefineFunction { .. }))
            .collect();

        assert_eq!(define_funcs.len(), 2);

        // Find Halt instruction
        let halt_index = bytecode
            .instructions
            .iter()
            .position(|i| matches!(i, Instruction::Halt))
            .expect("Should have Halt instruction");

        // Verify body_start for first function
        if let (
            _idx1,
            Instruction::DefineFunction {
                body_start: start1,
                body_len: len1,
                ..
            },
        ) = define_funcs[0]
        {
            // body_start should point AFTER the Halt instruction
            assert!(*start1 > halt_index, "body_start should point after Halt");
            assert!(*len1 > 0, "body_len should be positive");
        }

        // Verify body_start for second function
        if let (
            _idx2,
            Instruction::DefineFunction {
                body_start: start2,
                body_len: len2,
                ..
            },
        ) = define_funcs[1]
        {
            // body_start should point AFTER the Halt instruction
            assert!(*start2 > halt_index, "body_start should point after Halt");
            assert!(*len2 > 0, "body_len should be positive");

            // Second function should start after first function
            if let (
                _,
                Instruction::DefineFunction {
                    body_start: start1,
                    body_len: len1,
                    ..
                },
            ) = define_funcs[0]
            {
                assert!(
                    *start2 >= start1 + len1,
                    "Second function should start after first function"
                );
            }
        }
    }

    // ========== VariableInterner Tests ==========

    #[test]
    fn test_variable_interner_new_preinterns_a_z() {
        let interner = VariableInterner::new();

        // Verify a-z are pre-interned (26 letters)
        for c in b'a'..=b'z' {
            let name = (c as char).to_string();
            let id = interner.name_to_id.get(&name);
            assert!(id.is_some(), "Variable '{}' should be pre-interned", name);
        }
    }

    #[test]
    fn test_variable_interner_new_preinterns_common_names() {
        let interner = VariableInterner::new();

        // Verify common names are pre-interned
        let common_names = vec!["result", "value", "temp", "count", "index", "data"];
        for name in &common_names {
            let id = interner.name_to_id.get(*name);
            assert!(id.is_some(), "Variable '{}' should be pre-interned", name);
        }
    }

    #[test]
    fn test_variable_interner_new_count() {
        let interner = VariableInterner::new();

        // 26 letters + 6 common names = 32 total
        assert_eq!(
            interner.name_to_id.len(),
            32,
            "Should have exactly 32 pre-interned names"
        );
        assert_eq!(
            interner.id_to_name.len(),
            32,
            "Should have exactly 32 pre-interned IDs"
        );
        assert_eq!(interner.next_id, 32, "Next ID should be 32");
    }

    #[test]
    fn test_variable_interner_intern_new_name() {
        let mut interner = VariableInterner::new();

        let id = interner.intern("custom_var");
        assert_eq!(id, 32, "First custom variable should get ID 32");
        assert_eq!(interner.next_id, 33, "Next ID should be 33");
        assert_eq!(interner.name_to_id.get("custom_var"), Some(&32));
        assert_eq!(
            interner.id_to_name.get(&32),
            Some(&"custom_var".to_string())
        );
    }

    #[test]
    fn test_variable_interner_intern_deduplication() {
        let mut interner = VariableInterner::new();

        let id1 = interner.intern("my_var");
        let id2 = interner.intern("my_var");
        let id3 = interner.intern("my_var");

        assert_eq!(id1, id2, "Same variable should get same ID");
        assert_eq!(id2, id3, "Same variable should get same ID");
        assert_eq!(
            interner.name_to_id.len(),
            33,
            "Should only have one entry for my_var"
        );
    }

    #[test]
    fn test_variable_interner_intern_preintered_name() {
        let mut interner = VariableInterner::new();

        // Intern a pre-interned name
        let id_a = interner.intern("a");
        let id_result = interner.intern("result");

        // Should return the pre-interned IDs, not create new ones
        assert!(id_a < 32, "Pre-interned 'a' should have ID < 32");
        assert!(id_result < 32, "Pre-interned 'result' should have ID < 32");
        assert_eq!(interner.next_id, 32, "Next ID should still be 32");
    }

    #[test]
    fn test_variable_interner_get_name() {
        let mut interner = VariableInterner::new();

        let id = interner.intern("test_var");
        assert_eq!(interner.get_name(id), Some("test_var"));
        assert_eq!(
            interner.get_name(9999),
            None,
            "Non-existent ID should return None"
        );
    }

    #[test]
    fn test_variable_interner_get_all_names() {
        let mut interner = VariableInterner::new();

        interner.intern("zebra");
        interner.intern("apple");

        let all_names = interner.get_all_names();

        // Should have 32 pre-interned + 2 custom = 34 total
        assert_eq!(all_names.len(), 34);

        // Verify they're in ID order (not alphabetical)
        // The first 26 should be a-z in order
        assert_eq!(all_names[0], "a");
        assert_eq!(all_names[25], "z");
    }

    #[test]
    fn test_variable_interner_default() {
        let interner = VariableInterner::default();

        // Default should be same as new()
        assert_eq!(interner.name_to_id.len(), 32);
        assert_eq!(interner.next_id, 32);
    }

    #[test]
    fn test_variable_name_interning_in_compilation() {
        // Test that variable interning works correctly in actual compilation
        let program = Program {
            statements: vec![
                Statement::Assignment {
                    name: "x".to_string(),
                    value: Expression::Integer(10),
                },
                Statement::Assignment {
                    name: "x".to_string(), // Same variable name
                    value: Expression::Integer(20),
                },
                Statement::Expression {
                    value: Expression::Variable("x".to_string()),
                },
            ],
        };

        let bytecode = compile(&program).unwrap();

        // Extract all var_ids used in StoreVar and LoadVar instructions
        let mut var_ids = Vec::new();
        for instr in &bytecode.instructions {
            match instr {
                Instruction::StoreVar { var_id, .. } => var_ids.push(*var_id),
                Instruction::LoadVar { var_id, .. } => var_ids.push(*var_id),
                _ => {}
            }
        }

        // All references to "x" should use the same ID
        assert!(
            var_ids.len() >= 2,
            "Should have at least 2 variable operations"
        );
        assert!(
            var_ids.iter().all(|&id| id == var_ids[0]),
            "All references to 'x' should use the same var_id"
        );
    }

    #[test]
    fn test_multiple_variables_get_different_ids() {
        let program = Program {
            statements: vec![
                Statement::Assignment {
                    name: "x".to_string(),
                    value: Expression::Integer(10),
                },
                Statement::Assignment {
                    name: "y".to_string(),
                    value: Expression::Integer(20),
                },
                Statement::Assignment {
                    name: "z".to_string(),
                    value: Expression::Integer(30),
                },
            ],
        };

        let bytecode = compile(&program).unwrap();

        // Extract var_ids from StoreVar instructions
        let mut var_ids = Vec::new();
        for instr in &bytecode.instructions {
            if let Instruction::StoreVar { var_id, .. } = instr {
                var_ids.push(*var_id);
            }
        }

        assert_eq!(var_ids.len(), 3, "Should have 3 store operations");

        // All IDs should be different
        assert_ne!(var_ids[0], var_ids[1], "x and y should have different IDs");
        assert_ne!(var_ids[1], var_ids[2], "y and z should have different IDs");
        assert_ne!(var_ids[0], var_ids[2], "x and z should have different IDs");
    }

    #[test]
    fn test_var_ids_and_var_names_parallel() {
        let program = Program {
            statements: vec![
                Statement::Assignment {
                    name: "foo".to_string(),
                    value: Expression::Integer(1),
                },
                Statement::Assignment {
                    name: "bar".to_string(),
                    value: Expression::Integer(2),
                },
            ],
        };

        let bytecode = compile(&program).unwrap();

        // var_ids and var_names should be parallel arrays
        assert_eq!(
            bytecode.var_names.len(),
            bytecode.var_ids.len(),
            "var_names and var_ids should have same length"
        );

        // Each var_name should have corresponding var_id at same index
        for (idx, name) in bytecode.var_names.iter().enumerate() {
            let var_id = bytecode.var_ids[idx];
            // Find the instruction using this var_name_index
            for instr in &bytecode.instructions {
                if let Instruction::StoreVar {
                    var_name_index,
                    var_id: instr_var_id,
                    ..
                } = instr
                {
                    if *var_name_index == idx {
                        assert_eq!(
                            *instr_var_id, var_id,
                            "var_id in instruction should match var_id in pool for '{}'",
                            name
                        );
                    }
                }
            }
        }
    }

    #[test]
    fn test_preintered_variables_use_low_ids() {
        let program = Program {
            statements: vec![
                Statement::Assignment {
                    name: "a".to_string(),
                    value: Expression::Integer(1),
                },
                Statement::Assignment {
                    name: "result".to_string(),
                    value: Expression::Integer(2),
                },
                Statement::Assignment {
                    name: "custom_var".to_string(),
                    value: Expression::Integer(3),
                },
            ],
        };

        let bytecode = compile(&program).unwrap();

        // Extract var_ids from StoreVar instructions in order
        let mut var_id_map = std::collections::HashMap::new();
        for instr in &bytecode.instructions {
            if let Instruction::StoreVar {
                var_name_index,
                var_id,
                ..
            } = instr
            {
                let name = &bytecode.var_names[*var_name_index];
                var_id_map.insert(name.clone(), *var_id);
            }
        }

        // Pre-interned variables should have IDs < 32
        assert!(
            var_id_map.get("a").unwrap() < &32,
            "'a' should be pre-interned with ID < 32"
        );
        assert!(
            var_id_map.get("result").unwrap() < &32,
            "'result' should be pre-interned with ID < 32"
        );

        // Custom variable should have ID >= 32
        assert!(
            var_id_map.get("custom_var").unwrap() >= &32,
            "'custom_var' should have ID >= 32"
        );
    }
}
