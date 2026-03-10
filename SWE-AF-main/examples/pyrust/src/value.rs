//! Runtime value representation
//!
//! Provides the Value enum and operations for runtime evaluation.
//! Phase 1 supports only Integer values with arithmetic operations.

use crate::ast::{BinaryOperator, UnaryOperator};
use crate::error::RuntimeError;
use std::fmt;

/// Runtime value representation
///
/// Currently supports only Integer values in Phase 1.
/// Future phases will add Float, String, Boolean, and None.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Value {
    /// Integer value (i64)
    Integer(i64),
    /// None value (used for functions returning without value)
    None,
}

impl Value {
    /// Perform a binary operation on two values
    ///
    /// # Arguments
    /// * `op` - The binary operator to apply
    /// * `right` - The right-hand operand
    ///
    /// # Returns
    /// * `Ok(Value)` - The result of the operation
    /// * `Err(RuntimeError)` - If the operation fails (division by zero, overflow, etc.)
    ///
    /// # Errors
    /// * Division by zero for Div, FloorDiv, and Mod operations
    /// * Integer overflow/underflow for any arithmetic operation
    pub fn binary_op(&self, op: BinaryOperator, right: &Value) -> Result<Value, RuntimeError> {
        match (self, right) {
            (Value::None, _) | (_, Value::None) => Err(RuntimeError {
                message: "Cannot perform binary operation on None".to_string(),
                instruction_index: 0,
            }),
            (Value::Integer(left_val), Value::Integer(right_val)) => {
                let result = match op {
                    BinaryOperator::Add => {
                        left_val
                            .checked_add(*right_val)
                            .ok_or_else(|| RuntimeError {
                                message: format!("Integer overflow: {} + {}", left_val, right_val),
                                instruction_index: 0,
                            })?
                    }
                    BinaryOperator::Sub => {
                        left_val
                            .checked_sub(*right_val)
                            .ok_or_else(|| RuntimeError {
                                message: format!("Integer overflow: {} - {}", left_val, right_val),
                                instruction_index: 0,
                            })?
                    }
                    BinaryOperator::Mul => {
                        left_val
                            .checked_mul(*right_val)
                            .ok_or_else(|| RuntimeError {
                                message: format!("Integer overflow: {} * {}", left_val, right_val),
                                instruction_index: 0,
                            })?
                    }
                    BinaryOperator::Div => {
                        if *right_val == 0 {
                            return Err(RuntimeError {
                                message: "Division by zero".to_string(),
                                instruction_index: 0,
                            });
                        }
                        left_val
                            .checked_div(*right_val)
                            .ok_or_else(|| RuntimeError {
                                message: format!("Integer overflow: {} / {}", left_val, right_val),
                                instruction_index: 0,
                            })?
                    }
                    BinaryOperator::FloorDiv => {
                        if *right_val == 0 {
                            return Err(RuntimeError {
                                message: "Division by zero".to_string(),
                                instruction_index: 0,
                            });
                        }
                        // Floor division in Python/Rust: rounds toward negative infinity
                        let quot =
                            left_val
                                .checked_div(*right_val)
                                .ok_or_else(|| RuntimeError {
                                    message: format!(
                                        "Integer overflow: {} // {}",
                                        left_val, right_val
                                    ),
                                    instruction_index: 0,
                                })?;
                        let rem = left_val
                            .checked_rem(*right_val)
                            .ok_or_else(|| RuntimeError {
                                message: format!("Integer overflow: {} % {}", left_val, right_val),
                                instruction_index: 0,
                            })?;
                        // Adjust for Python floor division semantics
                        if (rem != 0) && ((left_val < &0) != (right_val < &0)) {
                            quot - 1
                        } else {
                            quot
                        }
                    }
                    BinaryOperator::Mod => {
                        if *right_val == 0 {
                            return Err(RuntimeError {
                                message: "Division by zero".to_string(),
                                instruction_index: 0,
                            });
                        }
                        // Python modulo: result has same sign as divisor
                        let rem = left_val
                            .checked_rem(*right_val)
                            .ok_or_else(|| RuntimeError {
                                message: format!("Integer overflow: {} % {}", left_val, right_val),
                                instruction_index: 0,
                            })?;
                        if (rem != 0) && ((left_val < &0) != (right_val < &0)) {
                            rem + right_val
                        } else {
                            rem
                        }
                    }
                };
                Ok(Value::Integer(result))
            }
        }
    }

    /// Perform a unary operation on the value
    ///
    /// # Arguments
    /// * `op` - The unary operator to apply
    ///
    /// # Returns
    /// * `Ok(Value)` - The result of the operation
    /// * `Err(RuntimeError)` - If the operation fails or is unsupported
    ///
    /// # Errors
    /// * Integer overflow for negation
    /// * Unsupported operation for operators not in Phase 1
    pub fn unary_op(&self, op: UnaryOperator) -> Result<Value, RuntimeError> {
        match self {
            Value::None => Err(RuntimeError {
                message: "Cannot perform unary operation on None".to_string(),
                instruction_index: 0,
            }),
            Value::Integer(val) => match op {
                UnaryOperator::Pos => Ok(Value::Integer(*val)),
                UnaryOperator::Neg => val
                    .checked_neg()
                    .ok_or_else(|| RuntimeError {
                        message: format!("Integer overflow: -{}", val),
                        instruction_index: 0,
                    })
                    .map(Value::Integer),
            },
        }
    }

    /// Extract the integer value
    ///
    /// # Returns
    /// The i64 value if this is an Integer variant
    ///
    /// # Panics
    /// Panics if called on a Value::None variant with the error message:
    /// "Called as_integer on None value: expected Value::Integer but found Value::None.
    /// This indicates a type error in the VM - ensure all operations produce valid Integer values."
    ///
    /// This should not occur during normal Phase 1 operation as all expressions
    /// should produce Integer values. If this panic occurs, it indicates a bug
    /// in the compiler or VM implementation.
    pub fn as_integer(&self) -> i64 {
        match self {
            Value::Integer(val) => *val,
            Value::None => panic!("Called as_integer on None value: expected Value::Integer but found Value::None. This indicates a type error in the VM - ensure all operations produce valid Integer values."),
        }
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Integer(val) => write!(f, "{}", val),
            Value::None => write!(f, ""),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_value_creation() {
        let val = Value::Integer(42);
        assert_eq!(val.as_integer(), 42);
    }

    #[test]
    fn test_display_integer() {
        let val = Value::Integer(42);
        assert_eq!(format!("{}", val), "42");

        let negative = Value::Integer(-100);
        assert_eq!(format!("{}", negative), "-100");

        let zero = Value::Integer(0);
        assert_eq!(format!("{}", zero), "0");
    }

    #[test]
    fn test_binary_op_add() {
        let left = Value::Integer(10);
        let right = Value::Integer(5);
        let result = left.binary_op(BinaryOperator::Add, &right).unwrap();
        assert_eq!(result.as_integer(), 15);
    }

    #[test]
    fn test_binary_op_subtract() {
        let left = Value::Integer(10);
        let right = Value::Integer(5);
        let result = left.binary_op(BinaryOperator::Sub, &right).unwrap();
        assert_eq!(result.as_integer(), 5);
    }

    #[test]
    fn test_binary_op_multiply() {
        let left = Value::Integer(10);
        let right = Value::Integer(5);
        let result = left.binary_op(BinaryOperator::Mul, &right).unwrap();
        assert_eq!(result.as_integer(), 50);
    }

    #[test]
    fn test_binary_op_divide() {
        let left = Value::Integer(10);
        let right = Value::Integer(5);
        let result = left.binary_op(BinaryOperator::Div, &right).unwrap();
        assert_eq!(result.as_integer(), 2);

        // Test negative division
        let left = Value::Integer(-10);
        let right = Value::Integer(5);
        let result = left.binary_op(BinaryOperator::Div, &right).unwrap();
        assert_eq!(result.as_integer(), -2);
    }

    #[test]
    fn test_binary_op_floor_div() {
        let left = Value::Integer(10);
        let right = Value::Integer(3);
        let result = left.binary_op(BinaryOperator::FloorDiv, &right).unwrap();
        assert_eq!(result.as_integer(), 3);

        // Test floor division with negatives (Python semantics)
        let left = Value::Integer(-10);
        let right = Value::Integer(3);
        let result = left.binary_op(BinaryOperator::FloorDiv, &right).unwrap();
        assert_eq!(result.as_integer(), -4); // Python: -10 // 3 = -4

        let left = Value::Integer(10);
        let right = Value::Integer(-3);
        let result = left.binary_op(BinaryOperator::FloorDiv, &right).unwrap();
        assert_eq!(result.as_integer(), -4); // Python: 10 // -3 = -4
    }

    #[test]
    fn test_binary_op_modulo() {
        let left = Value::Integer(10);
        let right = Value::Integer(3);
        let result = left.binary_op(BinaryOperator::Mod, &right).unwrap();
        assert_eq!(result.as_integer(), 1);

        // Test modulo with negatives (Python semantics)
        let left = Value::Integer(-10);
        let right = Value::Integer(3);
        let result = left.binary_op(BinaryOperator::Mod, &right).unwrap();
        assert_eq!(result.as_integer(), 2); // Python: -10 % 3 = 2

        let left = Value::Integer(10);
        let right = Value::Integer(-3);
        let result = left.binary_op(BinaryOperator::Mod, &right).unwrap();
        assert_eq!(result.as_integer(), -2); // Python: 10 % -3 = -2
    }

    #[test]
    fn test_division_by_zero() {
        let left = Value::Integer(10);
        let right = Value::Integer(0);

        // Test division by zero
        let result = left.binary_op(BinaryOperator::Div, &right);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().message, "Division by zero");

        // Test floor division by zero
        let result = left.binary_op(BinaryOperator::FloorDiv, &right);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().message, "Division by zero");

        // Test modulo by zero
        let result = left.binary_op(BinaryOperator::Mod, &right);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().message, "Division by zero");
    }

    #[test]
    fn test_integer_overflow_add() {
        let left = Value::Integer(i64::MAX);
        let right = Value::Integer(1);
        let result = left.binary_op(BinaryOperator::Add, &right);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.message.contains("Integer overflow"));
        assert!(err.message.contains(&i64::MAX.to_string()));
    }

    #[test]
    fn test_integer_overflow_subtract() {
        let left = Value::Integer(i64::MIN);
        let right = Value::Integer(1);
        let result = left.binary_op(BinaryOperator::Sub, &right);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.message.contains("Integer overflow"));
    }

    #[test]
    fn test_integer_overflow_multiply() {
        let left = Value::Integer(i64::MAX);
        let right = Value::Integer(2);
        let result = left.binary_op(BinaryOperator::Mul, &right);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.message.contains("Integer overflow"));
    }

    #[test]
    fn test_integer_overflow_divide() {
        // Special case: i64::MIN / -1 causes overflow
        let left = Value::Integer(i64::MIN);
        let right = Value::Integer(-1);
        let result = left.binary_op(BinaryOperator::Div, &right);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.message.contains("Integer overflow"));
    }

    #[test]
    fn test_unary_op_plus() {
        let val = Value::Integer(42);
        let result = val.unary_op(UnaryOperator::Pos).unwrap();
        assert_eq!(result.as_integer(), 42);

        let negative = Value::Integer(-100);
        let result = negative.unary_op(UnaryOperator::Pos).unwrap();
        assert_eq!(result.as_integer(), -100);
    }

    #[test]
    fn test_unary_op_minus() {
        let val = Value::Integer(42);
        let result = val.unary_op(UnaryOperator::Neg).unwrap();
        assert_eq!(result.as_integer(), -42);

        let negative = Value::Integer(-100);
        let result = negative.unary_op(UnaryOperator::Neg).unwrap();
        assert_eq!(result.as_integer(), 100);

        let zero = Value::Integer(0);
        let result = zero.unary_op(UnaryOperator::Neg).unwrap();
        assert_eq!(result.as_integer(), 0);
    }

    #[test]
    fn test_unary_op_neg_overflow() {
        // i64::MIN cannot be negated without overflow
        let val = Value::Integer(i64::MIN);
        let result = val.unary_op(UnaryOperator::Neg);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.message.contains("Integer overflow"));
        assert!(err.message.contains(&i64::MIN.to_string()));
    }

    #[test]
    fn test_value_equality() {
        let val1 = Value::Integer(42);
        let val2 = Value::Integer(42);
        let val3 = Value::Integer(43);

        assert_eq!(val1, val2);
        assert_ne!(val1, val3);
    }

    #[test]
    fn test_value_clone() {
        let val = Value::Integer(42);
        let cloned = val;
        assert_eq!(val, cloned);
    }

    #[test]
    fn test_complex_expression() {
        // Test: (10 + 5) * 2 - 3
        let ten = Value::Integer(10);
        let five = Value::Integer(5);
        let two = Value::Integer(2);
        let three = Value::Integer(3);

        let sum = ten.binary_op(BinaryOperator::Add, &five).unwrap();
        assert_eq!(sum.as_integer(), 15);

        let product = sum.binary_op(BinaryOperator::Mul, &two).unwrap();
        assert_eq!(product.as_integer(), 30);

        let result = product.binary_op(BinaryOperator::Sub, &three).unwrap();
        assert_eq!(result.as_integer(), 27);
    }

    #[test]
    fn test_negative_operands() {
        let neg_five = Value::Integer(-5);
        let neg_three = Value::Integer(-3);

        // -5 + -3 = -8
        let result = neg_five.binary_op(BinaryOperator::Add, &neg_three).unwrap();
        assert_eq!(result.as_integer(), -8);

        // -5 * -3 = 15
        let result = neg_five.binary_op(BinaryOperator::Mul, &neg_three).unwrap();
        assert_eq!(result.as_integer(), 15);

        // -5 - -3 = -2
        let result = neg_five.binary_op(BinaryOperator::Sub, &neg_three).unwrap();
        assert_eq!(result.as_integer(), -2);
    }

    #[test]
    fn test_zero_operations() {
        let zero = Value::Integer(0);
        let five = Value::Integer(5);

        // 0 + 5 = 5
        let result = zero.binary_op(BinaryOperator::Add, &five).unwrap();
        assert_eq!(result.as_integer(), 5);

        // 0 * 5 = 0
        let result = zero.binary_op(BinaryOperator::Mul, &five).unwrap();
        assert_eq!(result.as_integer(), 0);

        // 0 / 5 = 0
        let result = zero.binary_op(BinaryOperator::Div, &five).unwrap();
        assert_eq!(result.as_integer(), 0);

        // 0 % 5 = 0
        let result = zero.binary_op(BinaryOperator::Mod, &five).unwrap();
        assert_eq!(result.as_integer(), 0);
    }

    #[test]
    fn test_value_copy_trait() {
        // AC1: Verify Value implements Copy trait for zero-cost integer copies
        let original = Value::Integer(42);

        // Copy semantics: assignment creates a copy, not a move
        let copy1 = original;
        let copy2 = original; // Can still use original after copy1

        // All three are independent copies
        assert_eq!(original.as_integer(), 42);
        assert_eq!(copy1.as_integer(), 42);
        assert_eq!(copy2.as_integer(), 42);

        // Verify None variant is also Copy
        let none_val = Value::None;
        let none_copy = none_val;
        assert_eq!(none_val, Value::None);
        assert_eq!(none_copy, Value::None);
    }

    #[test]
    #[should_panic(
        expected = "Called as_integer on None value: expected Value::Integer but found Value::None. This indicates a type error in the VM - ensure all operations produce valid Integer values."
    )]
    fn test_as_integer_panic_on_none() {
        // AC2: Verify as_integer() panics with detailed error message on None
        let none_val = Value::None;
        let _ = none_val.as_integer(); // Should panic with documented message
    }

    #[test]
    fn test_display_none() {
        // Test that None displays as empty string
        let none_val = Value::None;
        assert_eq!(format!("{}", none_val), "");
    }

    #[test]
    fn test_value_none_equality() {
        // Test None equality
        let none1 = Value::None;
        let none2 = Value::None;
        let int_val = Value::Integer(0);

        assert_eq!(none1, none2);
        assert_ne!(none1, int_val);
    }

    #[test]
    fn test_binary_op_with_none() {
        // Test binary operations with None values produce appropriate errors
        let int_val = Value::Integer(5);
        let none_val = Value::None;

        // None on left
        let result = none_val.binary_op(BinaryOperator::Add, &int_val);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().message,
            "Cannot perform binary operation on None"
        );

        // None on right
        let result = int_val.binary_op(BinaryOperator::Add, &none_val);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().message,
            "Cannot perform binary operation on None"
        );

        // None on both sides
        let result = none_val.binary_op(BinaryOperator::Add, &none_val);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().message,
            "Cannot perform binary operation on None"
        );
    }

    #[test]
    fn test_unary_op_with_none() {
        // Test unary operations with None values produce appropriate errors
        let none_val = Value::None;

        let result = none_val.unary_op(UnaryOperator::Neg);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().message,
            "Cannot perform unary operation on None"
        );

        let result = none_val.unary_op(UnaryOperator::Pos);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().message,
            "Cannot perform unary operation on None"
        );
    }
}
