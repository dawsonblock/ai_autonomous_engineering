use clap::{Parser, Subcommand};
use std::path::PathBuf;

/// Command-line interface for the diagrams tool.
///
/// Provides three subcommands for working with architecture diagrams:
/// - `compile`: Generate SVG output from DSL
/// - `preview`: Show ASCII preview in terminal
/// - `validate`: Check DSL syntax and semantics
#[derive(Parser)]
#[command(name = "diagrams")]
#[command(about = "A CLI tool for generating architecture diagrams from DSL", long_about = None)]
pub struct Cli {
    /// The subcommand to execute
    #[command(subcommand)]
    pub command: Commands,
}

/// Available subcommands for the CLI.
///
/// Each command operates on DSL files and provides different output formats
/// or validation functionality.
#[derive(Subcommand)]
pub enum Commands {
    /// Compile DSL to SVG diagram
    Compile {
        /// Input DSL file path
        input: PathBuf,

        /// Output SVG file path
        #[arg(short, long)]
        output: PathBuf,
    },

    /// Preview DSL as ASCII art in terminal
    Preview {
        /// Input DSL file path
        input: PathBuf,
    },

    /// Validate DSL syntax and semantics
    Validate {
        /// Input DSL file path
        input: PathBuf,
    },
}

impl Cli {
    /// Parse command-line arguments.
    ///
    /// # Returns
    ///
    /// A `Cli` instance with the parsed command and arguments.
    pub fn parse_args() -> Self {
        <Self as Parser>::parse()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::CommandFactory;

    #[test]
    fn test_cli_structure_compiles() {
        // Verify that the Cli struct compiles with clap Parser derive
        let _ = Cli::command();
    }

    #[test]
    fn test_cli_has_three_subcommands() {
        let cmd = Cli::command();
        let subcommands: Vec<_> = cmd.get_subcommands().map(|s| s.get_name()).collect();
        assert_eq!(subcommands.len(), 3);
        assert!(subcommands.contains(&"compile"));
        assert!(subcommands.contains(&"preview"));
        assert!(subcommands.contains(&"validate"));
    }

    #[test]
    fn test_compile_subcommand_has_input_and_output() {
        let cmd = Cli::command();
        let compile_cmd = cmd
            .get_subcommands()
            .find(|s| s.get_name() == "compile")
            .expect("compile subcommand should exist");

        // Check for input positional argument
        let has_input = compile_cmd
            .get_positionals()
            .any(|arg| arg.get_id().as_str() == "input");
        assert!(has_input, "compile should have 'input' positional argument");

        // Check for --output flag
        let has_output = compile_cmd.get_arguments().any(|arg| {
            arg.get_id().as_str() == "output"
                && arg.get_short() == Some('o')
                && arg.get_long() == Some("output")
        });
        assert!(has_output, "compile should have '--output' / '-o' flag");
    }

    #[test]
    fn test_preview_subcommand_has_input() {
        let cmd = Cli::command();
        let preview_cmd = cmd
            .get_subcommands()
            .find(|s| s.get_name() == "preview")
            .expect("preview subcommand should exist");

        let has_input = preview_cmd
            .get_positionals()
            .any(|arg| arg.get_id().as_str() == "input");
        assert!(has_input, "preview should have 'input' positional argument");
    }

    #[test]
    fn test_validate_subcommand_has_input() {
        let cmd = Cli::command();
        let validate_cmd = cmd
            .get_subcommands()
            .find(|s| s.get_name() == "validate")
            .expect("validate subcommand should exist");

        let has_input = validate_cmd
            .get_positionals()
            .any(|arg| arg.get_id().as_str() == "input");
        assert!(
            has_input,
            "validate should have 'input' positional argument"
        );
    }

    #[test]
    fn test_cli_name_is_diagrams() {
        let cmd = Cli::command();
        assert_eq!(cmd.get_name(), "diagrams");
    }

    #[test]
    fn test_cli_has_about_text() {
        let cmd = Cli::command();
        let about = cmd.get_about();
        assert!(about.is_some());
        let about_text = about.unwrap().to_string();
        assert!(about_text.contains("architecture diagrams"));
    }

    #[test]
    fn test_compile_subcommand_description() {
        let cmd = Cli::command();
        let compile_cmd = cmd
            .get_subcommands()
            .find(|s| s.get_name() == "compile")
            .unwrap();
        let about = compile_cmd.get_about();
        assert!(about.is_some());
        let about_text = about.unwrap().to_string();
        assert!(about_text.contains("Compile") || about_text.contains("SVG"));
    }

    #[test]
    fn test_preview_subcommand_description() {
        let cmd = Cli::command();
        let preview_cmd = cmd
            .get_subcommands()
            .find(|s| s.get_name() == "preview")
            .unwrap();
        let about = preview_cmd.get_about();
        assert!(about.is_some());
        let about_text = about.unwrap().to_string();
        assert!(about_text.contains("Preview") || about_text.contains("ASCII"));
    }

    #[test]
    fn test_validate_subcommand_description() {
        let cmd = Cli::command();
        let validate_cmd = cmd
            .get_subcommands()
            .find(|s| s.get_name() == "validate")
            .unwrap();
        let about = validate_cmd.get_about();
        assert!(about.is_some());
        let about_text = about.unwrap().to_string();
        assert!(about_text.contains("Validate") || about_text.contains("syntax"));
    }

    #[test]
    fn test_parse_args_method_exists() {
        // This test verifies that Cli::parse_args() method exists
        // We can't call it directly in tests as it would parse actual CLI args
        // but we can verify it compiles by checking the type
        fn _check_signature() {
            let _: fn() -> Cli = Cli::parse_args;
        }
        _check_signature();
    }
}
