//! Abstract Syntax Tree (AST) node definitions for Phase 1
//!
//! Pure data structures optimized for arena allocation.
//! Represents the parsed structure of Python-like source code.

/// Root AST node containing a list of statements
#[derive(Debug, Clone, PartialEq)]
pub struct Program {
    pub statements: Vec<Statement>,
}

/// Statement variants in the language
#[derive(Debug, Clone, PartialEq)]
pub enum Statement {
    /// Variable assignment: `name = expression`
    Assignment { name: String, value: Expression },
    /// Print statement: `print(expression)`
    Print { value: Expression },
    /// Expression statement: standalone expression
    Expression { value: Expression },
    /// Function definition: `def name(params): body`
    FunctionDef {
        name: String,
        params: Vec<String>,
        body: Vec<Statement>,
    },
    /// Return statement: `return [value]`
    Return { value: Option<Expression> },
}

/// Expression variants representing values and operations
#[derive(Debug, Clone, PartialEq)]
pub enum Expression {
    /// Integer literal
    Integer(i64),
    /// Variable reference
    Variable(String),
    /// Binary operation: `left op right`
    BinaryOp {
        left: Box<Expression>,
        op: BinaryOperator,
        right: Box<Expression>,
    },
    /// Unary operation: `op operand`
    UnaryOp {
        op: UnaryOperator,
        operand: Box<Expression>,
    },
    /// Function call: `name(args)`
    Call { name: String, args: Vec<Expression> },
}

/// Binary operators with precedence levels
///
/// Precedence levels:
/// - Level 1: Addition, Subtraction
/// - Level 2: Multiplication, Division, Floor Division, Modulo
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinaryOperator {
    /// Addition operator (+)
    /// Precedence: 1
    Add,
    /// Subtraction operator (-)
    /// Precedence: 1
    Sub,
    /// Multiplication operator (*)
    /// Precedence: 2
    Mul,
    /// Division operator (/)
    /// Precedence: 2
    Div,
    /// Floor division operator (//)
    /// Precedence: 2
    FloorDiv,
    /// Modulo operator (%)
    /// Precedence: 2
    Mod,
}

impl BinaryOperator {
    /// Returns the precedence level of the operator
    ///
    /// Higher values indicate higher precedence (tighter binding).
    /// - Level 1: Add, Sub
    /// - Level 2: Mul, Div, FloorDiv, Mod
    pub fn precedence(&self) -> u8 {
        match self {
            BinaryOperator::Add | BinaryOperator::Sub => 1,
            BinaryOperator::Mul
            | BinaryOperator::Div
            | BinaryOperator::FloorDiv
            | BinaryOperator::Mod => 2,
        }
    }
}

/// Unary operators for future extensions
///
/// Currently supports negation and positive sign.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnaryOperator {
    /// Negation operator (-)
    /// Semantics: Returns the arithmetic negation of the operand
    Neg,
    /// Positive sign operator (+)
    /// Semantics: Returns the operand unchanged
    Pos,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_binary_operator_precedence() {
        // Additive operators have precedence 1
        assert_eq!(BinaryOperator::Add.precedence(), 1);
        assert_eq!(BinaryOperator::Sub.precedence(), 1);

        // Multiplicative operators have precedence 2
        assert_eq!(BinaryOperator::Mul.precedence(), 2);
        assert_eq!(BinaryOperator::Div.precedence(), 2);
        assert_eq!(BinaryOperator::FloorDiv.precedence(), 2);
        assert_eq!(BinaryOperator::Mod.precedence(), 2);

        // Verify precedence ordering
        assert!(BinaryOperator::Mul.precedence() > BinaryOperator::Add.precedence());
        assert!(BinaryOperator::Div.precedence() > BinaryOperator::Sub.precedence());
    }

    #[test]
    fn test_ast_construction() {
        // Test simple integer expression
        let expr = Expression::Integer(42);
        assert_eq!(expr, Expression::Integer(42));

        // Test variable expression
        let var = Expression::Variable("x".to_string());
        assert_eq!(var, Expression::Variable("x".to_string()));

        // Test binary operation
        let bin_op = Expression::BinaryOp {
            left: Box::new(Expression::Integer(1)),
            op: BinaryOperator::Add,
            right: Box::new(Expression::Integer(2)),
        };

        if let Expression::BinaryOp { left, op, right } = &bin_op {
            assert_eq!(**left, Expression::Integer(1));
            assert_eq!(*op, BinaryOperator::Add);
            assert_eq!(**right, Expression::Integer(2));
        } else {
            panic!("Expected BinaryOp");
        }
    }

    #[test]
    fn test_nested_expressions() {
        // Test nested binary operations: (1 + 2) * 3
        let nested = Expression::BinaryOp {
            left: Box::new(Expression::BinaryOp {
                left: Box::new(Expression::Integer(1)),
                op: BinaryOperator::Add,
                right: Box::new(Expression::Integer(2)),
            }),
            op: BinaryOperator::Mul,
            right: Box::new(Expression::Integer(3)),
        };

        // Verify structure
        if let Expression::BinaryOp { left, op, right } = &nested {
            assert_eq!(*op, BinaryOperator::Mul);
            assert_eq!(**right, Expression::Integer(3));

            if let Expression::BinaryOp {
                left: inner_left,
                op: inner_op,
                right: inner_right,
            } = &**left
            {
                assert_eq!(**inner_left, Expression::Integer(1));
                assert_eq!(*inner_op, BinaryOperator::Add);
                assert_eq!(**inner_right, Expression::Integer(2));
            } else {
                panic!("Expected nested BinaryOp");
            }
        } else {
            panic!("Expected BinaryOp");
        }
    }

    #[test]
    fn test_unary_expression() {
        // Test unary negation
        let unary = Expression::UnaryOp {
            op: UnaryOperator::Neg,
            operand: Box::new(Expression::Integer(42)),
        };

        if let Expression::UnaryOp { op, operand } = &unary {
            assert_eq!(*op, UnaryOperator::Neg);
            assert_eq!(**operand, Expression::Integer(42));
        } else {
            panic!("Expected UnaryOp");
        }
    }

    #[test]
    fn test_statement_variants() {
        // Test assignment statement
        let assign = Statement::Assignment {
            name: "x".to_string(),
            value: Expression::Integer(42),
        };

        if let Statement::Assignment { name, value } = &assign {
            assert_eq!(name, "x");
            assert_eq!(*value, Expression::Integer(42));
        } else {
            panic!("Expected Assignment");
        }

        // Test print statement
        let print = Statement::Print {
            value: Expression::Variable("x".to_string()),
        };

        if let Statement::Print { value } = &print {
            assert_eq!(*value, Expression::Variable("x".to_string()));
        } else {
            panic!("Expected Print");
        }

        // Test expression statement
        let expr_stmt = Statement::Expression {
            value: Expression::Integer(123),
        };

        if let Statement::Expression { value } = &expr_stmt {
            assert_eq!(*value, Expression::Integer(123));
        } else {
            panic!("Expected Expression statement");
        }
    }

    #[test]
    fn test_program_construction() {
        let program = Program {
            statements: vec![
                Statement::Assignment {
                    name: "x".to_string(),
                    value: Expression::Integer(10),
                },
                Statement::Print {
                    value: Expression::Variable("x".to_string()),
                },
            ],
        };

        assert_eq!(program.statements.len(), 2);
    }

    #[test]
    fn test_clone_trait() {
        let expr = Expression::Integer(42);
        let cloned = expr.clone();
        assert_eq!(expr, cloned);

        let stmt = Statement::Print {
            value: Expression::Integer(100),
        };
        let cloned_stmt = stmt.clone();
        assert_eq!(stmt, cloned_stmt);

        let op = BinaryOperator::Add;
        let cloned_op = op;
        assert_eq!(op, cloned_op);
    }

    #[test]
    fn test_equality_trait() {
        // Test expression equality
        assert_eq!(Expression::Integer(42), Expression::Integer(42));
        assert_ne!(Expression::Integer(42), Expression::Integer(43));

        assert_eq!(
            Expression::Variable("x".to_string()),
            Expression::Variable("x".to_string())
        );
        assert_ne!(
            Expression::Variable("x".to_string()),
            Expression::Variable("y".to_string())
        );

        // Test operator equality
        assert_eq!(BinaryOperator::Add, BinaryOperator::Add);
        assert_ne!(BinaryOperator::Add, BinaryOperator::Sub);

        assert_eq!(UnaryOperator::Neg, UnaryOperator::Neg);
        assert_ne!(UnaryOperator::Neg, UnaryOperator::Pos);

        // Test statement equality
        let stmt1 = Statement::Assignment {
            name: "x".to_string(),
            value: Expression::Integer(42),
        };
        let stmt2 = Statement::Assignment {
            name: "x".to_string(),
            value: Expression::Integer(42),
        };
        let stmt3 = Statement::Assignment {
            name: "y".to_string(),
            value: Expression::Integer(42),
        };

        assert_eq!(stmt1, stmt2);
        assert_ne!(stmt1, stmt3);
    }

    #[test]
    fn test_complex_nested_expression() {
        // Test: (a + b) * (c - d) / 2
        let complex = Expression::BinaryOp {
            left: Box::new(Expression::BinaryOp {
                left: Box::new(Expression::BinaryOp {
                    left: Box::new(Expression::Variable("a".to_string())),
                    op: BinaryOperator::Add,
                    right: Box::new(Expression::Variable("b".to_string())),
                }),
                op: BinaryOperator::Mul,
                right: Box::new(Expression::BinaryOp {
                    left: Box::new(Expression::Variable("c".to_string())),
                    op: BinaryOperator::Sub,
                    right: Box::new(Expression::Variable("d".to_string())),
                }),
            }),
            op: BinaryOperator::Div,
            right: Box::new(Expression::Integer(2)),
        };

        // Verify it's a valid expression (can be cloned and compared)
        let cloned = complex.clone();
        assert_eq!(complex, cloned);
    }

    // ========== Function AST Node Tests ==========

    #[test]
    fn test_function_def_construction_no_params() {
        let func = Statement::FunctionDef {
            name: "foo".to_string(),
            params: vec![],
            body: vec![Statement::Return {
                value: Some(Expression::Integer(42)),
            }],
        };
        if let Statement::FunctionDef { name, params, body } = &func {
            assert_eq!(name, "foo");
            assert_eq!(params.len(), 0);
            assert_eq!(body.len(), 1);
        } else {
            panic!("Expected FunctionDef");
        }
    }

    #[test]
    fn test_function_def_construction_with_params() {
        let func = Statement::FunctionDef {
            name: "add".to_string(),
            params: vec!["a".to_string(), "b".to_string()],
            body: vec![Statement::Return {
                value: Some(Expression::BinaryOp {
                    left: Box::new(Expression::Variable("a".to_string())),
                    op: BinaryOperator::Add,
                    right: Box::new(Expression::Variable("b".to_string())),
                }),
            }],
        };
        if let Statement::FunctionDef { name, params, body } = &func {
            assert_eq!(name, "add");
            assert_eq!(params.len(), 2);
            assert_eq!(params[0], "a");
            assert_eq!(params[1], "b");
            assert_eq!(body.len(), 1);
        } else {
            panic!("Expected FunctionDef");
        }
    }

    #[test]
    fn test_return_with_value() {
        let ret = Statement::Return {
            value: Some(Expression::Integer(42)),
        };
        if let Statement::Return { value } = &ret {
            assert!(value.is_some());
            assert_eq!(value.as_ref().unwrap(), &Expression::Integer(42));
        } else {
            panic!("Expected Return");
        }
    }

    #[test]
    fn test_return_without_value() {
        let ret = Statement::Return { value: None };
        if let Statement::Return { value } = &ret {
            assert!(value.is_none());
        } else {
            panic!("Expected Return");
        }
    }

    #[test]
    fn test_call_expression_no_args() {
        let call = Expression::Call {
            name: "foo".to_string(),
            args: vec![],
        };
        if let Expression::Call { name, args } = &call {
            assert_eq!(name, "foo");
            assert_eq!(args.len(), 0);
        } else {
            panic!("Expected Call");
        }
    }

    #[test]
    fn test_call_expression_with_args() {
        let call = Expression::Call {
            name: "add".to_string(),
            args: vec![Expression::Integer(10), Expression::Integer(20)],
        };
        if let Expression::Call { name, args } = &call {
            assert_eq!(name, "add");
            assert_eq!(args.len(), 2);
            assert_eq!(args[0], Expression::Integer(10));
            assert_eq!(args[1], Expression::Integer(20));
        } else {
            panic!("Expected Call");
        }
    }

    #[test]
    fn test_function_def_equality() {
        let func1 = Statement::FunctionDef {
            name: "foo".to_string(),
            params: vec!["x".to_string()],
            body: vec![Statement::Return {
                value: Some(Expression::Variable("x".to_string())),
            }],
        };
        let func2 = Statement::FunctionDef {
            name: "foo".to_string(),
            params: vec!["x".to_string()],
            body: vec![Statement::Return {
                value: Some(Expression::Variable("x".to_string())),
            }],
        };
        let func3 = Statement::FunctionDef {
            name: "bar".to_string(),
            params: vec!["x".to_string()],
            body: vec![Statement::Return {
                value: Some(Expression::Variable("x".to_string())),
            }],
        };
        assert_eq!(func1, func2);
        assert_ne!(func1, func3);
    }

    #[test]
    fn test_return_equality() {
        let ret1 = Statement::Return {
            value: Some(Expression::Integer(42)),
        };
        let ret2 = Statement::Return {
            value: Some(Expression::Integer(42)),
        };
        let ret3 = Statement::Return {
            value: Some(Expression::Integer(43)),
        };
        let ret4 = Statement::Return { value: None };
        assert_eq!(ret1, ret2);
        assert_ne!(ret1, ret3);
        assert_ne!(ret1, ret4);
    }

    #[test]
    fn test_call_equality() {
        let call1 = Expression::Call {
            name: "foo".to_string(),
            args: vec![Expression::Integer(1)],
        };
        let call2 = Expression::Call {
            name: "foo".to_string(),
            args: vec![Expression::Integer(1)],
        };
        let call3 = Expression::Call {
            name: "bar".to_string(),
            args: vec![Expression::Integer(1)],
        };
        assert_eq!(call1, call2);
        assert_ne!(call1, call3);
    }

    #[test]
    fn test_function_nodes_clone() {
        let func = Statement::FunctionDef {
            name: "test".to_string(),
            params: vec!["a".to_string(), "b".to_string()],
            body: vec![Statement::Return {
                value: Some(Expression::Integer(100)),
            }],
        };
        let func_cloned = func.clone();
        assert_eq!(func, func_cloned);
        let ret = Statement::Return {
            value: Some(Expression::Integer(42)),
        };
        let ret_cloned = ret.clone();
        assert_eq!(ret, ret_cloned);
        let call = Expression::Call {
            name: "func".to_string(),
            args: vec![Expression::Integer(1), Expression::Integer(2)],
        };
        let call_cloned = call.clone();
        assert_eq!(call, call_cloned);
    }

    #[test]
    fn test_function_def_nested_body() {
        let func = Statement::FunctionDef {
            name: "foo".to_string(),
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
                    value: Some(Expression::Variable("y".to_string())),
                },
            ],
        };
        if let Statement::FunctionDef { name, params, body } = &func {
            assert_eq!(name, "foo");
            assert_eq!(params.len(), 1);
            assert_eq!(body.len(), 3);
            assert!(matches!(body[0], Statement::Assignment { .. }));
            assert!(matches!(body[1], Statement::Print { .. }));
            assert!(matches!(body[2], Statement::Return { .. }));
        } else {
            panic!("Expected FunctionDef");
        }
    }

    #[test]
    fn test_call_with_complex_args() {
        let call = Expression::Call {
            name: "add".to_string(),
            args: vec![
                Expression::BinaryOp {
                    left: Box::new(Expression::Integer(1)),
                    op: BinaryOperator::Add,
                    right: Box::new(Expression::Integer(2)),
                },
                Expression::BinaryOp {
                    left: Box::new(Expression::Variable("x".to_string())),
                    op: BinaryOperator::Mul,
                    right: Box::new(Expression::Integer(3)),
                },
            ],
        };
        if let Expression::Call { name, args } = &call {
            assert_eq!(name, "add");
            assert_eq!(args.len(), 2);
            assert!(matches!(args[0], Expression::BinaryOp { .. }));
            assert!(matches!(args[1], Expression::BinaryOp { .. }));
        } else {
            panic!("Expected Call");
        }
    }

    #[test]
    fn test_nested_function_calls() {
        let call = Expression::Call {
            name: "foo".to_string(),
            args: vec![
                Expression::Call {
                    name: "bar".to_string(),
                    args: vec![Expression::Integer(1)],
                },
                Expression::Call {
                    name: "baz".to_string(),
                    args: vec![Expression::Integer(2), Expression::Integer(3)],
                },
            ],
        };
        if let Expression::Call { name, args } = &call {
            assert_eq!(name, "foo");
            assert_eq!(args.len(), 2);
            if let Expression::Call {
                name: inner_name,
                args: inner_args,
            } = &args[0]
            {
                assert_eq!(inner_name, "bar");
                assert_eq!(inner_args.len(), 1);
            } else {
                panic!("Expected nested Call");
            }
            if let Expression::Call {
                name: inner_name,
                args: inner_args,
            } = &args[1]
            {
                assert_eq!(inner_name, "baz");
                assert_eq!(inner_args.len(), 2);
            } else {
                panic!("Expected nested Call");
            }
        } else {
            panic!("Expected Call");
        }
    }

    #[test]
    fn test_function_returning_call() {
        let func = Statement::FunctionDef {
            name: "wrapper".to_string(),
            params: vec![],
            body: vec![Statement::Return {
                value: Some(Expression::Call {
                    name: "foo".to_string(),
                    args: vec![Expression::Integer(42)],
                }),
            }],
        };
        if let Statement::FunctionDef { name, params, body } = &func {
            assert_eq!(name, "wrapper");
            assert_eq!(params.len(), 0);
            assert_eq!(body.len(), 1);
            if let Statement::Return { value } = &body[0] {
                assert!(value.is_some());
                assert!(matches!(value.as_ref().unwrap(), Expression::Call { .. }));
            } else {
                panic!("Expected Return");
            }
        } else {
            panic!("Expected FunctionDef");
        }
    }

    // ========== Edge Case Tests ==========

    #[test]
    fn test_empty_function_body() {
        // Functions with empty body are valid AST nodes (validation happens at compile time)
        let func = Statement::FunctionDef {
            name: "empty".to_string(),
            params: vec![],
            body: vec![],
        };
        if let Statement::FunctionDef { name, params, body } = &func {
            assert_eq!(name, "empty");
            assert_eq!(params.len(), 0);
            assert_eq!(body.len(), 0);
        } else {
            panic!("Expected FunctionDef");
        }
    }

    #[test]
    fn test_function_with_many_params() {
        // Test boundary condition: many parameters
        let params: Vec<String> = (0..50).map(|i| format!("param{}", i)).collect();
        let func = Statement::FunctionDef {
            name: "many_params".to_string(),
            params: params.clone(),
            body: vec![Statement::Return { value: None }],
        };
        if let Statement::FunctionDef {
            name,
            params: p,
            body,
        } = &func
        {
            assert_eq!(name, "many_params");
            assert_eq!(p.len(), 50);
            assert_eq!(p[0], "param0");
            assert_eq!(p[49], "param49");
            assert_eq!(body.len(), 1);
        } else {
            panic!("Expected FunctionDef");
        }
    }

    #[test]
    fn test_call_with_many_args() {
        // Test boundary condition: many arguments
        let args: Vec<Expression> = (0..50).map(Expression::Integer).collect();
        let call = Expression::Call {
            name: "many_args".to_string(),
            args: args.clone(),
        };
        if let Expression::Call { name, args: a } = &call {
            assert_eq!(name, "many_args");
            assert_eq!(a.len(), 50);
            assert_eq!(a[0], Expression::Integer(0));
            assert_eq!(a[49], Expression::Integer(49));
        } else {
            panic!("Expected Call");
        }
    }

    #[test]
    fn test_empty_function_name() {
        // AST allows empty function names (semantic validation happens at compile time)
        let func = Statement::FunctionDef {
            name: "".to_string(),
            params: vec![],
            body: vec![Statement::Return { value: None }],
        };
        if let Statement::FunctionDef { name, .. } = &func {
            assert_eq!(name, "");
        } else {
            panic!("Expected FunctionDef");
        }
    }

    #[test]
    fn test_function_with_special_name() {
        // AST allows any string as function name (validation at compile/parse time)
        let func = Statement::FunctionDef {
            name: "__special_name__".to_string(),
            params: vec![],
            body: vec![Statement::Return { value: None }],
        };
        if let Statement::FunctionDef { name, .. } = &func {
            assert_eq!(name, "__special_name__");
        } else {
            panic!("Expected FunctionDef");
        }
    }

    #[test]
    fn test_return_with_complex_nested_expression() {
        // Return with deeply nested expression
        let ret = Statement::Return {
            value: Some(Expression::BinaryOp {
                left: Box::new(Expression::BinaryOp {
                    left: Box::new(Expression::Call {
                        name: "foo".to_string(),
                        args: vec![Expression::Integer(1)],
                    }),
                    op: BinaryOperator::Add,
                    right: Box::new(Expression::Integer(2)),
                }),
                op: BinaryOperator::Mul,
                right: Box::new(Expression::Call {
                    name: "bar".to_string(),
                    args: vec![],
                }),
            }),
        };
        if let Statement::Return { value } = &ret {
            assert!(value.is_some());
            assert!(matches!(
                value.as_ref().unwrap(),
                Expression::BinaryOp { .. }
            ));
        } else {
            panic!("Expected Return");
        }
    }

    #[test]
    fn test_function_def_clone_independence() {
        // Verify that cloning creates independent copies
        let func = Statement::FunctionDef {
            name: "original".to_string(),
            params: vec!["x".to_string()],
            body: vec![Statement::Return {
                value: Some(Expression::Variable("x".to_string())),
            }],
        };
        let cloned = func.clone();

        // Verify equality
        assert_eq!(func, cloned);

        // Both should be independent (can't directly test as Strings are immutable)
        if let Statement::FunctionDef { name, .. } = &cloned {
            assert_eq!(name, "original");
        }
    }

    #[test]
    fn test_call_with_mixed_expression_types() {
        // Call with various expression types as arguments
        let call = Expression::Call {
            name: "mixed".to_string(),
            args: vec![
                Expression::Integer(42),
                Expression::Variable("x".to_string()),
                Expression::BinaryOp {
                    left: Box::new(Expression::Integer(1)),
                    op: BinaryOperator::Add,
                    right: Box::new(Expression::Integer(2)),
                },
                Expression::UnaryOp {
                    op: UnaryOperator::Neg,
                    operand: Box::new(Expression::Integer(5)),
                },
                Expression::Call {
                    name: "nested".to_string(),
                    args: vec![],
                },
            ],
        };
        if let Expression::Call { name, args } = &call {
            assert_eq!(name, "mixed");
            assert_eq!(args.len(), 5);
            assert!(matches!(args[0], Expression::Integer(_)));
            assert!(matches!(args[1], Expression::Variable(_)));
            assert!(matches!(args[2], Expression::BinaryOp { .. }));
            assert!(matches!(args[3], Expression::UnaryOp { .. }));
            assert!(matches!(args[4], Expression::Call { .. }));
        } else {
            panic!("Expected Call");
        }
    }

    #[test]
    fn test_function_body_with_all_statement_types() {
        // Function with diverse statement types in body
        let func = Statement::FunctionDef {
            name: "complex".to_string(),
            params: vec!["a".to_string(), "b".to_string()],
            body: vec![
                Statement::Assignment {
                    name: "x".to_string(),
                    value: Expression::Variable("a".to_string()),
                },
                Statement::Print {
                    value: Expression::Variable("x".to_string()),
                },
                Statement::Expression {
                    value: Expression::BinaryOp {
                        left: Box::new(Expression::Variable("a".to_string())),
                        op: BinaryOperator::Add,
                        right: Box::new(Expression::Variable("b".to_string())),
                    },
                },
                Statement::Return {
                    value: Some(Expression::Variable("x".to_string())),
                },
            ],
        };
        if let Statement::FunctionDef { name, params, body } = &func {
            assert_eq!(name, "complex");
            assert_eq!(params.len(), 2);
            assert_eq!(body.len(), 4);
            assert!(matches!(body[0], Statement::Assignment { .. }));
            assert!(matches!(body[1], Statement::Print { .. }));
            assert!(matches!(body[2], Statement::Expression { .. }));
            assert!(matches!(body[3], Statement::Return { .. }));
        } else {
            panic!("Expected FunctionDef");
        }
    }

    #[test]
    fn test_return_equality_with_none() {
        // Verify None returns are equal to each other but not to Some returns
        let ret1 = Statement::Return { value: None };
        let ret2 = Statement::Return { value: None };
        let ret3 = Statement::Return {
            value: Some(Expression::Integer(0)),
        };

        assert_eq!(ret1, ret2);
        assert_ne!(ret1, ret3);
        assert_ne!(ret2, ret3);
    }

    #[test]
    fn test_call_with_variable_args() {
        // Call with only variable arguments
        let call = Expression::Call {
            name: "func".to_string(),
            args: vec![
                Expression::Variable("x".to_string()),
                Expression::Variable("y".to_string()),
                Expression::Variable("z".to_string()),
            ],
        };
        if let Expression::Call { name, args } = &call {
            assert_eq!(name, "func");
            assert_eq!(args.len(), 3);
            assert!(args.iter().all(|a| matches!(a, Expression::Variable(_))));
        } else {
            panic!("Expected Call");
        }
    }

    #[test]
    fn test_program_with_multiple_functions() {
        // Program with multiple function definitions
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
                    params: vec!["x".to_string()],
                    body: vec![Statement::Return {
                        value: Some(Expression::Variable("x".to_string())),
                    }],
                },
                Statement::Expression {
                    value: Expression::Call {
                        name: "foo".to_string(),
                        args: vec![],
                    },
                },
            ],
        };
        assert_eq!(program.statements.len(), 3);
        assert!(matches!(
            program.statements[0],
            Statement::FunctionDef { .. }
        ));
        assert!(matches!(
            program.statements[1],
            Statement::FunctionDef { .. }
        ));
        assert!(matches!(
            program.statements[2],
            Statement::Expression { .. }
        ));
    }

    #[test]
    fn test_deeply_nested_function_calls_in_args() {
        // Test call with deeply nested calls as arguments
        let call = Expression::Call {
            name: "outer".to_string(),
            args: vec![
                Expression::Call {
                    name: "inner1".to_string(),
                    args: vec![Expression::Call {
                        name: "innermost".to_string(),
                        args: vec![Expression::Integer(1)],
                    }],
                },
                Expression::Call {
                    name: "inner2".to_string(),
                    args: vec![Expression::Integer(2)],
                },
            ],
        };

        // Clone should work on deeply nested structures
        let cloned = call.clone();
        assert_eq!(call, cloned);
    }
}
