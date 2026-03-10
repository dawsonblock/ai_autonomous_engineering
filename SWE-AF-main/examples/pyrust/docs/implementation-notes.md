# Forward Reference Validation Implementation

## Summary
This issue implemented HashSet-based validation in `compiler.rs` to reject function calls before function definitions.

## Implementation Details

### Location
- File: `src/compiler.rs`
- Lines: 315-411

### Key Components

1. **`validate_no_forward_references()` (lines 317-338)**
   - Validates that a statement doesn't contain forward references
   - Checks Expression, Assignment, Print, and Return statements
   - Delegates to `check_expression_for_forward_references()` for expression validation

2. **`check_expression_for_forward_references()` (lines 341-374)**
   - Recursively checks expressions for forward function references
   - Detects when a function call references a function that will be defined later
   - Returns `CompileError` with descriptive message
   - Handles nested expressions (BinaryOp, UnaryOp, Call)

3. **Integration in `compile_program()` (lines 377-411)**
   - First pass: Collects all function names that will be defined (`all_defined_functions`)
   - Second pass: Validates statements in order
   - Tracks functions defined so far (`defined_so_far`)
   - For each statement:
     - If FunctionDef: adds to `defined_so_far` BEFORE validating body (allows recursion)
     - Otherwise: validates against forward references before processing

### Algorithm

```
1. Create HashSet of all function names that will be defined
2. Create empty HashSet for functions defined so far
3. For each statement in order:
   a. If function definition:
      - Add function name to defined_so_far
      - Validate function body (allows recursion since name already added)
   b. Otherwise:
      - Validate statement doesn't call functions defined later
      - Error if: function in all_defined_functions but NOT in defined_so_far
```

## Test Coverage

### Primary Test
- `test_function_calling_before_definition` (tests/test_functions.rs:777)
- Verifies that calling a function before its definition returns CompileError

### Test Case
```python
foo()           # Error: forward reference
def foo():
    return 42
```

### Expected Behavior
- Returns: `CompileError: Call to undefined function 'foo' (function defined later in program)`
- Test assertion: `assert!(result.is_err())`
- Status: ✅ PASSING

## Acceptance Criteria Status

✅ **AC4.3**: test_function_calling_before_definition returns CompileError
- Test passes (correctly detects and rejects forward reference)
- CompileError message: "Call to undefined function '{}' (function defined later in program)"

✅ **AC4.2**: All 462 currently passing lib tests still pass
- `cargo test --lib --release` shows 462 passed, 0 failed

## Edge Cases Handled

1. **Recursion**: ✅ Allowed (function name added to defined_so_far BEFORE body validation)
2. **Forward references in function bodies**: ✅ Detected (body validated with current defined_so_far set)
3. **Nested expressions**: ✅ Recursively validated (BinaryOp, UnaryOp, Call)
4. **Multiple statements**: ✅ Order-dependent validation (defined_so_far updated incrementally)

## Example Error Output

```bash
$ ./target/release/pyrust -c "foo()\ndef foo():\n    return 42"
CompileError: Call to undefined function 'foo' (function defined later in program)
```

## Implementation Complete
- Code: ✅ Implemented
- Tests: ✅ Passing
- Documentation: ✅ This file
- No regressions: ✅ All tests pass
