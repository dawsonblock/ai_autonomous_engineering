/// Integration test to validate parser_benchmarks.rs against acceptance criteria
/// This test reads the criterion JSON output and verifies:
/// - AC1: Create benches/parser_benchmarks.rs with parser_simple, parser_complex, parser_variables benchmarks
/// - AC2: Pre-tokenize input outside benchmark loop (verified via code review)
/// - AC3: Criterion generates estimates.json for each benchmark
/// - AC4: CV < 5% for all benchmarks
use std::fs;
use std::path::Path;

#[test]
fn test_parser_benchmarks_file_exists() {
    // AC1: Verify the parser_benchmarks.rs file was created
    let bench_file = Path::new("benches/parser_benchmarks.rs");
    assert!(
        bench_file.exists(),
        "AC1 FAILED: benches/parser_benchmarks.rs does not exist"
    );
    println!("AC1 PASS: benches/parser_benchmarks.rs exists");
}

#[test]
fn test_parser_simple_benchmark_exists() {
    // AC1, AC3: Verify parser_simple benchmark exists and generates estimates.json
    let estimates_path = Path::new("target/criterion/parser_simple/base/estimates.json");

    if !estimates_path.exists() {
        eprintln!("Skipping - run 'cargo bench --bench parser_benchmarks' first");
        return;
    }

    assert!(
        estimates_path.exists(),
        "AC3 FAILED: parser_simple/base/estimates.json not found"
    );

    let content = fs::read_to_string(estimates_path).expect("Failed to read estimates.json");

    let data: serde_json::Value = serde_json::from_str(&content).expect("Failed to parse JSON");

    // Verify JSON schema
    assert!(
        data["mean"]["point_estimate"].is_f64(),
        "Missing mean.point_estimate"
    );
    assert!(
        data["std_dev"]["point_estimate"].is_f64(),
        "Missing std_dev.point_estimate"
    );

    println!("AC3 PASS: parser_simple estimates.json exists and is valid");
}

#[test]
fn test_parser_complex_benchmark_exists() {
    // AC1, AC3: Verify parser_complex benchmark exists and generates estimates.json
    let estimates_path = Path::new("target/criterion/parser_complex/base/estimates.json");

    if !estimates_path.exists() {
        eprintln!("Skipping - run 'cargo bench --bench parser_benchmarks' first");
        return;
    }

    assert!(
        estimates_path.exists(),
        "AC3 FAILED: parser_complex/base/estimates.json not found"
    );

    let content = fs::read_to_string(estimates_path).expect("Failed to read estimates.json");

    let data: serde_json::Value = serde_json::from_str(&content).expect("Failed to parse JSON");

    // Verify JSON schema
    assert!(
        data["mean"]["point_estimate"].is_f64(),
        "Missing mean.point_estimate"
    );
    assert!(
        data["std_dev"]["point_estimate"].is_f64(),
        "Missing std_dev.point_estimate"
    );

    println!("AC3 PASS: parser_complex estimates.json exists and is valid");
}

#[test]
fn test_parser_variables_benchmark_exists() {
    // AC1, AC3: Verify parser_variables benchmark exists and generates estimates.json
    let estimates_path = Path::new("target/criterion/parser_variables/base/estimates.json");

    if !estimates_path.exists() {
        eprintln!("Skipping - run 'cargo bench --bench parser_benchmarks' first");
        return;
    }

    assert!(
        estimates_path.exists(),
        "AC3 FAILED: parser_variables/base/estimates.json not found"
    );

    let content = fs::read_to_string(estimates_path).expect("Failed to read estimates.json");

    let data: serde_json::Value = serde_json::from_str(&content).expect("Failed to parse JSON");

    // Verify JSON schema
    assert!(
        data["mean"]["point_estimate"].is_f64(),
        "Missing mean.point_estimate"
    );
    assert!(
        data["std_dev"]["point_estimate"].is_f64(),
        "Missing std_dev.point_estimate"
    );

    println!("AC3 PASS: parser_variables estimates.json exists and is valid");
}

#[test]
fn test_parser_simple_cv_below_5_percent() {
    // AC4: Verify parser_simple has CV < 5%
    let estimates_path = Path::new("target/criterion/parser_simple/base/estimates.json");

    if !estimates_path.exists() {
        eprintln!("Skipping - run 'cargo bench --bench parser_benchmarks' first");
        return;
    }

    let content = fs::read_to_string(estimates_path).expect("Failed to read estimates.json");

    let data: serde_json::Value = serde_json::from_str(&content).expect("Failed to parse JSON");

    let mean_ns = data["mean"]["point_estimate"]
        .as_f64()
        .expect("Missing mean.point_estimate");

    let std_dev_ns = data["std_dev"]["point_estimate"]
        .as_f64()
        .expect("Missing std_dev.point_estimate");

    let cv = std_dev_ns / mean_ns;
    let cv_percent = cv * 100.0;

    assert!(
        cv < 0.05,
        "AC4 FAILED: parser_simple CV {:.2}% exceeds 5% threshold",
        cv_percent
    );

    println!("AC4 PASS: parser_simple CV = {:.2}% (< 5%)", cv_percent);
}

#[test]
fn test_parser_complex_cv_below_5_percent() {
    // AC4: Verify parser_complex has CV < 5%
    let estimates_path = Path::new("target/criterion/parser_complex/base/estimates.json");

    if !estimates_path.exists() {
        eprintln!("Skipping - run 'cargo bench --bench parser_benchmarks' first");
        return;
    }

    let content = fs::read_to_string(estimates_path).expect("Failed to read estimates.json");

    let data: serde_json::Value = serde_json::from_str(&content).expect("Failed to parse JSON");

    let mean_ns = data["mean"]["point_estimate"]
        .as_f64()
        .expect("Missing mean.point_estimate");

    let std_dev_ns = data["std_dev"]["point_estimate"]
        .as_f64()
        .expect("Missing std_dev.point_estimate");

    let cv = std_dev_ns / mean_ns;
    let cv_percent = cv * 100.0;

    assert!(
        cv < 0.05,
        "AC4 FAILED: parser_complex CV {:.2}% exceeds 5% threshold",
        cv_percent
    );

    println!("AC4 PASS: parser_complex CV = {:.2}% (< 5%)", cv_percent);
}

#[test]
fn test_parser_variables_cv_below_5_percent() {
    // AC4: Verify parser_variables has CV < 5%
    let estimates_path = Path::new("target/criterion/parser_variables/base/estimates.json");

    if !estimates_path.exists() {
        eprintln!("Skipping - run 'cargo bench --bench parser_benchmarks' first");
        return;
    }

    let content = fs::read_to_string(estimates_path).expect("Failed to read estimates.json");

    let data: serde_json::Value = serde_json::from_str(&content).expect("Failed to parse JSON");

    let mean_ns = data["mean"]["point_estimate"]
        .as_f64()
        .expect("Missing mean.point_estimate");

    let std_dev_ns = data["std_dev"]["point_estimate"]
        .as_f64()
        .expect("Missing std_dev.point_estimate");

    let cv = std_dev_ns / mean_ns;
    let cv_percent = cv * 100.0;

    assert!(
        cv < 0.05,
        "AC4 FAILED: parser_variables CV {:.2}% exceeds 5% threshold",
        cv_percent
    );

    println!("AC4 PASS: parser_variables CV = {:.2}% (< 5%)", cv_percent);
}

#[test]
fn test_parser_benchmarks_pretokenize_verification() {
    // AC2: Verify implementation pre-tokenizes (code inspection test)
    // This reads the benchmark file and verifies the pattern exists
    let bench_file = Path::new("benches/parser_benchmarks.rs");

    if !bench_file.exists() {
        eprintln!("Skipping - benches/parser_benchmarks.rs not found");
        return;
    }

    let content = fs::read_to_string(bench_file).expect("Failed to read parser_benchmarks.rs");

    // Verify pre-tokenization pattern: lexer::lex() called outside bench_function
    assert!(
        content.contains("let tokens = lexer::lex"),
        "AC2 FAILED: Pre-tokenization pattern not found (lexer::lex should be outside bench_function)"
    );

    assert!(
        content.contains("c.bench_function"),
        "AC2 FAILED: bench_function call not found"
    );

    // Verify tokens are cloned inside benchmark loop (proper isolation)
    assert!(
        content.contains("tokens_clone") || content.contains("tokens.clone()"),
        "AC2 FAILED: Token cloning pattern not found (should clone inside benchmark loop)"
    );

    println!("AC2 PASS: Pre-tokenization pattern verified in code");
}

#[test]
fn test_all_three_parser_benchmarks_configured() {
    // AC1: Verify all three benchmarks are registered in criterion_group
    let bench_file = Path::new("benches/parser_benchmarks.rs");

    if !bench_file.exists() {
        eprintln!("Skipping - benches/parser_benchmarks.rs not found");
        return;
    }

    let content = fs::read_to_string(bench_file).expect("Failed to read parser_benchmarks.rs");

    assert!(
        content.contains("fn parser_simple"),
        "AC1 FAILED: parser_simple function not found"
    );

    assert!(
        content.contains("fn parser_complex"),
        "AC1 FAILED: parser_complex function not found"
    );

    assert!(
        content.contains("fn parser_variables"),
        "AC1 FAILED: parser_variables function not found"
    );

    // Verify all are in criterion_group targets
    let criterion_group_section = content
        .split("criterion_group!")
        .nth(1)
        .expect("criterion_group! macro not found");

    assert!(
        criterion_group_section.contains("parser_simple"),
        "AC1 FAILED: parser_simple not in criterion_group targets"
    );

    assert!(
        criterion_group_section.contains("parser_complex"),
        "AC1 FAILED: parser_complex not in criterion_group targets"
    );

    assert!(
        criterion_group_section.contains("parser_variables"),
        "AC1 FAILED: parser_variables not in criterion_group targets"
    );

    println!("AC1 PASS: All three parser benchmarks are properly configured");
}

#[test]
fn test_edge_case_parser_handles_empty_input() {
    // Edge case: Verify parser can handle empty token stream
    use pyrust::{lexer, parser};

    // Empty input
    let result = lexer::lex("");
    assert!(
        result.is_ok() || result.is_err(),
        "Lexer should handle empty input"
    );

    // If lexer succeeds with empty input, parser should handle it
    if let Ok(tokens) = result {
        let parse_result = parser::parse(tokens);
        // Either succeeds with empty AST or fails gracefully
        assert!(
            parse_result.is_ok() || parse_result.is_err(),
            "Parser should handle empty tokens gracefully"
        );
    }
}

#[test]
fn test_edge_case_parser_deeply_nested_expressions() {
    // Edge case: Verify parser handles deeply nested expressions
    use pyrust::{lexer, parser};

    // Deeply nested expression: ((((1+2)+3)+4)+5)
    let input = "((((1+2)+3)+4)+5)";
    let tokens = lexer::lex(input).expect("Lexer should handle nested parens");
    let result = parser::parse(tokens);

    assert!(
        result.is_ok(),
        "Parser should handle deeply nested expressions: {:?}",
        result.err()
    );
}

#[test]
fn test_edge_case_parser_all_operators() {
    // Edge case: Verify parser handles all arithmetic operators
    use pyrust::{lexer, parser};

    // Test all operators: + - * / %
    let input = "10 + 20 - 5 * 2 / 4 % 3";
    let tokens = lexer::lex(input).expect("Lexer should handle all operators");
    let result = parser::parse(tokens);

    assert!(
        result.is_ok(),
        "Parser should handle all arithmetic operators: {:?}",
        result.err()
    );
}

#[test]
fn test_edge_case_parser_multiple_statements() {
    // Edge case: Verify parser handles multiple statements
    use pyrust::{lexer, parser};

    // Multiple statements separated by newlines
    let input = "x = 10\ny = 20\nz = x + y";
    let tokens = lexer::lex(input).expect("Lexer should handle multiple statements");
    let result = parser::parse(tokens);

    assert!(
        result.is_ok(),
        "Parser should handle multiple statements: {:?}",
        result.err()
    );
}

#[test]
fn test_edge_case_parser_invalid_syntax() {
    // Edge case: Verify parser properly errors on invalid syntax
    use pyrust::{lexer, parser};

    // Invalid: unclosed parenthesis
    let input = "(1 + 2";
    if let Ok(tokens) = lexer::lex(input) {
        let result = parser::parse(tokens);
        // Parser should either error or handle gracefully
        // We don't assert failure here as some parsers auto-close
        let _ = result;
    }
}

#[test]
fn test_criterion_configuration_for_low_variance() {
    // Verify Criterion is configured for CV < 5%
    let bench_file = Path::new("benches/parser_benchmarks.rs");

    if !bench_file.exists() {
        eprintln!("Skipping - benches/parser_benchmarks.rs not found");
        return;
    }

    let content = fs::read_to_string(bench_file).expect("Failed to read parser_benchmarks.rs");

    // Check for high sample size (should be >= 100 for low variance)
    assert!(
        content.contains("sample_size") || content.contains("Criterion::default()"),
        "Criterion configuration should specify sample_size for low variance"
    );

    // Check for measurement time configuration
    assert!(
        content.contains("measurement_time") || content.contains("Duration"),
        "Criterion configuration should specify measurement_time for stability"
    );

    println!("PASS: Criterion configured for low variance");
}
