//! Cross-conflict integration tests
//!
//! Tests interactions between multiple conflict resolution areas to ensure
//! they work together correctly.
//!
//! Priority: HIGH - Tests compound effects of multiple conflict resolutions
//!
//! Tests interactions between:
//! - daemon_client.rs (closure removal + formatting)
//! - profiling.rs (unsigned_abs + formatting)
//! - vm.rs (body_len removal + formatting)
//! - Cargo.toml/Cargo.lock (PyO3 v0.22 + untracking)

use pyrust::profiling::execute_python_profiled;
use std::fs;
use std::process::Command;

#[test]
fn test_profiling_with_vm_functions_combined() {
    // Test profiling (unsigned_abs) with VM functions (no body_len)
    // This combines two conflict resolution areas
    let code = r#"
def fibonacci(n):
    if n <= 1:
        return n
    return fibonacci(n-1) + fibonacci(n-2)

fibonacci(8)
"#;

    let (output, profile) = execute_python_profiled(code).unwrap();
    assert_eq!(output, "21");

    // Verify profiling timing validation (unsigned_abs from conflict)
    assert!(
        profile.validate_timing_sum(),
        "Profiling validation with VM functions should work"
    );

    // VM execute should have significant time for recursive function
    assert!(profile.vm_execute_ns > 0);
    assert!(profile.compile_ns > 0);
    assert!(profile.parse_ns > 0);
}

#[test]
fn test_formatted_code_with_all_features() {
    // Test that formatted code works across all conflict areas
    let code = r#"
# Test multi-line formatting (issue/06) with:
# - VM functions (issue/03 - no body_len)
# - Profiling (issue/05 - unsigned_abs)

def calculate(
    x,
    y,
    z
):
    result = (
        x * y + z
    )
    return result

calculate(2, 3, 4)
"#;

    let (output, profile) = execute_python_profiled(code).unwrap();
    assert_eq!(output, "10");
    assert!(profile.validate_timing_sum());
}

#[test]
fn test_daemon_client_with_profiled_execution() {
    // Test daemon client (closure removal) with profiling (unsigned_abs)
    use pyrust::daemon_client::DaemonClient;

    // When daemon is not running, should fall back to direct execution
    let result = DaemonClient::execute_or_fallback("2+3");
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "5");

    // Now test the same code with profiling
    let (output, profile) = execute_python_profiled("2+3").unwrap();
    assert_eq!(output, "5");
    assert!(profile.validate_timing_sum());

    // Results should match between daemon fallback and profiled execution
}

#[test]
fn test_all_conflict_areas_with_complex_code() {
    // Comprehensive test using all conflict resolution areas
    let code = r#"
# This tests:
# 1. VM functions without body_len (issue/03)
# 2. Multi-line formatting (issue/06)
# 3. Profiling with unsigned_abs (issue/05)

def factorial(n):
    if n <= 1:
        return 1
    else:
        return n * factorial(n - 1)

def sum_factorials(
    start,
    end
):
    total = 0
    i = start
    while i <= end:
        total = total + factorial(i)
        i = i + 1
    return total

result = sum_factorials(1, 5)
print(result)
"#;

    let (output, profile) = execute_python_profiled(code).unwrap();
    // sum of factorial(1..5) = 1 + 2 + 6 + 24 + 120 = 153
    assert_eq!(output, "153\n");

    // Verify profiling validation
    assert!(profile.validate_timing_sum());

    // All stages should execute
    assert!(profile.lex_ns > 0);
    assert!(profile.parse_ns > 0);
    assert!(profile.compile_ns > 0);
    assert!(profile.vm_execute_ns > 0);
    assert!(profile.format_ns > 0);
}

#[test]
fn test_cargo_build_with_all_changes() {
    // Verify that cargo build works with all conflict resolutions:
    // - PyO3 v0.22 (issue/01)
    // - Cargo.lock untracked (issue/07)
    // - All code changes compiled together
    let output = Command::new("cargo")
        .args(&["build", "--lib", "--quiet"])
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("Failed to run cargo build");

    assert!(
        output.status.success(),
        "Build should succeed with all conflict resolutions"
    );

    // Verify no warnings from clippy issues
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        !stderr.contains("warning"),
        "Should have no warnings after all fixes"
    );
}

#[test]
fn test_daemon_client_error_paths_with_vm_execution() {
    // Test daemon client error handling (direct function pointers)
    // combined with VM execution (no body_len)
    use pyrust::daemon_client::DaemonClient;

    // Error case: division by zero
    let result = DaemonClient::execute_or_fallback("1 / 0");
    assert!(result.is_err(), "Should propagate division error");

    // Success case with function
    let result = DaemonClient::execute_or_fallback("def f():\n    return 42\nf()");
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "42");
}

#[test]
fn test_profiling_format_output_with_vm_functions() {
    // Test profiling format methods with VM function execution
    let code = r#"
def multiply(a, b):
    return a * b

multiply(6, 7)
"#;

    let (output, profile) = execute_python_profiled(code).unwrap();
    assert_eq!(output, "42");

    // Test table formatting
    let table = profile.format_table();
    assert!(table.contains("VM Execute"));
    assert!(table.contains("Compile"));
    assert!(table.contains("TOTAL"));

    // Test JSON formatting
    let json = profile.format_json();
    assert!(json.contains("\"vm_execute_ns\":"));
    assert!(json.contains("\"compile_ns\":"));
}

#[test]
fn test_formatting_consistency_across_modules() {
    // Verify multi-line formatting is consistent across all modified files
    // Check daemon_client.rs formatting
    let daemon_client_path = concat!(env!("CARGO_MANIFEST_DIR"), "/src/daemon_client.rs");
    let daemon_client = fs::read_to_string(daemon_client_path).unwrap();

    // Should have multi-line error handling
    assert!(daemon_client.contains("map_err"));

    // Check profiling.rs formatting
    let profiling_path = concat!(env!("CARGO_MANIFEST_DIR"), "/src/profiling.rs");
    let profiling = fs::read_to_string(profiling_path).unwrap();

    // Should have unsigned_abs
    assert!(profiling.contains("unsigned_abs"));

    // Check vm.rs for body_len: _
    let vm_path = concat!(env!("CARGO_MANIFEST_DIR"), "/src/vm.rs");
    let vm = fs::read_to_string(vm_path).unwrap();

    // Should have body_len: _ pattern
    assert!(vm.contains("body_len: _"));
}

#[test]
fn test_no_dead_code_warnings_after_cleanup() {
    // Verify all dead code has been removed (issues 02, 03, 04, 05)
    let output = Command::new("cargo")
        .args(&["clippy", "--lib", "--", "-D", "warnings"])
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("Failed to run cargo clippy");

    let stderr = String::from_utf8_lossy(&output.stderr);
    let stdout = String::from_utf8_lossy(&output.stdout);

    // Should not have dead_code warnings
    assert!(
        !stderr.contains("dead_code") && !stdout.contains("dead_code"),
        "Should have no dead_code warnings after cleanup"
    );

    // Should not have redundant_closure warnings
    assert!(
        !stderr.contains("redundant_closure") && !stdout.contains("redundant_closure"),
        "Should have no redundant_closure warnings after cleanup"
    );

    // Should not have cast_abs_to_unsigned warnings
    assert!(
        !stderr.contains("cast_abs_to_unsigned") && !stdout.contains("cast_abs_to_unsigned"),
        "Should have no cast warnings after fix"
    );
}

#[test]
fn test_readme_and_license_exist_after_cleanup() {
    // Verify production files exist (issues 09, 10)
    let readme_path = concat!(env!("CARGO_MANIFEST_DIR"), "/README.md");
    assert!(
        std::path::Path::new(readme_path).exists(),
        "README.md should exist after cleanup"
    );

    let license_path = concat!(env!("CARGO_MANIFEST_DIR"), "/LICENSE");
    assert!(
        std::path::Path::new(license_path).exists(),
        "LICENSE should exist after cleanup"
    );
}

#[test]
fn test_no_backup_files_after_cleanup() {
    // Verify all backup files removed (issue/07)
    let output = Command::new("find")
        .args(&[
            ".", "-name", "*.backup", "-o", "-name", "*.tmp", "-o", "-name", "*.bak",
        ])
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("Failed to run find");

    let found_files = String::from_utf8_lossy(&output.stdout);
    assert!(
        found_files.trim().is_empty(),
        "Should have no backup files: {}",
        found_files
    );
}

#[test]
fn test_gitignore_covers_all_artifacts() {
    // Verify .gitignore has all patterns (issue/08)
    let gitignore_path = concat!(env!("CARGO_MANIFEST_DIR"), "/.gitignore");
    let gitignore = fs::read_to_string(gitignore_path).unwrap();

    // Should cover common artifact patterns
    let patterns = vec!["*.log", "target/", "*.swp"];

    for pattern in patterns {
        assert!(
            gitignore.contains(pattern) || gitignore.contains(&pattern.replace("*", "")),
            ".gitignore should contain pattern: {}",
            pattern
        );
    }
}

#[test]
fn test_complete_pipeline_with_all_features() {
    // End-to-end test of complete pipeline with all conflict resolutions
    let test_code = r#"
# Complete pipeline test
def power(base, exp):
    result = 1
    i = 0
    while i < exp:
        result = result * base
        i = i + 1
    return result

def sum_powers(n):
    total = 0
    i = 1
    while i <= n:
        total = total + power(2, i)
        i = i + 1
    return total

result = sum_powers(5)
print(result)
"#;

    // Test with profiling
    let (output, profile) = execute_python_profiled(test_code).unwrap();
    // sum of 2^1 + 2^2 + 2^3 + 2^4 + 2^5 = 2 + 4 + 8 + 16 + 32 = 62
    assert_eq!(output, "62\n");

    // Verify profiling works (unsigned_abs)
    assert!(profile.validate_timing_sum());

    // Test with daemon client fallback
    use pyrust::daemon_client::DaemonClient;
    let daemon_result = DaemonClient::execute_or_fallback(test_code);
    assert!(daemon_result.is_ok());
    assert_eq!(daemon_result.unwrap(), "62\n");

    // Both execution paths should produce same result
}

#[test]
fn test_performance_after_optimizations() {
    // Verify performance is acceptable after all changes
    use std::time::Instant;

    let code = "2 + 2";
    let iterations = 100;

    let start = Instant::now();
    for _ in 0..iterations {
        let _ = execute_python_profiled(code);
    }
    let elapsed = start.elapsed();

    // Should complete 100 iterations in reasonable time
    assert!(
        elapsed.as_millis() < 1000,
        "Performance regression detected: {} ms for {} iterations",
        elapsed.as_millis(),
        iterations
    );
}
