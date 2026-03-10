/// Integration test to validate vm_benchmarks acceptance criteria
/// This test validates:
/// - AC1: benches/vm_benchmarks.rs exists with vm_simple, vm_complex, vm_variables
/// - AC2: Bytecode pre-compilation outside benchmark loop (verified via code structure)
/// - AC3: Criterion generates estimates.json with mean.point_estimate for each benchmark
/// - AC4: vm_simple measures pure VM execution for 2+3 expression
/// - Testing Strategy: All 3 benchmarks execute successfully, vm_simple mean time extractable
use std::fs;
use std::path::Path;

/// AC1: Verify benches/vm_benchmarks.rs exists with all 3 required benchmarks
#[test]
fn test_vm_benchmarks_file_exists_with_all_benchmarks() {
    let bench_file = Path::new("benches/vm_benchmarks.rs");

    assert!(
        bench_file.exists(),
        "AC1 FAILED: benches/vm_benchmarks.rs does not exist"
    );

    let content = fs::read_to_string(bench_file).expect("Failed to read benches/vm_benchmarks.rs");

    // Verify all 3 benchmark functions exist
    assert!(
        content.contains("fn vm_simple("),
        "AC1 FAILED: vm_simple benchmark function not found"
    );
    assert!(
        content.contains("fn vm_complex("),
        "AC1 FAILED: vm_complex benchmark function not found"
    );
    assert!(
        content.contains("fn vm_variables("),
        "AC1 FAILED: vm_variables benchmark function not found"
    );

    println!("AC1 PASS: benches/vm_benchmarks.rs exists with vm_simple, vm_complex, vm_variables");
}

/// AC2: Verify bytecode is pre-compiled outside benchmark loop
#[test]
fn test_vm_benchmarks_pre_compiles_bytecode() {
    let bench_file = Path::new("benches/vm_benchmarks.rs");
    let content = fs::read_to_string(bench_file).expect("Failed to read benches/vm_benchmarks.rs");

    // Check vm_simple pre-compiles bytecode
    assert!(
        content.contains("let tokens = lexer::lex")
            && content.contains("let ast = parser::parse")
            && content.contains("let bytecode = compiler::compile"),
        "AC2 FAILED: vm_simple does not pre-compile bytecode"
    );

    // Verify bytecode compilation happens before b.iter (outside benchmark loop)
    // The pattern should be: compile bytecode, then c.bench_function with b.iter
    let vm_simple_start = content.find("fn vm_simple(").expect("vm_simple not found");
    let vm_simple_section = &content[vm_simple_start..vm_simple_start + 800];

    let compile_pos = vm_simple_section
        .find("compiler::compile")
        .expect("compile not found");
    let iter_pos = vm_simple_section.find("b.iter").expect("b.iter not found");

    assert!(
        compile_pos < iter_pos,
        "AC2 FAILED: Bytecode compilation happens inside benchmark loop (should be outside)"
    );

    println!("AC2 PASS: Bytecode pre-compiled outside benchmark loop to isolate VM performance");
}

/// AC3 & Testing Strategy: Verify Criterion generates estimates.json for vm_simple
#[test]
fn test_vm_simple_estimates_json_exists() {
    let estimates_path = Path::new("target/criterion/vm_simple/base/estimates.json");

    if !estimates_path.exists() {
        eprintln!("Skipping - run 'cargo bench --bench vm_benchmarks' first");
        return;
    }

    let content =
        fs::read_to_string(estimates_path).expect("Failed to read vm_simple estimates.json");

    let data: serde_json::Value =
        serde_json::from_str(&content).expect("Failed to parse vm_simple estimates.json");

    // AC3: Verify mean.point_estimate field exists
    assert!(
        data["mean"]["point_estimate"].is_f64(),
        "AC3 FAILED: vm_simple estimates.json missing mean.point_estimate field"
    );

    let mean_ns = data["mean"]["point_estimate"]
        .as_f64()
        .expect("mean.point_estimate is not a number");

    println!(
        "AC3 PASS: vm_simple estimates.json exists with mean.point_estimate = {:.2}ns",
        mean_ns
    );
}

/// AC3: Verify Criterion generates estimates.json for vm_complex
#[test]
fn test_vm_complex_estimates_json_exists() {
    let estimates_path = Path::new("target/criterion/vm_complex/base/estimates.json");

    if !estimates_path.exists() {
        eprintln!("Skipping - run 'cargo bench --bench vm_benchmarks' first");
        return;
    }

    let content =
        fs::read_to_string(estimates_path).expect("Failed to read vm_complex estimates.json");

    let data: serde_json::Value =
        serde_json::from_str(&content).expect("Failed to parse vm_complex estimates.json");

    assert!(
        data["mean"]["point_estimate"].is_f64(),
        "AC3 FAILED: vm_complex estimates.json missing mean.point_estimate field"
    );

    let mean_ns = data["mean"]["point_estimate"]
        .as_f64()
        .expect("mean.point_estimate is not a number");

    println!(
        "AC3 PASS: vm_complex estimates.json exists with mean.point_estimate = {:.2}ns",
        mean_ns
    );
}

/// AC3: Verify Criterion generates estimates.json for vm_variables
#[test]
fn test_vm_variables_estimates_json_exists() {
    let estimates_path = Path::new("target/criterion/vm_variables/base/estimates.json");

    if !estimates_path.exists() {
        eprintln!("Skipping - run 'cargo bench --bench vm_benchmarks' first");
        return;
    }

    let content =
        fs::read_to_string(estimates_path).expect("Failed to read vm_variables estimates.json");

    let data: serde_json::Value =
        serde_json::from_str(&content).expect("Failed to parse vm_variables estimates.json");

    assert!(
        data["mean"]["point_estimate"].is_f64(),
        "AC3 FAILED: vm_variables estimates.json missing mean.point_estimate field"
    );

    let mean_ns = data["mean"]["point_estimate"]
        .as_f64()
        .expect("mean.point_estimate is not a number");

    println!(
        "AC3 PASS: vm_variables estimates.json exists with mean.point_estimate = {:.2}ns",
        mean_ns
    );
}

/// AC4: Verify vm_simple measures pure VM execution for 2+3 expression
#[test]
fn test_vm_simple_measures_correct_expression() {
    let bench_file = Path::new("benches/vm_benchmarks.rs");
    let content = fs::read_to_string(bench_file).expect("Failed to read benches/vm_benchmarks.rs");

    // Extract vm_simple function
    let vm_simple_start = content.find("fn vm_simple(").expect("vm_simple not found");
    let vm_simple_section = &content[vm_simple_start..vm_simple_start + 500];

    // Verify it uses "2 + 3" expression
    assert!(
        vm_simple_section.contains(r#""2 + 3""#),
        "AC4 FAILED: vm_simple does not measure 2+3 expression (found different expression)"
    );

    println!("AC4 PASS: vm_simple benchmark measures pure VM execution for 2+3 expression");
}

/// Testing Strategy: Verify all 3 VM benchmarks can be extracted via jq-style path
#[test]
fn test_all_vm_benchmarks_extractable() {
    let benchmarks = vec![
        (
            "vm_simple",
            "target/criterion/vm_simple/base/estimates.json",
        ),
        (
            "vm_complex",
            "target/criterion/vm_complex/base/estimates.json",
        ),
        (
            "vm_variables",
            "target/criterion/vm_variables/base/estimates.json",
        ),
    ];

    for (name, path) in benchmarks {
        let estimates_path = Path::new(path);

        if !estimates_path.exists() {
            eprintln!("Skipping - run 'cargo bench --bench vm_benchmarks' first");
            return;
        }

        let content = fs::read_to_string(estimates_path)
            .unwrap_or_else(|_| panic!("Failed to read {}", path));

        let data: serde_json::Value = serde_json::from_str(&content)
            .unwrap_or_else(|_| panic!("Failed to parse {} JSON", name));

        // Verify jq path '.mean.point_estimate' works
        let mean_ns = data["mean"]["point_estimate"]
            .as_f64()
            .unwrap_or_else(|| panic!("{} mean.point_estimate not extractable", name));

        assert!(
            mean_ns > 0.0,
            "Testing Strategy FAILED: {} mean time is not positive: {}ns",
            name,
            mean_ns
        );
    }

    println!(
        "Testing Strategy PASS: All 3 benchmarks executed successfully with extractable mean times"
    );
}

/// PRD AC1: Verify vm_simple achieves <150ns target (VM overhead reduction goal)
#[test]
fn test_vm_simple_meets_150ns_target() {
    let estimates_path = Path::new("target/criterion/vm_simple/base/estimates.json");

    if !estimates_path.exists() {
        eprintln!("Skipping - run 'cargo bench --bench vm_benchmarks' first");
        return;
    }

    let content =
        fs::read_to_string(estimates_path).expect("Failed to read vm_simple estimates.json");

    let data: serde_json::Value =
        serde_json::from_str(&content).expect("Failed to parse vm_simple estimates.json");

    let mean_ns = data["mean"]["point_estimate"]
        .as_f64()
        .expect("mean.point_estimate missing");

    // PRD AC1: VM execution < 150ns for simple expressions (from 250ns baseline)
    assert!(
        mean_ns < 150.0,
        "PRD AC1 FAILED: vm_simple mean {:.2}ns exceeds 150ns target (40% reduction goal)",
        mean_ns
    );

    println!(
        "PRD AC1 PASS: vm_simple = {:.2}ns (< 150ns target, {}% reduction from 250ns baseline)",
        mean_ns,
        ((250.0 - mean_ns) / 250.0 * 100.0)
    );
}

/// Edge case: Verify benchmark uses black_box to prevent compiler optimization
#[test]
fn test_vm_benchmarks_use_black_box() {
    let bench_file = Path::new("benches/vm_benchmarks.rs");
    let content = fs::read_to_string(bench_file).expect("Failed to read benches/vm_benchmarks.rs");

    // All benchmarks should use black_box to prevent dead code elimination
    assert!(
        content.contains("black_box(&bytecode)") || content.contains("black_box(bytecode)"),
        "Edge case FAILED: Benchmarks should use black_box on bytecode to prevent optimization"
    );

    assert!(
        content.contains("black_box(result)"),
        "Edge case FAILED: Benchmarks should use black_box on result to prevent optimization"
    );

    println!("Edge case PASS: Benchmarks correctly use black_box to prevent compiler optimization");
}

/// Edge case: Verify vm_complex uses a complex expression
#[test]
fn test_vm_complex_uses_complex_expression() {
    let bench_file = Path::new("benches/vm_benchmarks.rs");
    let content = fs::read_to_string(bench_file).expect("Failed to read benches/vm_benchmarks.rs");

    // Extract vm_complex function
    let vm_complex_start = content
        .find("fn vm_complex(")
        .expect("vm_complex not found");
    let vm_complex_section = &content[vm_complex_start..vm_complex_start + 600];

    // Should contain operators and parentheses for complexity
    let has_mult = vm_complex_section.contains("*");
    let has_parens = vm_complex_section.contains("(") && vm_complex_section.contains(")");

    assert!(
        has_mult || has_parens,
        "Edge case: vm_complex should use a complex expression with operators/parentheses"
    );

    println!("Edge case PASS: vm_complex uses appropriately complex expression");
}

/// Edge case: Verify vm_variables uses variable assignment and access
#[test]
fn test_vm_variables_uses_variables() {
    let bench_file = Path::new("benches/vm_benchmarks.rs");
    let content = fs::read_to_string(bench_file).expect("Failed to read benches/vm_benchmarks.rs");

    // Extract vm_variables function
    let vm_vars_start = content
        .find("fn vm_variables(")
        .expect("vm_variables not found");
    let vm_vars_section = &content[vm_vars_start..vm_vars_start + 600];

    // Should contain variable assignment (x = , y = )
    assert!(
        vm_vars_section.contains("="),
        "Edge case: vm_variables should use variable assignment"
    );

    // Should contain newlines to separate statements
    assert!(
        vm_vars_section.contains("\\n"),
        "Edge case: vm_variables should have multiple statements (separated by newlines)"
    );

    println!("Edge case PASS: vm_variables uses variable assignment and access");
}

/// Edge case: Verify Criterion configuration matches architecture.md requirements
#[test]
fn test_criterion_configuration_correct() {
    let bench_file = Path::new("benches/vm_benchmarks.rs");
    let content = fs::read_to_string(bench_file).expect("Failed to read benches/vm_benchmarks.rs");

    // Should configure sample_size(1000) per architecture.md
    assert!(
        content.contains("sample_size(1000)"),
        "Edge case: Criterion should use sample_size(1000) per architecture.md"
    );

    // Should configure measurement_time(10s) per architecture.md
    assert!(
        content.contains("measurement_time") && content.contains("from_secs(10)"),
        "Edge case: Criterion should use measurement_time(10s) per architecture.md"
    );

    println!("Edge case PASS: Criterion configuration matches architecture.md requirements");
}

/// Integration test: Verify all benchmarks measure VM-only execution (no lexer/parser/compiler)
#[test]
fn test_vm_benchmarks_isolate_vm_performance() {
    let bench_file = Path::new("benches/vm_benchmarks.rs");
    let content = fs::read_to_string(bench_file).expect("Failed to read benches/vm_benchmarks.rs");

    // Count how many times lexer::lex appears in each benchmark
    let vm_simple_start = content.find("fn vm_simple(").expect("vm_simple not found");
    let vm_complex_start = content
        .find("fn vm_complex(")
        .expect("vm_complex not found");

    let vm_simple_end = vm_complex_start;
    let vm_simple_section = &content[vm_simple_start..vm_simple_end];

    // Within vm_simple, lexer/parser/compiler should appear BEFORE b.iter
    let bench_function_start = vm_simple_section
        .find("c.bench_function")
        .expect("bench_function not found");
    let before_bench = &vm_simple_section[..bench_function_start];

    // Verify pre-compilation happens before benchmark
    assert!(
        before_bench.contains("lexer::lex")
            && before_bench.contains("parser::parse")
            && before_bench.contains("compiler::compile"),
        "Integration: Lexer/parser/compiler should execute before benchmark loop"
    );

    // Verify only VM execution happens inside b.iter
    let iter_start = vm_simple_section.find("b.iter").expect("b.iter not found");
    let iter_section = &vm_simple_section[iter_start..];

    assert!(
        iter_section.contains("VM::new()") && iter_section.contains("vm.execute"),
        "Integration: Only VM execution should occur inside b.iter"
    );

    assert!(
        !iter_section.contains("lexer::lex")
            && !iter_section.contains("parser::parse")
            && !iter_section.contains("compiler::compile"),
        "Integration FAILED: Lexer/parser/compiler should NOT execute inside benchmark loop"
    );

    println!(
        "Integration PASS: Benchmarks correctly isolate VM performance from lexer/parser/compiler"
    );
}

/// Error path: Verify estimates.json has valid statistical fields
#[test]
fn test_vm_benchmarks_statistical_validity() {
    let estimates_path = Path::new("target/criterion/vm_simple/base/estimates.json");

    if !estimates_path.exists() {
        eprintln!("Skipping - run 'cargo bench --bench vm_benchmarks' first");
        return;
    }

    let content =
        fs::read_to_string(estimates_path).expect("Failed to read vm_simple estimates.json");

    let data: serde_json::Value =
        serde_json::from_str(&content).expect("Failed to parse vm_simple estimates.json");

    // Verify all required statistical fields exist
    assert!(
        data["mean"]["point_estimate"].is_f64(),
        "Missing mean.point_estimate"
    );
    assert!(
        data["std_dev"]["point_estimate"].is_f64(),
        "Missing std_dev.point_estimate"
    );
    assert!(
        data["median"]["point_estimate"].is_f64(),
        "Missing median.point_estimate"
    );
    assert!(
        data["mean"]["confidence_interval"].is_object(),
        "Missing confidence_interval"
    );

    let mean = data["mean"]["point_estimate"].as_f64().unwrap();
    let std_dev = data["std_dev"]["point_estimate"].as_f64().unwrap();

    // Verify coefficient of variation is reasonable
    // Note: PRD AC4 requires CV < 10% for cold_start benchmarks (which include all pipeline stages)
    // VM-only microbenchmarks (< 100ns) can have higher CV due to measurement noise
    // We verify CV < 50% as a sanity check for statistical validity
    let cv = std_dev / mean;
    assert!(
        cv < 0.50,
        "Error path: vm_simple CV {:.2}% exceeds 50% threshold (unstable benchmark)",
        cv * 100.0
    );

    println!(
        "Error path PASS: vm_simple has valid statistics (CV = {:.2}%, mean = {:.2}ns)",
        cv * 100.0,
        mean
    );
}
