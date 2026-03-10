/// Test suite for binary optimization via LTO and static linking
/// Tests acceptance criteria AC1.1-AC1.5 for issue binary-optimization
use std::fs;
use std::process::Command;

#[test]
fn test_ac1_1_release_build_succeeds() {
    // AC1.1: cargo build --release completes successfully with new [profile.release] configuration

    // Verify Cargo.toml has the release profile
    let cargo_toml = fs::read_to_string("Cargo.toml").expect("Failed to read Cargo.toml");

    assert!(
        cargo_toml.contains("[profile.release]"),
        "Cargo.toml should contain [profile.release] section"
    );
    assert!(
        cargo_toml.contains("lto = \"fat\""),
        "Release profile should have fat LTO"
    );
    assert!(
        cargo_toml.contains("codegen-units = 1"),
        "Release profile should have single codegen unit"
    );
    assert!(
        cargo_toml.contains("strip = true"),
        "Release profile should have symbol stripping"
    );
    assert!(
        cargo_toml.contains("panic = \"abort\""),
        "Release profile should have panic=abort"
    );
    assert!(
        cargo_toml.contains("opt-level = 3"),
        "Release profile should have opt-level=3"
    );

    // Verify .cargo/config.toml exists with target-cpu=native
    let config_toml = fs::read_to_string(".cargo/config.toml")
        .expect("Failed to read .cargo/config.toml - file should exist");

    assert!(
        config_toml.contains("target-cpu=native"),
        ".cargo/config.toml should contain target-cpu=native optimization"
    );
}

#[test]
fn test_ac1_2_binary_size_under_500kb() {
    // AC1.2: Binary size ≤500KB measured via stat command

    let binary_path = "target/release/pyrust";
    let metadata = fs::metadata(binary_path)
        .expect("Release binary should exist - run 'cargo build --release' first");

    let size_bytes = metadata.len();
    let size_kb = size_bytes / 1024;

    println!("Binary size: {} bytes ({} KB)", size_bytes, size_kb);

    assert!(
        size_bytes <= 500_000,
        "Binary size {} bytes ({} KB) exceeds 500KB limit",
        size_bytes,
        size_kb
    );
}

#[test]
fn test_ac1_3_startup_measurement_script_exists() {
    // AC1.3: Binary startup overhead ≤500μs mean measured via hyperfine 100 runs
    // This test verifies the measurement script exists and is executable

    let script_path = "scripts/measure_binary_startup.sh";

    // Check script exists
    assert!(
        fs::metadata(script_path).is_ok(),
        "Measurement script should exist at {}",
        script_path
    );

    // Check script is executable (on Unix)
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let metadata = fs::metadata(script_path).unwrap();
        let permissions = metadata.permissions();
        assert!(
            permissions.mode() & 0o111 != 0,
            "Script should be executable"
        );
    }

    // Verify script contains hyperfine call
    let script_content = fs::read_to_string(script_path).expect("Should be able to read script");

    assert!(
        script_content.contains("hyperfine"),
        "Script should use hyperfine for measurement"
    );
    assert!(
        script_content.contains("--runs 100") || script_content.contains("--runs=100"),
        "Script should run 100 iterations"
    );
    assert!(
        script_content.contains("500"),
        "Script should check against 500μs threshold"
    );
}

#[test]
fn test_ac1_4_binary_executes_correctly() {
    // AC1.4: All 664 currently passing tests still pass after optimization
    // This test verifies the optimized binary still executes correctly

    let output = Command::new("./target/release/pyrust")
        .arg("-c")
        .arg("2+3")
        .output()
        .expect("Failed to execute binary");

    assert!(output.status.success(), "Binary execution should succeed");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_eq!(stdout.trim(), "5", "Binary should produce correct output");
}

#[test]
fn test_ac1_4_complex_expression_execution() {
    // Additional test for AC1.4: Verify more complex expressions work

    let test_cases = vec![
        ("10 + 20", "30"),
        ("100 - 50", "50"),
        ("6 * 7", "42"),
        ("10 / 2", "5"),
        ("10 % 3", "1"),
        ("(1 + 2) * 3", "9"),
    ];

    for (expr, expected) in test_cases {
        let output = Command::new("./target/release/pyrust")
            .arg("-c")
            .arg(expr)
            .output()
            .unwrap_or_else(|_| panic!("Failed to execute: {}", expr));

        assert!(
            output.status.success(),
            "Expression '{}' should execute successfully",
            expr
        );

        let stdout = String::from_utf8_lossy(&output.stdout);
        assert_eq!(
            stdout.trim(),
            expected,
            "Expression '{}' should produce {}",
            expr,
            expected
        );
    }
}

#[test]
fn test_ac1_5_library_api_still_fast() {
    // AC1.5: Library API performance unchanged at 293ns ±10%
    // This is a sanity test - actual verification is done via Criterion benchmarks

    use pyrust::{compiler, lexer, parser, vm::VM};
    use std::time::Instant;

    // Pre-compile the bytecode
    let tokens = lexer::lex("2 + 3").unwrap();
    let ast = parser::parse(tokens).unwrap();
    let bytecode = compiler::compile(&ast).unwrap();

    // Warm up
    for _ in 0..1000 {
        let mut vm = VM::new();
        let _ = vm.execute(&bytecode);
    }

    // Measure
    let iterations = 10000;
    let start = Instant::now();
    for _ in 0..iterations {
        let mut vm = VM::new();
        let _ = vm.execute(&bytecode);
    }
    let elapsed = start.elapsed();

    let avg_ns = elapsed.as_nanos() / iterations as u128;

    println!("Library API execution: {} ns average", avg_ns);

    // Performance should be reasonable (under 1μs in release, under 10μs in debug)
    let threshold = if cfg!(debug_assertions) { 10_000 } else { 1000 };
    assert!(
        avg_ns < threshold,
        "Library API should execute in under {}ns, got {} ns",
        threshold,
        avg_ns
    );
}

#[test]
fn test_optimization_config_completeness() {
    // Verify all optimization settings are properly configured

    let cargo_toml = fs::read_to_string("Cargo.toml").unwrap();
    let config_toml = fs::read_to_string(".cargo/config.toml").unwrap();

    // All required profile.release settings
    let required_settings = vec![
        ("lto", "\"fat\""),
        ("codegen-units", "1"),
        ("strip", "true"),
        ("panic", "\"abort\""),
        ("opt-level", "3"),
    ];

    for (key, value) in required_settings {
        assert!(
            cargo_toml.contains(&format!("{} = {}", key, value)),
            "Missing or incorrect setting: {} = {}",
            key,
            value
        );
    }

    // Verify target-cpu optimization
    assert!(
        config_toml.contains("target-cpu=native"),
        "Missing target-cpu=native in .cargo/config.toml"
    );
}

#[test]
fn test_binary_edge_cases() {
    // Test that optimization doesn't break edge cases

    let test_cases = vec![
        ("", "", true),                        // Empty program
        ("print(42)", "42\n", true),           // Print statement
        ("x = 10\ny = 20\nx + y", "30", true), // Variables
        ("1 / 0", "", false),                  // Division by zero (should fail)
    ];

    for (code, expected_output, should_succeed) in test_cases {
        let output = Command::new("./target/release/pyrust")
            .arg("-c")
            .arg(code)
            .output()
            .unwrap_or_else(|_| panic!("Failed to run: {}", code));

        if should_succeed {
            assert!(output.status.success(), "Code '{}' should succeed", code);
            let stdout = String::from_utf8_lossy(&output.stdout);
            assert_eq!(
                stdout, expected_output,
                "Code '{}' should output '{}'",
                code, expected_output
            );
        } else {
            assert!(!output.status.success(), "Code '{}' should fail", code);
        }
    }
}

#[test]
fn test_strip_reduces_binary_size() {
    // Verify that strip=true actually reduces binary size
    // The optimized binary should be significantly smaller than without stripping

    let binary_path = "target/release/pyrust";
    let metadata = fs::metadata(binary_path).unwrap();
    let size_bytes = metadata.len();

    // With all optimizations, binary should be quite compact
    // Original baseline was 577KB, optimized should be ≤500KB
    assert!(
        size_bytes < 577_000,
        "Optimized binary ({} bytes) should be smaller than original (577,136 bytes)",
        size_bytes
    );
}
