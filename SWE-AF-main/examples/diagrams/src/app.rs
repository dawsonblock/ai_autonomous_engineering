use crate::ascii::AsciiRenderer;
use crate::error::{DiagramError, IoError, Result};
use crate::layout::LayoutEngine;
use crate::lexer::Lexer;
use crate::parser::Parser;
use crate::svg::SvgRenderer;
use crate::types::Diagram;
use crate::validator::Validator;
use std::fs;
use std::path::Path;

/// Application orchestration layer
///
/// Coordinates the full pipeline from file I/O through parsing,
/// validation, layout, and rendering.
pub struct App;

impl App {
    /// Compile DSL file to SVG
    ///
    /// Pipeline: read → lex → parse → validate → layout → render SVG → write
    ///
    /// # Arguments
    /// * `input_path` - Path to DSL source file
    /// * `output_path` - Path where SVG output will be written
    ///
    /// # Returns
    /// * `Ok(())` on success
    /// * `Err(DiagramError::Io)` for file I/O errors
    /// * `Err(DiagramError::Syntax)` for parsing errors
    /// * `Err(DiagramError::Semantic)` for validation errors
    pub fn compile<P: AsRef<Path>>(input_path: P, output_path: P) -> Result<()> {
        // 1. Read input file
        let source = Self::read_file(&input_path)?;

        // 2. Parse to AST
        let diagram = Self::parse_source(&source)?;

        // 3. Validate semantics
        Validator::validate(&diagram)?;

        // 4. Compute layout
        let layout = LayoutEngine::layout(&diagram);

        // 5. Render to SVG
        let svg = SvgRenderer::render(&layout);

        // 6. Write output file
        Self::write_file(&output_path, &svg)?;

        Ok(())
    }

    /// Preview DSL file as ASCII art
    ///
    /// Pipeline: read → lex → parse → validate → layout → render ASCII
    ///
    /// # Arguments
    /// * `input_path` - Path to DSL source file
    ///
    /// # Returns
    /// * `Ok(String)` containing ASCII art representation on success
    /// * `Err(DiagramError::Io)` for file I/O errors
    /// * `Err(DiagramError::Syntax)` for parsing errors
    /// * `Err(DiagramError::Semantic)` for validation errors
    pub fn preview<P: AsRef<Path>>(input_path: P) -> Result<String> {
        // 1. Read input file
        let source = Self::read_file(&input_path)?;

        // 2. Parse to AST
        let diagram = Self::parse_source(&source)?;

        // 3. Validate semantics
        Validator::validate(&diagram)?;

        // 4. Compute layout
        let layout = LayoutEngine::layout(&diagram);

        // 5. Render to ASCII
        let ascii = AsciiRenderer::render(&layout);

        Ok(ascii)
    }

    /// Validate DSL file without generating output
    ///
    /// Pipeline: read → lex → parse → validate
    ///
    /// # Arguments
    /// * `input_path` - Path to DSL source file
    ///
    /// # Returns
    /// * `Ok(())` if DSL is syntactically and semantically valid
    /// * `Err(DiagramError::Io)` for file I/O errors
    /// * `Err(DiagramError::Syntax)` for parsing errors
    /// * `Err(DiagramError::Semantic)` for validation errors
    pub fn validate<P: AsRef<Path>>(input_path: P) -> Result<()> {
        // 1. Read input file
        let source = Self::read_file(&input_path)?;

        // 2. Parse to AST
        let diagram = Self::parse_source(&source)?;

        // 3. Validate semantics
        Validator::validate(&diagram)?;

        Ok(())
    }

    /// Read file contents, converting I/O errors to DiagramError
    fn read_file<P: AsRef<Path>>(path: P) -> Result<String> {
        fs::read_to_string(&path).map_err(|e| {
            DiagramError::Io(IoError {
                message: format!("Failed to read file '{}': {}", path.as_ref().display(), e),
                source: Some(e),
            })
        })
    }

    /// Write file contents, converting I/O errors to DiagramError
    fn write_file<P: AsRef<Path>>(path: P, content: &str) -> Result<()> {
        fs::write(&path, content).map_err(|e| {
            DiagramError::Io(IoError {
                message: format!("Failed to write file '{}': {}", path.as_ref().display(), e),
                source: Some(e),
            })
        })
    }

    /// Parse source string into Diagram AST
    ///
    /// Combines lexer and parser steps
    fn parse_source(source: &str) -> Result<Diagram> {
        let mut lexer = Lexer::new(source);
        let tokens = lexer.tokenize()?;
        let mut parser = Parser::new(tokens);
        parser.parse()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::{NamedTempFile, TempDir};

    // Helper to create a temp file with content
    fn create_temp_file_with_content(content: &str) -> NamedTempFile {
        let mut file = NamedTempFile::new().unwrap();
        file.write_all(content.as_bytes()).unwrap();
        file.flush().unwrap();
        file
    }

    #[test]
    fn test_parse_source_valid_dsl() {
        let source = r#"node "API" as api
node "DB" as db
api -> db : "SQL"
"#;
        let result = App::parse_source(source);
        assert!(result.is_ok());
        let diagram = result.unwrap();
        assert_eq!(diagram.nodes.len(), 2);
        assert_eq!(diagram.connections.len(), 1);
    }

    #[test]
    fn test_parse_source_invalid_syntax() {
        let source = "invalid syntax here";
        let result = App::parse_source(source);
        assert!(result.is_err());
        match result.unwrap_err() {
            DiagramError::Syntax(_) => {}
            _ => panic!("Expected Syntax error"),
        }
    }

    #[test]
    fn test_read_file_success() {
        let file = create_temp_file_with_content("test content");
        let result = App::read_file(file.path());
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "test content");
    }

    #[test]
    fn test_read_file_nonexistent() {
        let result = App::read_file("/nonexistent/path/to/file.dsl");
        assert!(result.is_err());
        match result.unwrap_err() {
            DiagramError::Io(e) => {
                assert!(e.message.contains("Failed to read file"));
                assert!(e.message.contains("/nonexistent/path/to/file.dsl"));
            }
            _ => panic!("Expected IoError"),
        }
    }

    #[test]
    fn test_write_file_success() {
        let temp_dir = TempDir::new().unwrap();
        let output_path = temp_dir.path().join("output.svg");
        let content = "<svg>test</svg>";

        let result = App::write_file(&output_path, content);
        assert!(result.is_ok());

        // Verify file was written
        let written_content = fs::read_to_string(&output_path).unwrap();
        assert_eq!(written_content, content);
    }

    #[test]
    fn test_write_file_unwritable_directory() {
        // Try to write to a directory that doesn't exist
        let result = App::write_file("/nonexistent/directory/file.svg", "content");
        assert!(result.is_err());
        match result.unwrap_err() {
            DiagramError::Io(e) => {
                assert!(e.message.contains("Failed to write file"));
                assert!(e.message.contains("/nonexistent/directory/file.svg"));
            }
            _ => panic!("Expected IoError"),
        }
    }

    #[test]
    fn test_compile_valid_dsl() {
        let dsl_content = r#"node "API Gateway" as api
node "Database" as db
api -> db : "SQL queries"
"#;
        let input_file = create_temp_file_with_content(dsl_content);
        let temp_dir = TempDir::new().unwrap();
        let output_path = temp_dir.path().join("output.svg");

        let result = App::compile(input_file.path(), &output_path);
        assert!(result.is_ok());

        // Verify SVG file was created
        assert!(output_path.exists());

        // Verify SVG content
        let svg_content = fs::read_to_string(&output_path).unwrap();
        assert!(svg_content.starts_with("<svg"));
        assert!(svg_content.contains("API Gateway"));
        assert!(svg_content.contains("Database"));
        assert!(svg_content.contains("SQL queries"));
        assert!(svg_content.contains("</svg>"));
    }

    #[test]
    fn test_compile_invalid_syntax() {
        let dsl_content = "invalid syntax";
        let input_file = create_temp_file_with_content(dsl_content);
        let temp_dir = TempDir::new().unwrap();
        let output_path = temp_dir.path().join("output.svg");

        let result = App::compile(input_file.path(), &output_path);
        assert!(result.is_err());
        match result.unwrap_err() {
            DiagramError::Syntax(_) => {}
            e => panic!("Expected Syntax error, got {:?}", e),
        }
    }

    #[test]
    fn test_compile_semantic_error() {
        let dsl_content = r#"node "API" as api
api -> undefined_node : "bad connection"
"#;
        let input_file = create_temp_file_with_content(dsl_content);
        let temp_dir = TempDir::new().unwrap();
        let output_path = temp_dir.path().join("output.svg");

        let result = App::compile(input_file.path(), &output_path);
        assert!(result.is_err());
        match result.unwrap_err() {
            DiagramError::Semantic(_) => {}
            e => panic!("Expected Semantic error, got {:?}", e),
        }
    }

    #[test]
    fn test_compile_nonexistent_input() {
        let result = App::compile("/nonexistent/input.dsl", "/tmp/output.svg");
        assert!(result.is_err());
        match result.unwrap_err() {
            DiagramError::Io(_) => {}
            e => panic!("Expected IoError, got {:?}", e),
        }
    }

    #[test]
    fn test_preview_valid_dsl() {
        let dsl_content = r#"node "API" as api
node "DB" as db
api -> db
"#;
        let input_file = create_temp_file_with_content(dsl_content);

        let result = App::preview(input_file.path());
        assert!(result.is_ok());

        let ascii = result.unwrap();
        // ASCII output should contain box drawing characters
        assert!(
            ascii.contains('┌')
                || ascii.contains('─')
                || ascii.contains('│')
                || ascii.contains('┐')
                || ascii.contains('└')
                || ascii.contains('┘')
        );
    }

    #[test]
    fn test_preview_invalid_syntax() {
        let dsl_content = "invalid syntax";
        let input_file = create_temp_file_with_content(dsl_content);

        let result = App::preview(input_file.path());
        assert!(result.is_err());
        match result.unwrap_err() {
            DiagramError::Syntax(_) => {}
            e => panic!("Expected Syntax error, got {:?}", e),
        }
    }

    #[test]
    fn test_preview_nonexistent_file() {
        let result = App::preview("/nonexistent/file.dsl");
        assert!(result.is_err());
        match result.unwrap_err() {
            DiagramError::Io(_) => {}
            e => panic!("Expected IoError, got {:?}", e),
        }
    }

    #[test]
    fn test_validate_valid_dsl() {
        let dsl_content = r#"node "Service A" as svc_a
node "Service B" as svc_b
svc_a -> svc_b : "HTTP"
"#;
        let input_file = create_temp_file_with_content(dsl_content);

        let result = App::validate(input_file.path());
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_invalid_syntax() {
        let dsl_content = "node \"Missing as keyword\" identifier";
        let input_file = create_temp_file_with_content(dsl_content);

        let result = App::validate(input_file.path());
        assert!(result.is_err());
        match result.unwrap_err() {
            DiagramError::Syntax(_) => {}
            e => panic!("Expected Syntax error, got {:?}", e),
        }
    }

    #[test]
    fn test_validate_semantic_error_undefined_node() {
        let dsl_content = r#"node "API" as api
api -> missing_node
"#;
        let input_file = create_temp_file_with_content(dsl_content);

        let result = App::validate(input_file.path());
        assert!(result.is_err());
        match result.unwrap_err() {
            DiagramError::Semantic(_) => {}
            e => panic!("Expected Semantic error, got {:?}", e),
        }
    }

    #[test]
    fn test_validate_nonexistent_file() {
        let result = App::validate("/nonexistent/file.dsl");
        assert!(result.is_err());
        match result.unwrap_err() {
            DiagramError::Io(_) => {}
            e => panic!("Expected IoError, got {:?}", e),
        }
    }

    #[test]
    fn test_validate_empty_file() {
        let dsl_content = "";
        let input_file = create_temp_file_with_content(dsl_content);

        let result = App::validate(input_file.path());
        // Empty file should validate successfully (empty diagram is valid)
        assert!(result.is_ok());
    }

    #[test]
    fn test_compile_empty_diagram() {
        let dsl_content = "# Just a comment\n";
        let input_file = create_temp_file_with_content(dsl_content);
        let temp_dir = TempDir::new().unwrap();
        let output_path = temp_dir.path().join("output.svg");

        let result = App::compile(input_file.path(), &output_path);
        assert!(result.is_ok());

        // Verify SVG file was created
        assert!(output_path.exists());
        let svg_content = fs::read_to_string(&output_path).unwrap();
        assert!(svg_content.starts_with("<svg"));
    }

    #[test]
    fn test_preview_empty_diagram() {
        let dsl_content = "\n\n";
        let input_file = create_temp_file_with_content(dsl_content);

        let result = App::preview(input_file.path());
        assert!(result.is_ok());
        // Empty diagram produces empty or minimal ASCII output
        let ascii = result.unwrap();
        assert_eq!(ascii, "");
    }

    // Edge case tests

    #[test]
    fn test_compile_overwrite_existing_output() {
        let dsl_content = r#"node "API" as api"#;
        let input_file = create_temp_file_with_content(dsl_content);
        let temp_dir = TempDir::new().unwrap();
        let output_path = temp_dir.path().join("output.svg");

        // Write initial file
        fs::write(&output_path, "old content").unwrap();

        // Compile should overwrite
        let result = App::compile(input_file.path(), &output_path);
        assert!(result.is_ok());

        let content = fs::read_to_string(&output_path).unwrap();
        assert!(content.contains("<svg"));
        assert!(!content.contains("old content"));
    }

    #[test]
    fn test_validate_whitespace_only() {
        let dsl_content = "   \n\t\n   ";
        let input_file = create_temp_file_with_content(dsl_content);

        let result = App::validate(input_file.path());
        assert!(result.is_ok());
    }

    #[test]
    fn test_preview_with_comments_only() {
        let dsl_content = "# Comment 1\n# Comment 2\n";
        let input_file = create_temp_file_with_content(dsl_content);

        let result = App::preview(input_file.path());
        assert!(result.is_ok());
    }

    #[test]
    fn test_compile_complex_diagram() {
        let dsl_content = r#"node "Service A" as svc_a
node "Service B" as svc_b
node "Service C" as svc_c
node "Database" as db
svc_a -> svc_b : "REST API"
svc_b -> svc_c : "gRPC"
svc_c -> db : "SQL"
svc_a -> db : "Direct query"
"#;
        let input_file = create_temp_file_with_content(dsl_content);
        let temp_dir = TempDir::new().unwrap();
        let output_path = temp_dir.path().join("complex.svg");

        let result = App::compile(input_file.path(), &output_path);
        assert!(result.is_ok());

        let svg_content = fs::read_to_string(&output_path).unwrap();
        assert!(svg_content.contains("Service A"));
        assert!(svg_content.contains("Service B"));
        assert!(svg_content.contains("Service C"));
        assert!(svg_content.contains("Database"));
        assert!(svg_content.contains("REST API"));
        assert!(svg_content.contains("gRPC"));
    }

    #[test]
    fn test_validate_duplicate_node_identifiers() {
        let dsl_content = r#"node "First" as same_id
node "Second" as same_id
"#;
        let input_file = create_temp_file_with_content(dsl_content);

        let result = App::validate(input_file.path());
        // Should fail validation due to duplicate identifiers
        assert!(result.is_err());
        match result.unwrap_err() {
            DiagramError::Semantic(_) => {}
            e => panic!("Expected Semantic error for duplicate IDs, got {:?}", e),
        }
    }

    #[test]
    fn test_compile_output_path_with_unicode() {
        let dsl_content = r#"node "API" as api"#;
        let input_file = create_temp_file_with_content(dsl_content);
        let temp_dir = TempDir::new().unwrap();
        let output_path = temp_dir.path().join("output_测试_файл.svg");

        let result = App::compile(input_file.path(), &output_path);
        assert!(result.is_ok());
        assert!(output_path.exists());
    }

    #[test]
    fn test_preview_returns_string_not_empty_for_valid_diagram() {
        let dsl_content = r#"node "A" as a
node "B" as b
a -> b
"#;
        let input_file = create_temp_file_with_content(dsl_content);

        let result = App::preview(input_file.path());
        assert!(result.is_ok());
        let ascii = result.unwrap();
        // Should produce non-empty ASCII for valid diagram with nodes
        assert!(!ascii.is_empty());
    }

    #[test]
    fn test_compile_writes_complete_svg_structure() {
        let dsl_content = r#"node "Test" as t"#;
        let input_file = create_temp_file_with_content(dsl_content);
        let temp_dir = TempDir::new().unwrap();
        let output_path = temp_dir.path().join("test.svg");

        let result = App::compile(input_file.path(), &output_path);
        assert!(result.is_ok());

        let svg_content = fs::read_to_string(&output_path).unwrap();
        // Verify complete SVG structure (SVG renderer adds newline at end)
        assert!(svg_content.starts_with("<svg"));
        assert!(svg_content.trim_end().ends_with("</svg>"));
        assert!(svg_content.contains("xmlns"));
    }

    #[test]
    fn test_validate_connection_to_self() {
        let dsl_content = r#"node "API" as api
api -> api : "self-reference"
"#;
        let input_file = create_temp_file_with_content(dsl_content);

        let result = App::validate(input_file.path());
        // Self-referencing connections are NOT allowed by the validator
        assert!(result.is_err());
        match result.unwrap_err() {
            DiagramError::Semantic(_) => {}
            e => panic!("Expected Semantic error for self-connection, got {:?}", e),
        }
    }

    #[test]
    fn test_read_file_error_message_contains_path() {
        let nonexistent_path = "/this/path/does/not/exist/file.dsl";
        let result = App::read_file(nonexistent_path);

        assert!(result.is_err());
        if let Err(DiagramError::Io(e)) = result {
            assert!(e.message.contains(nonexistent_path));
            assert!(e.message.contains("Failed to read file"));
        } else {
            panic!("Expected IoError with descriptive message");
        }
    }

    #[test]
    fn test_write_file_error_message_contains_path() {
        let unwritable_path = "/root/cannot/write/here.svg";
        let result = App::write_file(unwritable_path, "test content");

        assert!(result.is_err());
        if let Err(DiagramError::Io(e)) = result {
            assert!(e.message.contains(unwritable_path));
            assert!(e.message.contains("Failed to write file"));
        } else {
            panic!("Expected IoError with descriptive message");
        }
    }
}
