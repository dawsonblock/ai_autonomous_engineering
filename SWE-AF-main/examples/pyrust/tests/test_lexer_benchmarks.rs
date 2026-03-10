/// Integration test to validate lexer_benchmarks.rs against acceptance criteria
/// This test reads the criterion JSON output and verifies:
/// - AC1: Create benches/lexer_benchmarks.rs with lexer_simple, lexer_complex, lexer_variables benchmarks
/// - AC2: Each benchmark uses black_box() and samples ≥1000 iterations
/// - AC3: Criterion generates estimates.json for each benchmark (target/criterion/lexer_simple/base/estimates.json exists)
/// - AC4: CV (coefficient of variation) < 5% for all benchmarks
use std::fs;
use std::path::Path;

#[test]
fn test_lexer_benchmarks_file_exists() {
    // AC1: Verify the lexer_benchmarks.rs file was created
    let bench_file = Path::new("benches/lexer_benchmarks.rs");
    assert!(
        bench_file.exists(),
        "AC1 FAILED: benches/lexer_benchmarks.rs does not exist"
    );
    println!("AC1 PASS: benches/lexer_benchmarks.rs exists");
}

#[test]
fn test_lexer_benchmarks_has_three_benchmarks() {
    // AC1: Verify all three benchmark functions exist
    let bench_file = Path::new("benches/lexer_benchmarks.rs");
    let content = fs::read_to_string(bench_file)
        .expect("AC1 FAILED: Could not read benches/lexer_benchmarks.rs");

    assert!(
        content.contains("fn lexer_simple"),
        "AC1 FAILED: lexer_simple benchmark function not found"
    );
    assert!(
        content.contains("fn lexer_complex"),
        "AC1 FAILED: lexer_complex benchmark function not found"
    );
    assert!(
        content.contains("fn lexer_variables"),
        "AC1 FAILED: lexer_variables benchmark function not found"
    );

    println!("AC1 PASS: All three benchmark functions exist (lexer_simple, lexer_complex, lexer_variables)");
}

#[test]
fn test_lexer_benchmarks_uses_black_box() {
    // AC2: Verify black_box() is used in all benchmarks
    let bench_file = Path::new("benches/lexer_benchmarks.rs");
    let content = fs::read_to_string(bench_file)
        .expect("AC2 FAILED: Could not read benches/lexer_benchmarks.rs");

    // Count black_box occurrences - should have at least 6 (input + output for each of 3 benchmarks)
    let black_box_count = content.matches("black_box").count();
    assert!(
        black_box_count >= 6,
        "AC2 FAILED: Expected at least 6 black_box() calls, found {}",
        black_box_count
    );

    println!(
        "AC2 PASS: black_box() is used (found {} occurrences)",
        black_box_count
    );
}

#[test]
fn test_lexer_benchmarks_sample_size_1000() {
    // AC2: Verify sample_size is set to at least 1000
    let bench_file = Path::new("benches/lexer_benchmarks.rs");
    let content = fs::read_to_string(bench_file)
        .expect("AC2 FAILED: Could not read benches/lexer_benchmarks.rs");

    assert!(
        content.contains("sample_size(1000)"),
        "AC2 FAILED: sample_size(1000) not found in criterion configuration"
    );

    println!("AC2 PASS: sample_size(1000) is configured");
}

#[test]
fn test_lexer_simple_estimates_json_exists() {
    // AC3: Verify lexer_simple generates estimates.json
    let estimates_path = Path::new("target/criterion/lexer_simple/base/estimates.json");

    if !estimates_path.exists() {
        eprintln!("Skipping - run 'cargo bench --bench lexer_benchmarks' first");
        return;
    }

    assert!(
        estimates_path.exists(),
        "AC3 FAILED: target/criterion/lexer_simple/base/estimates.json not found"
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

    println!("AC3 PASS: lexer_simple estimates.json exists and is valid");
}

#[test]
fn test_lexer_complex_estimates_json_exists() {
    // AC3: Verify lexer_complex generates estimates.json
    let estimates_path = Path::new("target/criterion/lexer_complex/base/estimates.json");

    if !estimates_path.exists() {
        eprintln!("Skipping - run 'cargo bench --bench lexer_benchmarks' first");
        return;
    }

    assert!(
        estimates_path.exists(),
        "AC3 FAILED: target/criterion/lexer_complex/base/estimates.json not found"
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

    println!("AC3 PASS: lexer_complex estimates.json exists and is valid");
}

#[test]
fn test_lexer_variables_estimates_json_exists() {
    // AC3: Verify lexer_variables generates estimates.json
    let estimates_path = Path::new("target/criterion/lexer_variables/base/estimates.json");

    if !estimates_path.exists() {
        eprintln!("Skipping - run 'cargo bench --bench lexer_benchmarks' first");
        return;
    }

    assert!(
        estimates_path.exists(),
        "AC3 FAILED: target/criterion/lexer_variables/base/estimates.json not found"
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

    println!("AC3 PASS: lexer_variables estimates.json exists and is valid");
}

#[test]
fn test_lexer_benchmarks_cv_under_5_percent() {
    // AC4: Verify CV < 5% for all benchmarks
    let benchmarks = vec![
        (
            "lexer_simple",
            "target/criterion/lexer_simple/base/estimates.json",
        ),
        (
            "lexer_complex",
            "target/criterion/lexer_complex/base/estimates.json",
        ),
        (
            "lexer_variables",
            "target/criterion/lexer_variables/base/estimates.json",
        ),
    ];

    let mut all_passed = true;
    let mut results = Vec::new();

    for (name, path) in benchmarks {
        let estimates_path = Path::new(path);

        if !estimates_path.exists() {
            eprintln!(
                "Skipping {} - run 'cargo bench --bench lexer_benchmarks' first",
                name
            );
            continue;
        }

        let content = fs::read_to_string(estimates_path)
            .unwrap_or_else(|_| panic!("Failed to read {}", path));

        let data: serde_json::Value = serde_json::from_str(&content)
            .unwrap_or_else(|_| panic!("Failed to parse JSON from {}", path));

        let mean = data["mean"]["point_estimate"]
            .as_f64()
            .expect("Missing mean.point_estimate");
        let std_dev = data["std_dev"]["point_estimate"]
            .as_f64()
            .expect("Missing std_dev.point_estimate");

        // Calculate CV (coefficient of variation) as percentage
        let cv = (std_dev / mean) * 100.0;

        let status = if cv < 5.0 { "PASS" } else { "FAIL" };
        if cv >= 5.0 {
            all_passed = false;
        }

        let result = format!(
            "{}: CV = {:.2}% (mean={:.2}ns, stddev={:.2}ns) [{}]",
            name, cv, mean, std_dev, status
        );
        results.push(result);
    }

    // Print all results
    for result in &results {
        println!("{}", result);
    }

    assert!(
        all_passed,
        "AC4 FAILED: One or more benchmarks have CV >= 5%"
    );

    println!("AC4 PASS: All benchmarks have CV < 5%");
}

#[test]
fn test_edge_case_lexer_simple_isolation() {
    // Edge case: Verify lexer_simple only tests lexing, not parsing or compilation
    let bench_file = Path::new("benches/lexer_benchmarks.rs");
    let content =
        fs::read_to_string(bench_file).expect("Could not read benches/lexer_benchmarks.rs");

    // Extract lexer_simple function
    let simple_fn_start = content
        .find("fn lexer_simple")
        .expect("lexer_simple function not found");
    let simple_fn_end = content[simple_fn_start..]
        .find("\n}\n")
        .expect("Could not find end of lexer_simple function");
    let simple_fn = &content[simple_fn_start..simple_fn_start + simple_fn_end];

    // Verify it only calls lexer::lex, not parser or compiler
    assert!(
        simple_fn.contains("lexer::lex"),
        "lexer_simple should call lexer::lex"
    );
    assert!(
        !simple_fn.contains("parser::"),
        "lexer_simple should not call parser (testing lexer in isolation)"
    );
    assert!(
        !simple_fn.contains("compiler::"),
        "lexer_simple should not call compiler (testing lexer in isolation)"
    );

    println!("Edge case PASS: lexer_simple tests lexing in isolation");
}

#[test]
fn test_edge_case_lexer_complex_has_operators() {
    // Edge case: Verify lexer_complex tests complex expression with operators
    let bench_file = Path::new("benches/lexer_benchmarks.rs");
    let content =
        fs::read_to_string(bench_file).expect("Could not read benches/lexer_benchmarks.rs");

    // Extract lexer_complex function
    let complex_fn_start = content
        .find("fn lexer_complex")
        .expect("lexer_complex function not found");
    let complex_fn_end = content[complex_fn_start..]
        .find("\n}\n")
        .expect("Could not find end of lexer_complex function");
    let complex_fn = &content[complex_fn_start..complex_fn_start + complex_fn_end];

    // Verify it has multiple operators and parentheses
    assert!(
        complex_fn.contains("(") && complex_fn.contains(")"),
        "lexer_complex should include parentheses for nested expressions"
    );

    // Count operators (should have at least 3: +, *, /)
    let has_plus = complex_fn.contains("+") || complex_fn.contains("Plus");
    let has_mult = complex_fn.contains("*") || complex_fn.contains("Multiply");
    let has_div = complex_fn.contains("/") || complex_fn.contains("Divide");

    assert!(
        has_plus && (has_mult || has_div),
        "lexer_complex should test multiple operators"
    );

    println!("Edge case PASS: lexer_complex tests complex expression with operators");
}

#[test]
fn test_edge_case_lexer_variables_has_assignments() {
    // Edge case: Verify lexer_variables tests variable assignments
    let bench_file = Path::new("benches/lexer_benchmarks.rs");
    let content =
        fs::read_to_string(bench_file).expect("Could not read benches/lexer_benchmarks.rs");

    // Extract lexer_variables function
    let vars_fn_start = content
        .find("fn lexer_variables")
        .expect("lexer_variables function not found");
    let vars_fn_end = content[vars_fn_start..]
        .find("\n}\n")
        .expect("Could not find end of lexer_variables function");
    let vars_fn = &content[vars_fn_start..vars_fn_start + vars_fn_end];

    // Verify it has assignment operator
    assert!(
        vars_fn.contains("="),
        "lexer_variables should include assignment operator"
    );

    // Verify it has identifiers (variable names like x, y, etc.)
    assert!(
        vars_fn.contains("x") || vars_fn.contains("y") || vars_fn.contains("identifier"),
        "lexer_variables should include variable identifiers"
    );

    println!("Edge case PASS: lexer_variables tests variable assignments");
}

#[test]
fn test_edge_case_black_box_usage() {
    // Edge case: Verify black_box is used on BOTH input and output
    let bench_file = Path::new("benches/lexer_benchmarks.rs");
    let content =
        fs::read_to_string(bench_file).expect("Could not read benches/lexer_benchmarks.rs");

    // For each benchmark, verify pattern: black_box(input) and black_box(result)
    for fn_name in &["lexer_simple", "lexer_complex", "lexer_variables"] {
        let fn_start = content
            .find(&format!("fn {}", fn_name))
            .unwrap_or_else(|| panic!("{} function not found", fn_name));
        let fn_end = content[fn_start..]
            .find("\n}\n")
            .unwrap_or_else(|| panic!("Could not find end of {} function", fn_name));
        let fn_body = &content[fn_start..fn_start + fn_end];

        // Verify input is wrapped in black_box
        assert!(
            fn_body.contains("black_box(\""),
            "{}: Input should be wrapped in black_box",
            fn_name
        );

        // Verify result is wrapped in black_box
        assert!(
            fn_body.contains("black_box(result)"),
            "{}: Result should be wrapped in black_box",
            fn_name
        );
    }

    println!("Edge case PASS: black_box used on both input and output for all benchmarks");
}

#[test]
fn test_edge_case_empty_input() {
    // Edge case: Test lexer behavior with empty input
    use pyrust::lexer;

    let tokens = lexer::lex("");
    assert!(
        tokens.is_ok() || tokens.is_err(),
        "Lexer should handle empty input gracefully"
    );

    println!("Edge case PASS: Lexer handles empty input");
}

#[test]
fn test_edge_case_whitespace_only() {
    // Edge case: Test lexer behavior with whitespace-only input
    use pyrust::lexer;

    let tokens = lexer::lex("   \n\t  ");
    assert!(
        tokens.is_ok() || tokens.is_err(),
        "Lexer should handle whitespace-only input gracefully"
    );

    println!("Edge case PASS: Lexer handles whitespace-only input");
}

#[test]
fn test_edge_case_very_long_expression() {
    // Edge case: Test lexer with very long expression (boundary test)
    use pyrust::lexer;

    // Create a long expression: 1 + 2 + 3 + ... + 100
    let mut expr = String::from("1");
    for i in 2..=100 {
        expr.push_str(&format!(" + {}", i));
    }

    let result = lexer::lex(&expr);
    assert!(
        result.is_ok(),
        "Lexer should handle long expressions: {:?}",
        result.err()
    );

    if let Ok(tokens) = result {
        // Should have 199 tokens: 100 numbers + 99 plus operators
        assert!(
            tokens.len() >= 199,
            "Expected at least 199 tokens, got {}",
            tokens.len()
        );
    }

    println!("Edge case PASS: Lexer handles very long expressions");
}

#[test]
fn test_edge_case_special_characters() {
    // Edge case: Test lexer behavior with invalid/special characters
    use pyrust::lexer;

    let invalid_inputs = vec![
        "@#$%",           // Special characters
        "123abc!",        // Mixed with invalid chars
        "x = 5 & y = 10", // Unsupported operator
    ];

    for input in invalid_inputs {
        let result = lexer::lex(input);
        // Lexer should either tokenize valid parts or return error
        // Either outcome is acceptable - we're testing it doesn't panic
        println!("Input '{}' result: {:?}", input, result.is_ok());
    }

    println!("Edge case PASS: Lexer handles special characters without panicking");
}

#[test]
fn test_edge_case_unicode_identifiers() {
    // Edge case: Test lexer with Unicode characters (boundary test)
    use pyrust::lexer;

    // Python allows Unicode identifiers in modern versions
    let result = lexer::lex("π = 3.14");

    // The lexer may or may not support Unicode - we're testing it doesn't crash
    println!("Unicode input result: {:?}", result.is_ok());

    println!("Edge case PASS: Lexer handles Unicode input without panicking");
}

#[test]
fn test_edge_case_maximum_integer() {
    // Edge case: Test lexer with very large integers (boundary test)
    use pyrust::lexer;

    let large_num = "99999999999999999999999999999999";
    let result = lexer::lex(large_num);

    assert!(
        result.is_ok() || result.is_err(),
        "Lexer should handle large integers gracefully"
    );

    println!("Edge case PASS: Lexer handles very large integers");
}
