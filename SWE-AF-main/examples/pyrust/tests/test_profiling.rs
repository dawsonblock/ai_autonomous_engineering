use std::path::PathBuf;
use std::process::Command;

fn get_binary_path() -> PathBuf {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("target");
    path.push("release");
    path.push("pyrust");
    path
}

#[test]
fn test_profiling_table_output_contains_all_stages() {
    // AC5.1: pyrust -c '2+3' --profile outputs table with 5 stage timings in nanoseconds
    let output = Command::new(get_binary_path())
        .args(&["-c", "2+3", "--profile"])
        .output()
        .expect("Failed to execute command");

    // Output should be "5" on stdout
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_eq!(stdout.trim(), "5", "Expected output '5' on stdout");

    // Profile should be on stderr
    let stderr = String::from_utf8_lossy(&output.stderr);

    // Verify all 5 stages are present
    assert!(stderr.contains("Lex"), "Profile should contain Lex stage");
    assert!(
        stderr.contains("Parse"),
        "Profile should contain Parse stage"
    );
    assert!(
        stderr.contains("Compile"),
        "Profile should contain Compile stage"
    );
    assert!(
        stderr.contains("VM Execute"),
        "Profile should contain VM Execute stage"
    );
    assert!(
        stderr.contains("Format"),
        "Profile should contain Format stage"
    );
    assert!(stderr.contains("TOTAL"), "Profile should contain TOTAL");

    // Verify table structure
    assert!(
        stderr.contains("Stage Breakdown:"),
        "Profile should have header"
    );
    assert!(
        stderr.contains("Time(ns)"),
        "Profile should show nanoseconds"
    );
    assert!(
        stderr.contains("Percent"),
        "Profile should show percentages"
    );
}

#[test]
fn test_profiling_stage_timings_sum_validation() {
    // AC5.2: Sum of stage timings within 5% of total measured time
    let output = Command::new(get_binary_path())
        .args(&["-c", "2+3", "--profile"])
        .output()
        .expect("Failed to execute command");

    let stderr = String::from_utf8_lossy(&output.stderr);

    // Parse timings from the table
    let mut stage_times: Vec<u64> = Vec::new();
    let mut total_time: u64 = 0;

    for line in stderr.lines() {
        if line.contains("│") && !line.contains("─") && !line.contains("Stage") {
            // Extract the time value (second column)
            let parts: Vec<&str> = line.split('│').collect();
            if parts.len() >= 3 {
                let time_str = parts[2].trim();
                if let Ok(time) = time_str.parse::<u64>() {
                    if line.contains("TOTAL") {
                        total_time = time;
                    } else {
                        stage_times.push(time);
                    }
                }
            }
        }
    }

    assert_eq!(stage_times.len(), 5, "Should have exactly 5 stage timings");
    assert!(total_time > 0, "Total time should be greater than 0");

    let sum: u64 = stage_times.iter().sum();
    let diff = (sum as i64 - total_time as i64).abs() as u64;
    let threshold = (total_time as f64 * 0.05) as u64; // 5%

    assert!(
        diff <= threshold,
        "Sum of stages ({}) should be within 5% of total ({}). Difference: {}ns, threshold: {}ns",
        sum,
        total_time,
        diff,
        threshold
    );
}

#[test]
fn test_profiling_json_output_valid_schema() {
    // AC5.3: --profile-json outputs valid JSON matching PipelineProfile schema
    let output = Command::new(get_binary_path())
        .args(&["-c", "2+3", "--profile-json"])
        .output()
        .expect("Failed to execute command");

    // Output should be "5" on stdout
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_eq!(stdout.trim(), "5", "Expected output '5' on stdout");

    // JSON should be on stderr
    let stderr = String::from_utf8_lossy(&output.stderr);

    // Parse JSON
    let json: serde_json::Value =
        serde_json::from_str(&stderr).expect("Profile output should be valid JSON");

    // Verify all required fields exist and are non-negative integers
    let required_fields = [
        "lex_ns",
        "parse_ns",
        "compile_ns",
        "vm_execute_ns",
        "format_ns",
        "total_ns",
    ];

    for field in &required_fields {
        assert!(
            json.get(field).is_some(),
            "JSON should contain field '{}'",
            field
        );
        let value = json[field].as_u64().expect(&format!(
            "Field '{}' should be a non-negative integer",
            field
        ));
        assert!(value > 0, "Field '{}' should be greater than 0", field);
    }

    // Verify sum validation (within 5%)
    let lex_ns = json["lex_ns"].as_u64().unwrap();
    let parse_ns = json["parse_ns"].as_u64().unwrap();
    let compile_ns = json["compile_ns"].as_u64().unwrap();
    let vm_execute_ns = json["vm_execute_ns"].as_u64().unwrap();
    let format_ns = json["format_ns"].as_u64().unwrap();
    let total_ns = json["total_ns"].as_u64().unwrap();

    let sum = lex_ns + parse_ns + compile_ns + vm_execute_ns + format_ns;
    let diff = (sum as i64 - total_ns as i64).abs() as u64;
    let threshold = (total_ns as f64 * 0.05) as u64;

    assert!(
        diff <= threshold,
        "Sum of stages ({}) should be within 5% of total ({})",
        sum,
        total_ns
    );
}

#[test]
fn test_profiling_with_print_statement() {
    let output = Command::new(get_binary_path())
        .args(&["-c", "print(42)", "--profile"])
        .output()
        .expect("Failed to execute command");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_eq!(stdout, "42\n", "Print statement should output '42\\n'");

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Lex"), "Profile should be present");
}

#[test]
fn test_profiling_with_variable_assignment() {
    let output = Command::new(get_binary_path())
        .args(&["-c", "x = 10\ny = 20\nx + y", "--profile"])
        .output()
        .expect("Failed to execute command");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_eq!(stdout.trim(), "30", "Expected output '30'");

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Lex"), "Profile should be present");
}

#[test]
fn test_profiling_empty_code() {
    let output = Command::new(get_binary_path())
        .args(&["-c", "", "--profile"])
        .output()
        .expect("Failed to execute command");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_eq!(stdout, "", "Empty code should produce no output");

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("TOTAL"),
        "Profile should still be present for empty code"
    );
}

#[test]
fn test_profiling_error_handling() {
    // Test that profiling works even when code has errors
    let output = Command::new(get_binary_path())
        .args(&["-c", "1 / 0", "--profile"])
        .output()
        .expect("Failed to execute command");

    // Should exit with error
    assert!(!output.status.success(), "Division by zero should fail");

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("Division by zero"),
        "Should show error message"
    );
    // Profile should NOT be present for errors
    assert!(
        !stderr.contains("Stage Breakdown"),
        "Profile should not be shown on error"
    );
}

#[test]
fn test_normal_execution_without_profiling() {
    // Verify normal execution still works
    let output = Command::new(get_binary_path())
        .args(&["-c", "2+3"])
        .output()
        .expect("Failed to execute command");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_eq!(stdout.trim(), "5", "Expected output '5'");

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_eq!(
        stderr.trim(),
        "",
        "Stderr should be empty without --profile"
    );
}

#[test]
fn test_profiling_complex_program() {
    let code = "a = 10\nb = 20\nc = a + b\nprint(c)\nd = c * 2\nprint(d)\nd";
    let output = Command::new(get_binary_path())
        .args(&["-c", code, "--profile"])
        .output()
        .expect("Failed to execute command");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_eq!(
        stdout, "30\n60\n60",
        "Complex program should execute correctly"
    );

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Lex"), "Profile should be present");
    assert!(
        stderr.contains("Parse"),
        "Profile should contain Parse stage"
    );
    assert!(
        stderr.contains("Compile"),
        "Profile should contain Compile stage"
    );
    assert!(
        stderr.contains("VM Execute"),
        "Profile should contain VM Execute stage"
    );
}

#[test]
fn test_profiling_json_vs_table_format() {
    // Run with table format
    let table_output = Command::new(get_binary_path())
        .args(&["-c", "2+3", "--profile"])
        .output()
        .expect("Failed to execute command");

    // Run with JSON format
    let json_output = Command::new(get_binary_path())
        .args(&["-c", "2+3", "--profile-json"])
        .output()
        .expect("Failed to execute command");

    // Both should have same stdout
    assert_eq!(
        String::from_utf8_lossy(&table_output.stdout).trim(),
        String::from_utf8_lossy(&json_output.stdout).trim(),
        "Both formats should produce same program output"
    );

    // Table should have table format
    let table_stderr = String::from_utf8_lossy(&table_output.stderr);
    assert!(
        table_stderr.contains("┌"),
        "Table should have box drawing characters"
    );
    assert!(
        table_stderr.contains("Stage Breakdown"),
        "Table should have header"
    );

    // JSON should be parseable
    let json_stderr = String::from_utf8_lossy(&json_output.stderr);
    let _: serde_json::Value =
        serde_json::from_str(&json_stderr).expect("--profile-json should output valid JSON");
}
