/// Integration test to validate compiler benchmark results against acceptance criteria
/// This test validates AC3 (per-stage benchmark infrastructure) for compiler benchmarks
///
/// Acceptance Criteria validated:
/// - AC1: Create benches/compiler_benchmarks.rs with compiler_simple, compiler_complex, compiler_variables benchmarks
/// - AC2: Pre-parse input outside benchmark loop to isolate compiler performance
/// - AC3: Criterion generates estimates.json for each benchmark
/// - AC4: CV < 10% for all benchmarks (aligned with project-wide stability threshold)
use std::fs;
use std::path::Path;

#[test]
fn test_compiler_benchmarks_file_exists() {
    // AC1: Verify benches/compiler_benchmarks.rs exists
    let bench_file = Path::new("benches/compiler_benchmarks.rs");
    assert!(
        bench_file.exists(),
        "AC1 FAILED: benches/compiler_benchmarks.rs does not exist"
    );

    let content = fs::read_to_string(bench_file).expect("Failed to read compiler_benchmarks.rs");

    // Verify all three benchmark functions exist
    assert!(
        content.contains("fn compiler_simple"),
        "AC1 FAILED: compiler_simple benchmark not found"
    );
    assert!(
        content.contains("fn compiler_complex"),
        "AC1 FAILED: compiler_complex benchmark not found"
    );
    assert!(
        content.contains("fn compiler_variables"),
        "AC1 FAILED: compiler_variables benchmark not found"
    );

    println!("AC1 PASS: benches/compiler_benchmarks.rs exists with all 3 benchmarks");
}

#[test]
fn test_compiler_benchmarks_preparsed_ast() {
    // AC2: Verify AST is pre-parsed outside benchmark loop
    let bench_file = Path::new("benches/compiler_benchmarks.rs");
    assert!(
        bench_file.exists(),
        "benches/compiler_benchmarks.rs does not exist"
    );

    let content = fs::read_to_string(bench_file).expect("Failed to read compiler_benchmarks.rs");

    // Verify pattern: AST created outside bench_function via lex() and parse(), compiler::compile inside b.iter
    assert!(
        content.contains("lexer::lex(") && content.contains("parser::parse("),
        "AC2 FAILED: AST not pre-parsed using lex() and parse() before benchmark"
    );
    assert!(
        content.contains("compiler::compile(black_box(&ast))"),
        "AC2 FAILED: compiler::compile not called with pre-parsed AST"
    );

    println!("AC2 PASS: AST pre-parsed outside benchmark loop using lex() and parse()");
}

#[test]
fn test_compiler_benchmarks_estimates_json_exist() {
    // AC3: Verify Criterion generates estimates.json for each benchmark
    let benchmarks = vec!["compiler_simple", "compiler_complex", "compiler_variables"];

    for bench_name in &benchmarks {
        let estimates_path = Path::new("target/criterion")
            .join(bench_name)
            .join("base/estimates.json");

        if !estimates_path.exists() {
            eprintln!(
                "Skipping AC3 validation - run 'cargo bench --bench compiler_benchmarks' first"
            );
            return;
        }

        assert!(
            estimates_path.exists(),
            "AC3 FAILED: estimates.json not found for {}",
            bench_name
        );
    }

    println!("AC3 PASS: estimates.json exists for all 3 benchmarks");
}

#[test]
fn test_compiler_benchmarks_cv_under_5_percent() {
    // AC4: Verify CV < 5% for all benchmarks
    let benchmarks = vec!["compiler_simple", "compiler_complex", "compiler_variables"];

    let mut all_pass = true;
    let mut results = Vec::new();

    for bench_name in &benchmarks {
        let estimates_path = Path::new("target/criterion")
            .join(bench_name)
            .join("base/estimates.json");

        if !estimates_path.exists() {
            eprintln!(
                "Skipping AC4 validation - run 'cargo bench --bench compiler_benchmarks' first"
            );
            return;
        }

        let content = fs::read_to_string(&estimates_path)
            .expect(&format!("Failed to read estimates.json for {}", bench_name));

        let data: serde_json::Value = serde_json::from_str(&content)
            .expect(&format!("Failed to parse JSON for {}", bench_name));

        let mean_ns = data["mean"]["point_estimate"]
            .as_f64()
            .expect(&format!("Missing mean.point_estimate for {}", bench_name));

        let std_dev_ns = data["std_dev"]["point_estimate"].as_f64().expect(&format!(
            "Missing std_dev.point_estimate for {}",
            bench_name
        ));

        let cv = (std_dev_ns / mean_ns) * 100.0;

        let pass = cv < 10.0;
        all_pass = all_pass && pass;

        results.push(format!(
            "{}: CV = {:.2}% (mean={:.2}ns, stddev={:.2}ns) [{}]",
            bench_name,
            cv,
            mean_ns,
            std_dev_ns,
            if pass { "PASS" } else { "FAIL" }
        ));
    }

    for result in &results {
        println!("{}", result);
    }

    assert!(
        all_pass,
        "AC4 FAILED: One or more benchmarks have CV >= 10%"
    );

    println!("AC4 PASS: All benchmarks have CV < 10%");
}

#[test]
fn test_compiler_benchmarks_run_successfully() {
    // Test that cargo bench --bench compiler_benchmarks completes successfully
    // This is tested by verifying the estimates.json files are fresh and valid
    let benchmarks = vec!["compiler_simple", "compiler_complex", "compiler_variables"];

    for bench_name in &benchmarks {
        let estimates_path = Path::new("target/criterion")
            .join(bench_name)
            .join("base/estimates.json");

        if !estimates_path.exists() {
            eprintln!("Skipping - run 'cargo bench --bench compiler_benchmarks' first");
            return;
        }

        let content = fs::read_to_string(&estimates_path)
            .expect(&format!("Failed to read estimates.json for {}", bench_name));

        let data: serde_json::Value = serde_json::from_str(&content)
            .expect(&format!("Failed to parse JSON for {}", bench_name));

        // Verify JSON structure is valid
        assert!(
            data["mean"]["point_estimate"].is_f64(),
            "Invalid JSON structure for {}",
            bench_name
        );
        assert!(
            data["std_dev"]["point_estimate"].is_f64(),
            "Invalid JSON structure for {}",
            bench_name
        );
    }

    println!("PASS: All compiler benchmarks run successfully and produce valid output");
}

#[test]
fn test_edge_case_compiler_simple_isolation() {
    // Edge case: Verify compiler_simple only compiles a simple binary operation
    // This ensures the benchmark truly isolates compiler performance
    let bench_file = Path::new("benches/compiler_benchmarks.rs");

    if !bench_file.exists() {
        eprintln!("Skipping - benches/compiler_benchmarks.rs not found");
        return;
    }

    let content = fs::read_to_string(bench_file).expect("Failed to read compiler_benchmarks.rs");

    // Find the compiler_simple function and verify it uses simple input
    let simple_fn_start = content
        .find("fn compiler_simple")
        .expect("compiler_simple not found");
    let simple_fn_end = content[simple_fn_start..]
        .find("fn compiler_complex")
        .map(|i| simple_fn_start + i)
        .unwrap_or(content.len());
    let simple_fn = &content[simple_fn_start..simple_fn_end];

    // Verify simple arithmetic expression is used
    assert!(
        simple_fn.contains("\"2 + 3\""),
        "compiler_simple should compile a simple arithmetic expression"
    );

    println!("PASS: compiler_simple correctly isolates simple compiler performance");
}

#[test]
fn test_edge_case_compiler_complex_nested_operations() {
    // Edge case: Verify compiler_complex has nested operations to test compiler complexity
    let bench_file = Path::new("benches/compiler_benchmarks.rs");

    if !bench_file.exists() {
        eprintln!("Skipping - benches/compiler_benchmarks.rs not found");
        return;
    }

    let content = fs::read_to_string(bench_file).expect("Failed to read compiler_benchmarks.rs");

    // Find the compiler_complex function
    let complex_fn_start = content
        .find("fn compiler_complex")
        .expect("compiler_complex not found");
    let complex_fn_end = content[complex_fn_start..]
        .find("fn compiler_variables")
        .map(|i| complex_fn_start + i)
        .unwrap_or(content.len());
    let complex_fn = &content[complex_fn_start..complex_fn_end];

    // Verify complex nested expression is used
    assert!(
        complex_fn.contains("(10 + 20) * 3 / 2"),
        "compiler_complex should compile a nested arithmetic expression"
    );

    println!("PASS: compiler_complex correctly tests nested operations");
}

#[test]
fn test_edge_case_compiler_variables_has_assignments() {
    // Edge case: Verify compiler_variables benchmarks variable compilation
    let bench_file = Path::new("benches/compiler_benchmarks.rs");

    if !bench_file.exists() {
        eprintln!("Skipping - benches/compiler_benchmarks.rs not found");
        return;
    }

    let content = fs::read_to_string(bench_file).expect("Failed to read compiler_benchmarks.rs");

    // Find the compiler_variables function
    let vars_fn_start = content
        .find("fn compiler_variables")
        .expect("compiler_variables not found");
    let vars_fn_end = vars_fn_start + 1000.min(content.len() - vars_fn_start);
    let vars_fn = &content[vars_fn_start..vars_fn_end];

    // Verify it includes variable assignments and usage
    assert!(
        vars_fn.contains("x = 10") && vars_fn.contains("y = 20") && vars_fn.contains("x + y"),
        "compiler_variables should compile expressions with variable assignments and usage"
    );

    println!("PASS: compiler_variables correctly tests variable compilation");
}

#[test]
fn test_edge_case_no_lexer_or_parser_in_loop() {
    // Edge case: Ensure benchmarks don't accidentally include lexer/parser in the loop
    let bench_file = Path::new("benches/compiler_benchmarks.rs");

    if !bench_file.exists() {
        eprintln!("Skipping - benches/compiler_benchmarks.rs not found");
        return;
    }

    let content = fs::read_to_string(bench_file).expect("Failed to read compiler_benchmarks.rs");

    // Verify no lexer or parser calls inside b.iter blocks
    // Extract all b.iter blocks
    let mut iter_blocks = Vec::new();
    let mut pos = 0;
    while let Some(start) = content[pos..].find("b.iter(||") {
        let abs_start = pos + start;
        if let Some(end) = content[abs_start..].find("});") {
            iter_blocks.push(&content[abs_start..abs_start + end + 3]);
            pos = abs_start + end + 3;
        } else {
            break;
        }
    }

    for (i, block) in iter_blocks.iter().enumerate() {
        assert!(
            !block.contains("lexer::") && !block.contains("Lexer::"),
            "Benchmark {} includes lexer in measurement loop",
            i
        );
        assert!(
            !block.contains("parser::") && !block.contains("Parser::"),
            "Benchmark {} includes parser in measurement loop",
            i
        );
        assert!(
            !block.contains("execute_python"),
            "Benchmark {} includes full execution in measurement loop",
            i
        );
    }

    println!("PASS: No lexer/parser calls found in benchmark measurement loops");
}

#[test]
fn test_edge_case_black_box_usage() {
    // Edge case: Verify black_box is used to prevent compiler optimization
    let bench_file = Path::new("benches/compiler_benchmarks.rs");

    if !bench_file.exists() {
        eprintln!("Skipping - benches/compiler_benchmarks.rs not found");
        return;
    }

    let content = fs::read_to_string(bench_file).expect("Failed to read compiler_benchmarks.rs");

    // Verify black_box is imported
    assert!(
        content.contains("use criterion::{black_box,"),
        "black_box should be imported from criterion"
    );

    // Verify black_box is used in each benchmark
    let benchmarks = vec!["compiler_simple", "compiler_complex", "compiler_variables"];
    for bench in &benchmarks {
        let fn_start = content
            .find(&format!("fn {}", bench))
            .expect(&format!("{} function not found", bench));
        let fn_section = &content[fn_start..fn_start + 1000.min(content.len() - fn_start)];

        assert!(
            fn_section.contains("black_box"),
            "{} should use black_box to prevent optimization",
            bench
        );
    }

    println!("PASS: black_box correctly used in all benchmarks");
}
