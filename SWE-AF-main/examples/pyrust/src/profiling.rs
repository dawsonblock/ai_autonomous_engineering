use crate::{compiler, error::PyRustError, lexer, parser, vm};
use std::time::Instant;

/// Pipeline profiling data with per-stage nanosecond timings
#[derive(Debug, Clone, Copy, Default)]
pub struct PipelineProfile {
    pub lex_ns: u64,
    pub parse_ns: u64,
    pub compile_ns: u64,
    pub vm_execute_ns: u64,
    pub format_ns: u64,
    pub total_ns: u64,
}

impl PipelineProfile {
    /// Format as human-readable table
    pub fn format_table(&self) -> String {
        let mut output = String::new();
        output.push_str("Stage Breakdown:\n");
        output.push_str("┌──────────────┬──────────┬──────────┐\n");
        output.push_str("│ Stage        │ Time(ns) │ Percent  │\n");
        output.push_str("├──────────────┼──────────┼──────────┤\n");

        let stages = [
            ("Lex", self.lex_ns),
            ("Parse", self.parse_ns),
            ("Compile", self.compile_ns),
            ("VM Execute", self.vm_execute_ns),
            ("Format", self.format_ns),
        ];

        for (name, time_ns) in &stages {
            let percent = if self.total_ns > 0 {
                (*time_ns as f64 / self.total_ns as f64) * 100.0
            } else {
                0.0
            };
            output.push_str(&format!(
                "│ {:<12} │ {:>8} │ {:>6.2}%  │\n",
                name, time_ns, percent
            ));
        }

        output.push_str("├──────────────┼──────────┼──────────┤\n");
        output.push_str(&format!(
            "│ {:<12} │ {:>8} │ {:>6.2}%  │\n",
            "TOTAL", self.total_ns, 100.0
        ));
        output.push_str("└──────────────┴──────────┴──────────┘\n");

        output
    }

    /// Format as JSON matching schema
    pub fn format_json(&self) -> String {
        format!(
            r#"{{
  "lex_ns": {},
  "parse_ns": {},
  "compile_ns": {},
  "vm_execute_ns": {},
  "format_ns": {},
  "total_ns": {}
}}"#,
            self.lex_ns,
            self.parse_ns,
            self.compile_ns,
            self.vm_execute_ns,
            self.format_ns,
            self.total_ns
        )
    }

    /// Validate that sum of stages ≈ total (within 5%)
    /// Used to detect measurement errors or hidden overhead
    pub fn validate_timing_sum(&self) -> bool {
        let sum =
            self.lex_ns + self.parse_ns + self.compile_ns + self.vm_execute_ns + self.format_ns;
        let diff = (sum as i64 - self.total_ns as i64).unsigned_abs();
        let threshold = (self.total_ns as f64 * 0.05) as u64; // 5%
        diff <= threshold
    }
}

/// Execute Python with profiling instrumentation
/// Returns (output, profile) or error
pub fn execute_python_profiled(code: &str) -> Result<(String, PipelineProfile), PyRustError> {
    let mut profile = PipelineProfile::default();
    let start_time = Instant::now();
    let mut last_time = start_time;

    // Stage 1: Lex
    let tokens = lexer::lex(code)?;
    let now = Instant::now();
    profile.lex_ns = now.duration_since(last_time).as_nanos() as u64;
    last_time = now;

    // Stage 2: Parse
    let ast = parser::parse(tokens)?;
    let now = Instant::now();
    profile.parse_ns = now.duration_since(last_time).as_nanos() as u64;
    last_time = now;

    // Stage 3: Compile
    let bytecode = compiler::compile(&ast)?;
    let now = Instant::now();
    profile.compile_ns = now.duration_since(last_time).as_nanos() as u64;
    last_time = now;

    // Stage 4: VM Execute
    let mut vm = vm::VM::new();
    let result = vm.execute(&bytecode)?;
    let now = Instant::now();
    profile.vm_execute_ns = now.duration_since(last_time).as_nanos() as u64;
    last_time = now;

    // Stage 5: Format Output
    let output = vm.format_output(result);
    let now = Instant::now();
    profile.format_ns = now.duration_since(last_time).as_nanos() as u64;

    // Calculate total from beginning
    profile.total_ns = now.duration_since(start_time).as_nanos() as u64;

    Ok((output, profile))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_execute_python_profiled_simple_expression() {
        let (output, profile) = execute_python_profiled("2+3").unwrap();
        assert_eq!(output, "5");

        // Verify all stages have non-zero times
        assert!(profile.lex_ns > 0);
        assert!(profile.parse_ns > 0);
        assert!(profile.compile_ns > 0);
        assert!(profile.vm_execute_ns > 0);
        assert!(profile.format_ns > 0);
        assert!(profile.total_ns > 0);
    }

    #[test]
    fn test_validate_timing_sum() {
        let (_, profile) = execute_python_profiled("2+3").unwrap();
        assert!(
            profile.validate_timing_sum(),
            "Sum of stage timings should be within 5% of total"
        );
    }

    #[test]
    fn test_format_table_contains_all_stages() {
        let (_, profile) = execute_python_profiled("2+3").unwrap();
        let table = profile.format_table();

        assert!(table.contains("Lex"));
        assert!(table.contains("Parse"));
        assert!(table.contains("Compile"));
        assert!(table.contains("VM Execute"));
        assert!(table.contains("Format"));
        assert!(table.contains("TOTAL"));
    }

    #[test]
    fn test_format_json_valid_structure() {
        let (_, profile) = execute_python_profiled("2+3").unwrap();
        let json = profile.format_json();

        assert!(json.contains("\"lex_ns\":"));
        assert!(json.contains("\"parse_ns\":"));
        assert!(json.contains("\"compile_ns\":"));
        assert!(json.contains("\"vm_execute_ns\":"));
        assert!(json.contains("\"format_ns\":"));
        assert!(json.contains("\"total_ns\":"));
    }

    #[test]
    fn test_profiling_with_print_statement() {
        let (output, profile) = execute_python_profiled("print(42)").unwrap();
        assert_eq!(output, "42\n");
        assert!(profile.validate_timing_sum());
    }

    #[test]
    fn test_profiling_with_variable_assignment() {
        let (output, profile) = execute_python_profiled("x = 10\ny = 20\nx + y").unwrap();
        assert_eq!(output, "30");
        assert!(profile.validate_timing_sum());
    }

    #[test]
    fn test_profiling_empty_code() {
        let (output, profile) = execute_python_profiled("").unwrap();
        assert_eq!(output, "");
        // Even empty code should have some minimal timing
        assert!(profile.total_ns > 0);
    }

    #[test]
    fn test_profiling_error_propagation() {
        // Test that errors are properly propagated
        let result = execute_python_profiled("1 / 0");
        assert!(result.is_err());

        let result = execute_python_profiled("x = @");
        assert!(result.is_err());
    }
}
