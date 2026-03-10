use std::fmt;

/// All errors that can occur during Python execution
#[derive(Debug, Clone, PartialEq)]
pub enum PyRustError {
    /// Lexer failed to tokenize input
    LexError(LexError),
    /// Parser failed to build AST
    ParseError(ParseError),
    /// Compiler failed to generate bytecode
    CompileError(CompileError),
    /// Runtime error during execution
    RuntimeError(RuntimeError),
}

/// Lexer error with location information
#[derive(Debug, Clone, PartialEq)]
pub struct LexError {
    pub message: String,
    pub line: usize,
    pub column: usize,
}

/// Parser error with location information
#[derive(Debug, Clone, PartialEq)]
pub struct ParseError {
    pub message: String,
    pub line: usize,
    pub column: usize,
    pub found_token: String,
    pub expected_tokens: Vec<String>,
}

/// Compiler error (should be rare in Phase 1)
#[derive(Debug, Clone, PartialEq)]
pub struct CompileError {
    pub message: String,
}

/// Runtime error during execution
#[derive(Debug, Clone, PartialEq)]
pub struct RuntimeError {
    pub message: String,
    /// Index into bytecode.instructions Vec (NOT byte offset)
    pub instruction_index: usize,
}

impl fmt::Display for PyRustError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PyRustError::LexError(e) => {
                write!(f, "LexError at {}:{}: {}", e.line, e.column, e.message)
            }
            PyRustError::ParseError(e) => write!(
                f,
                "ParseError at {}:{}: {}\n  Found: {}\n  Expected: {}",
                e.line,
                e.column,
                e.message,
                e.found_token,
                e.expected_tokens.join(" | ")
            ),
            PyRustError::CompileError(e) => write!(f, "CompileError: {}", e.message),
            PyRustError::RuntimeError(e) => write!(
                f,
                "RuntimeError at instruction {}: {}",
                e.instruction_index, e.message
            ),
        }
    }
}

impl std::error::Error for PyRustError {}

// Conversion traits for ergonomic error propagation with ? operator
impl From<LexError> for PyRustError {
    fn from(e: LexError) -> Self {
        PyRustError::LexError(e)
    }
}

impl From<ParseError> for PyRustError {
    fn from(e: ParseError) -> Self {
        PyRustError::ParseError(e)
    }
}

impl From<CompileError> for PyRustError {
    fn from(e: CompileError) -> Self {
        PyRustError::CompileError(e)
    }
}

impl From<RuntimeError> for PyRustError {
    fn from(e: RuntimeError) -> Self {
        PyRustError::RuntimeError(e)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lex_error_display() {
        let err = LexError {
            message: "Unexpected character".to_string(),
            line: 1,
            column: 5,
        };
        let display = format!("{}", PyRustError::from(err));
        assert!(display.contains("LexError at 1:5"));
        assert!(display.contains("Unexpected character"));
    }

    #[test]
    fn test_parse_error_display() {
        let err = ParseError {
            message: "Expected expression".to_string(),
            line: 2,
            column: 10,
            found_token: "+".to_string(),
            expected_tokens: vec!["integer".to_string(), "identifier".to_string()],
        };
        let display = format!("{}", PyRustError::from(err));
        assert!(display.contains("ParseError at 2:10"));
        assert!(display.contains("Expected expression"));
        assert!(display.contains("Found: +"));
        assert!(display.contains("integer | identifier"));
    }

    #[test]
    fn test_compile_error_display() {
        let err = CompileError {
            message: "Register overflow".to_string(),
        };
        let display = format!("{}", PyRustError::from(err));
        assert!(display.contains("CompileError"));
        assert!(display.contains("Register overflow"));
    }

    #[test]
    fn test_runtime_error_display() {
        let err = RuntimeError {
            message: "Division by zero".to_string(),
            instruction_index: 42,
        };
        let display = format!("{}", PyRustError::from(err));
        assert!(display.contains("RuntimeError at instruction 42"));
        assert!(display.contains("Division by zero"));
    }

    #[test]
    fn test_error_conversion_traits() {
        let lex_err = LexError {
            message: "test".to_string(),
            line: 1,
            column: 1,
        };
        let _: PyRustError = lex_err.into();
        // Should compile successfully
    }
}
