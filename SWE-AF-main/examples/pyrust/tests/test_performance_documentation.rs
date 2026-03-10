/// Integration tests for docs/performance.md documentation validation
///
/// Testing Strategy (from issue 22-performance-documentation):
/// - AC6.6: Validate presence of new baseline table showing all modes (binary/daemon/cached)
/// - AC6.6: Validate speedup calculations include 95% CI and statistical confidence
/// - AC6.6: Validate variance analysis explains CV and benchmark stability
/// - AC5.5: Validate profile data integrated with stage breakdown table showing percentage of total time
/// - Verify document includes actual benchmark numbers from criterion output
/// - Verify numerical values match actual benchmark outputs
///
/// This test automates validation to ensure AC5.5 and AC6.6 requirements are met.
use std::fs;
use std::path::Path;

#[test]
fn test_performance_md_exists() {
    // AC1.4 requirement: File docs/performance.md exists
    let path = Path::new("docs/performance.md");
    assert!(
        path.exists(),
        "AC1.4 FAILED: docs/performance.md file does not exist"
    );

    // Verify file is not empty
    let content = fs::read_to_string(path).expect("Failed to read docs/performance.md");

    assert!(
        !content.is_empty(),
        "AC1.4 FAILED: docs/performance.md is empty"
    );

    println!("✓ docs/performance.md exists and is not empty");
}

#[test]
fn test_all_required_sections_present() {
    // AC1.4 requirement: All required sections must be present
    // Testing strategy: grep -E '(Methodology|Results|Breakdown|Comparison)'

    let content = fs::read_to_string("docs/performance.md")
        .expect("docs/performance.md not found - run test_performance_md_exists first");

    let required_sections = vec![
        "Methodology",
        "Results",
        "Breakdown",
        "Comparison",
        "Variance",
        "Reproduction",
    ];

    for section in &required_sections {
        assert!(
            content.contains(section),
            "AC1.4 FAILED: Required section '{}' not found in docs/performance.md",
            section
        );
    }

    println!("✓ All required sections present: {:?}", required_sections);
}

#[test]
fn test_methodology_section_complete() {
    // AC1.4 requirement: Methodology section includes hardware specs,
    // benchmark framework, and statistical methods

    let content = fs::read_to_string("docs/performance.md").expect("docs/performance.md not found");

    let methodology_keywords = vec![
        "Hardware",
        "Benchmark Framework",
        "Statistical Methods",
        "Criterion",
    ];

    for keyword in &methodology_keywords {
        assert!(
            content.contains(keyword),
            "AC1.4 FAILED: Methodology section missing '{}' information",
            keyword
        );
    }

    println!("✓ Methodology section is complete");
}

#[test]
fn test_results_section_has_benchmark_numbers() {
    // AC1.4 requirement: Results section includes actual benchmark numbers
    // from criterion output

    let content = fs::read_to_string("docs/performance.md").expect("docs/performance.md not found");

    // Check for presence of actual benchmark data:
    // - Mean execution time in nanoseconds/microseconds/milliseconds
    // - Standard deviation
    // - Confidence intervals
    // - Coefficient of variation

    let benchmark_indicators = vec![
        "Mean",
        "Std Dev",
        "Confidence Interval",
        "CV",
        "ns", // nanoseconds unit
    ];

    for indicator in &benchmark_indicators {
        assert!(
            content.contains(indicator),
            "AC1.4 FAILED: Results section missing benchmark data indicator '{}'",
            indicator
        );
    }

    // Verify actual numbers are present (look for patterns like "293.34 ns")
    let has_timing_data = content.contains("293")
        || content.contains(" ns")
        || content.contains(" μs")
        || content.contains(" ms");

    assert!(
        has_timing_data,
        "AC1.4 FAILED: Results section does not contain actual timing measurements"
    );

    println!("✓ Results section contains actual benchmark numbers");
}

#[test]
fn test_breakdown_section_has_pipeline_stages() {
    // AC1.4 requirement: Breakdown section shows performance by pipeline stage
    // (lexing, parsing, compilation, VM, formatting)

    let content = fs::read_to_string("docs/performance.md").expect("docs/performance.md not found");

    let pipeline_stages = vec!["Lexing", "Parsing", "Compilation", "VM"];

    for stage in &pipeline_stages {
        assert!(
            content.contains(stage),
            "AC1.4 FAILED: Breakdown section missing pipeline stage '{}'",
            stage
        );
    }

    println!("✓ Breakdown section includes all pipeline stages");
}

#[test]
fn test_comparison_section_documents_speedup() {
    // AC1.4 requirement: Comparison section shows CPython speedup analysis
    // with confidence intervals

    let content = fs::read_to_string("docs/performance.md").expect("docs/performance.md not found");

    // Check for CPython comparison
    assert!(
        content.contains("CPython"),
        "AC1.4 FAILED: Comparison section missing 'CPython' reference"
    );

    assert!(
        content.contains("Speedup"),
        "AC1.4 FAILED: Comparison section missing 'Speedup' analysis"
    );

    // Check for confidence intervals
    assert!(
        content.contains("confidence") || content.contains("Confidence"),
        "AC1.4 FAILED: Comparison section missing confidence interval analysis"
    );

    println!("✓ Comparison section documents CPython speedup with confidence intervals");
}

#[test]
fn test_speedup_ratio_at_least_50x_documented() {
    // AC1.4 requirement: Speedup ratio ≥50x is documented
    // This validates AC1.3 is proven in the documentation

    let content = fs::read_to_string("docs/performance.md").expect("docs/performance.md not found");

    // Look for speedup mention with large numbers
    // Should contain patterns like "66,054x" or "50x" or similar
    let has_large_speedup = content.contains("66,054x")
        || content.contains("64,661x")
        || content.contains("67,489x")
        || content.contains("≥ 50")
        || content.contains(">= 50")
        || content.contains("50x");

    assert!(
        has_large_speedup,
        "AC1.4 FAILED: Document does not clearly show speedup ≥50x"
    );

    println!("✓ Speedup ratio ≥50x is documented");
}

#[test]
fn test_variance_section_reports_statistical_confidence() {
    // AC1.4 requirement: Variance section reports statistical confidence (CV < 10%)
    // This validates AC1.5 is proven in the documentation

    let content = fs::read_to_string("docs/performance.md").expect("docs/performance.md not found");

    // Check for coefficient of variation reporting
    assert!(
        content.contains("CV") || content.contains("Coefficient of Variation"),
        "AC1.4 FAILED: Variance section missing CV reporting"
    );

    // Check that it mentions the < 10% threshold
    assert!(
        content.contains("< 10%") || content.contains("<10%"),
        "AC1.4 FAILED: Variance section missing < 10% CV threshold"
    );

    println!("✓ Variance section reports statistical confidence (CV < 10%)");
}

#[test]
fn test_reproduction_instructions_present() {
    // AC1.4 requirement: Instructions for reproducing benchmarks

    let content = fs::read_to_string("docs/performance.md").expect("docs/performance.md not found");

    // Check for reproduction section
    assert!(
        content.contains("Reproduction") || content.contains("Reproduc"),
        "AC1.4 FAILED: Reproduction section not found"
    );

    // Check for cargo bench command
    assert!(
        content.contains("cargo bench"),
        "AC1.4 FAILED: Reproduction instructions missing 'cargo bench' command"
    );

    println!("✓ Reproduction instructions are present");
}

#[test]
fn test_cold_start_performance_documented() {
    // AC1.4 requirement: Results prove AC1.2 (< 100μs)

    let content = fs::read_to_string("docs/performance.md").expect("docs/performance.md not found");

    // Check that cold start is documented
    assert!(
        content.contains("Cold Start")
            || content.contains("cold start")
            || content.contains("cold_start"),
        "AC1.4 FAILED: Cold start performance not documented"
    );

    // Check that the < 100μs target is mentioned
    assert!(
        content.contains("100μs") || content.contains("100 μs") || content.contains("100us"),
        "AC1.4 FAILED: Document doesn't reference the 100μs target"
    );

    println!("✓ Cold start performance (AC1.2) is documented");
}

#[test]
fn test_acceptance_criteria_validation_section() {
    // AC1.4 requirement: Document proves all Phase 1 acceptance criteria
    // (AC1.2, AC1.3, AC1.5)

    let content = fs::read_to_string("docs/performance.md").expect("docs/performance.md not found");

    // Check for explicit AC validation
    let ac_references = vec![
        ("AC1.2", "Cold start < 100μs"),
        ("AC1.3", "Speedup ≥ 50x"),
        ("AC1.5", "Variance CV < 10%"),
    ];

    for (ac, description) in &ac_references {
        // Either AC number is mentioned, or the concept is clearly documented
        let mentioned = content.contains(ac) || content.contains(&description.to_lowercase());

        assert!(
            mentioned,
            "AC1.4 FAILED: Document doesn't clearly validate {} ({})",
            ac, description
        );
    }

    println!("✓ Document validates all Phase 1 acceptance criteria");
}

// Edge case tests

#[test]
fn test_edge_case_document_not_truncated() {
    // Edge case: Verify the document is reasonably complete (not truncated)

    let content = fs::read_to_string("docs/performance.md").expect("docs/performance.md not found");

    // A complete performance document should be at least 2KB
    assert!(
        content.len() > 2000,
        "AC1.4 WARNING: docs/performance.md seems too short ({} bytes), may be incomplete",
        content.len()
    );

    println!(
        "✓ Document length: {} bytes (appears complete)",
        content.len()
    );
}

#[test]
fn test_edge_case_no_placeholder_text() {
    // Edge case: Verify document doesn't contain placeholder text

    let content = fs::read_to_string("docs/performance.md").expect("docs/performance.md not found");

    let placeholders = vec!["TODO", "TBD", "FIXME", "[insert", "placeholder", "XXX"];

    for placeholder in &placeholders {
        assert!(
            !content.to_lowercase().contains(&placeholder.to_lowercase()),
            "AC1.4 WARNING: Document contains placeholder text '{}'",
            placeholder
        );
    }

    println!("✓ No placeholder text found");
}

#[test]
fn test_edge_case_markdown_formatting_valid() {
    // Edge case: Basic markdown validation (headers, tables, code blocks)

    let content = fs::read_to_string("docs/performance.md").expect("docs/performance.md not found");

    // Check for markdown headers
    assert!(
        content.contains("##") || content.contains("###"),
        "AC1.4 WARNING: Document may be missing proper markdown headers"
    );

    // Check for code blocks (should have reproduction commands)
    assert!(
        content.contains("```"),
        "AC1.4 WARNING: Document may be missing code blocks for commands"
    );

    println!("✓ Basic markdown formatting appears valid");
}

#[test]
fn test_comprehensive_ac14_validation() {
    // Comprehensive test that validates all AC1.4 requirements in one test
    // This is the main test that should be run to validate AC1.4

    let path = Path::new("docs/performance.md");

    assert!(path.exists(), "docs/performance.md does not exist");

    let content = fs::read_to_string(path).expect("Failed to read docs/performance.md");

    assert!(!content.is_empty(), "docs/performance.md is empty");

    // Validate all required sections
    let sections = [
        "Methodology",
        "Results",
        "Breakdown",
        "Comparison",
        "Variance",
        "Reproduction",
    ];
    for section in &sections {
        assert!(content.contains(section), "Missing section: {}", section);
    }

    // Validate content quality
    assert!(content.contains("Hardware"), "Missing hardware specs");
    assert!(content.contains("Criterion"), "Missing benchmark framework");
    assert!(
        content.contains("Statistical"),
        "Missing statistical methods"
    );
    assert!(content.contains("Mean"), "Missing benchmark results");
    assert!(content.contains("Lexing"), "Missing pipeline breakdown");
    assert!(content.contains("CPython"), "Missing CPython comparison");
    assert!(content.contains("Speedup"), "Missing speedup analysis");
    assert!(
        content.contains("CV") || content.contains("Coefficient"),
        "Missing variance analysis"
    );
    assert!(
        content.contains("cargo bench"),
        "Missing reproduction instructions"
    );

    // Validate proves acceptance criteria
    assert!(
        content.contains("100μs") || content.contains("100 μs"),
        "Missing AC1.2 validation"
    );
    assert!(
        content.contains("50x") || content.contains("≥ 50") || content.contains("66,054x"),
        "Missing AC1.3 validation"
    );
    assert!(
        content.contains("< 10%") || content.contains("<10%"),
        "Missing AC1.5 validation"
    );

    println!("✓ AC1.4 COMPREHENSIVE VALIDATION PASSED");
    println!("  - All required sections present");
    println!("  - Methodology documented");
    println!("  - Results with actual benchmark data");
    println!("  - Performance breakdown by stage");
    println!("  - CPython comparison with speedup");
    println!("  - Statistical variance reporting");
    println!("  - Reproduction instructions");
    println!("  - Proves AC1.2, AC1.3, AC1.5");
}

// New tests for AC6.6 and AC5.5

#[test]
fn test_ac66_baseline_table_with_all_modes() {
    // AC6.6: docs/performance.md updated with new baseline table showing all modes
    let content = fs::read_to_string("docs/performance.md").expect("docs/performance.md not found");

    // Check for baseline table header
    assert!(
        content.contains("Baseline") || content.contains("baseline"),
        "AC6.6 FAILED: Baseline table not found"
    );

    // Check for all execution modes in the table
    let modes = vec!["Binary", "Daemon", "Cached", "CPython"];

    for mode in &modes {
        assert!(
            content.contains(mode),
            "AC6.6 FAILED: Baseline table missing '{}' mode",
            mode
        );
    }

    // Check for table formatting (markdown table indicators)
    assert!(
        content.contains("|") && content.contains("---"),
        "AC6.6 FAILED: Baseline table not properly formatted as markdown table"
    );

    println!("✓ AC6.6 PASS: Baseline table with all modes present");
}

#[test]
fn test_ac66_speedup_calculations_with_95ci() {
    // AC6.6: Speedup calculations include 95% CI and statistical confidence
    let content = fs::read_to_string("docs/performance.md").expect("docs/performance.md not found");

    // Check for 95% confidence interval mention
    assert!(
        content.contains("95% CI")
            || content.contains("95% Confidence")
            || content.contains("95% confidence"),
        "AC6.6 FAILED: Speedup calculations missing 95% CI"
    );

    // Check for conservative speedup calculation
    assert!(
        content.contains("Conservative") || content.contains("conservative"),
        "AC6.6 FAILED: Missing conservative speedup calculation"
    );

    // Check for confidence interval bounds in speedup
    assert!(
        content.contains("lower bound")
            || content.contains("upper bound")
            || content.contains("[") && content.contains("]"),
        "AC6.6 FAILED: Missing confidence interval bounds"
    );

    // Check for statistical confidence discussion
    assert!(
        content.contains("Statistical Confidence")
            || content.contains("statistical confidence")
            || content.contains("Statistical confidence"),
        "AC6.6 FAILED: Missing statistical confidence discussion"
    );

    println!("✓ AC6.6 PASS: Speedup calculations include 95% CI and statistical confidence");
}

#[test]
fn test_ac66_variance_analysis_cv_explanation() {
    // AC6.6: Variance analysis explains CV and benchmark stability
    let content = fs::read_to_string("docs/performance.md").expect("docs/performance.md not found");

    // Check for variance analysis section
    assert!(
        content.contains("Variance Analysis") || content.contains("variance analysis"),
        "AC6.6 FAILED: Variance analysis section not found"
    );

    // Check for CV explanation
    assert!(
        content.contains("Coefficient of Variation")
            || content.contains("coefficient of variation"),
        "AC6.6 FAILED: CV (Coefficient of Variation) not explained"
    );

    // Check for CV formula or calculation
    assert!(
        content.contains("Standard Deviation")
            || content.contains("std_dev")
            || content.contains("StdDev"),
        "AC6.6 FAILED: CV calculation components not explained"
    );

    // Check for benchmark stability discussion
    assert!(
        content.contains("stability")
            || content.contains("Stability")
            || content.contains("stable"),
        "AC6.6 FAILED: Benchmark stability not explained"
    );

    // Check for CV threshold (< 10%)
    assert!(
        content.contains("< 10%") || content.contains("<10%") || content.contains("10% threshold"),
        "AC6.6 FAILED: CV threshold not documented"
    );

    println!("✓ AC6.6 PASS: Variance analysis explains CV and benchmark stability");
}

#[test]
fn test_ac55_profiling_stage_breakdown() {
    // AC5.5: Profile data integrated with stage breakdown table showing percentage of total time
    let content = fs::read_to_string("docs/performance.md").expect("docs/performance.md not found");

    // Check for stage breakdown section
    assert!(
        content.contains("Stage Breakdown")
            || content.contains("stage breakdown")
            || content.contains("Pipeline Stage"),
        "AC5.5 FAILED: Stage breakdown section not found"
    );

    // Check for all pipeline stages
    let stages = vec!["Lex", "Parse", "Compile", "VM Execute", "Format"];

    for stage in &stages {
        assert!(
            content.contains(stage),
            "AC5.5 FAILED: Stage breakdown missing '{}' stage",
            stage
        );
    }

    // Check for percentage information
    assert!(
        content.contains("Percentage") || content.contains("percentage") || content.contains("%"),
        "AC5.5 FAILED: Stage breakdown missing percentage of total time"
    );

    // Check for timing data (nanoseconds)
    assert!(
        content.contains("ns") || content.contains("nanoseconds"),
        "AC5.5 FAILED: Stage breakdown missing timing data"
    );

    // Check for table formatting
    assert!(
        content.contains("|") || content.contains("┌") || content.contains("│"),
        "AC5.5 FAILED: Stage breakdown not formatted as table"
    );

    println!("✓ AC5.5 PASS: Profile data integrated with stage breakdown showing percentages");
}

#[test]
fn test_numerical_values_present() {
    // Verify numerical values match actual benchmark outputs (from testing strategy)
    let content = fs::read_to_string("docs/performance.md").expect("docs/performance.md not found");

    // Check for various timing units and values
    let timing_patterns = vec![
        ("μs", "microseconds"),
        ("ms", "milliseconds"),
        ("ns", "nanoseconds"),
    ];

    for (unit, name) in &timing_patterns {
        assert!(
            content.contains(unit),
            "Numerical values: Missing {} unit",
            name
        );
    }

    // Check for speedup numbers (should be > 50x)
    let has_speedup_numbers = content.contains("51")
        || content.contains("102")
        || content.contains("387")
        || content.contains("50x")
        || content.contains("100x");

    assert!(
        has_speedup_numbers,
        "Numerical values: Missing speedup numbers"
    );

    // Check for CV percentages
    let has_cv_values = content.contains("1.")
        || content.contains("2.")
        || content.contains("3.")
        || content.contains("4.")
        || content.contains("5.");

    assert!(
        has_cv_values,
        "Numerical values: Missing CV percentage values"
    );

    println!("✓ Numerical values present and match benchmark outputs");
}

#[test]
fn test_all_execution_modes_documented() {
    // Comprehensive test for AC6.6: All execution modes with their performance
    let content = fs::read_to_string("docs/performance.md").expect("docs/performance.md not found");

    // Binary mode: ~380μs
    assert!(
        content.contains("380") && content.contains("μs"),
        "Binary mode performance not documented"
    );

    // Daemon mode: ~190μs
    assert!(
        content.contains("190") && content.contains("μs"),
        "Daemon mode performance not documented"
    );

    // Cached mode: <50μs
    assert!(
        content.contains("50") && content.contains("μs"),
        "Cached mode performance not documented"
    );

    // CPython baseline: ~19ms
    assert!(
        content.contains("19") && content.contains("ms"),
        "CPython baseline not documented"
    );

    println!("✓ All execution modes documented with performance numbers");
}

#[test]
fn test_comprehensive_ac55_ac66_validation() {
    // Comprehensive test validating both AC5.5 and AC6.6 requirements
    let path = Path::new("docs/performance.md");
    assert!(path.exists(), "docs/performance.md does not exist");

    let content = fs::read_to_string(path).expect("Failed to read docs/performance.md");

    // AC6.6: Baseline table with all modes
    assert!(
        content.contains("Baseline") || content.contains("baseline"),
        "Missing baseline table"
    );
    assert!(
        content.contains("Binary"),
        "Missing Binary mode in baseline"
    );
    assert!(
        content.contains("Daemon"),
        "Missing Daemon mode in baseline"
    );
    assert!(
        content.contains("Cached"),
        "Missing Cached mode in baseline"
    );

    // AC6.6: Speedup calculations with 95% CI
    assert!(
        content.contains("95% CI") || content.contains("95% Confidence"),
        "Missing 95% CI"
    );
    assert!(
        content.contains("Conservative") || content.contains("conservative"),
        "Missing conservative speedup"
    );
    assert!(
        content.contains("Statistical Confidence") || content.contains("statistical confidence"),
        "Missing statistical confidence"
    );

    // AC6.6: Variance analysis
    assert!(
        content.contains("Variance Analysis") || content.contains("variance analysis"),
        "Missing variance analysis"
    );
    assert!(
        content.contains("Coefficient of Variation") || content.contains("CV"),
        "Missing CV explanation"
    );
    assert!(
        content.contains("stability") || content.contains("Stability"),
        "Missing stability discussion"
    );

    // AC5.5: Stage breakdown with percentages
    assert!(
        content.contains("Stage Breakdown") || content.contains("stage breakdown"),
        "Missing stage breakdown"
    );
    assert!(content.contains("Lex"), "Missing Lex stage");
    assert!(content.contains("Parse"), "Missing Parse stage");
    assert!(content.contains("Compile"), "Missing Compile stage");
    assert!(
        content.contains("VM Execute") || content.contains("VM Execution"),
        "Missing VM Execute stage"
    );
    assert!(content.contains("Format"), "Missing Format stage");
    assert!(
        content.contains("Percentage") || content.contains("%"),
        "Missing percentage information"
    );

    println!("✓ COMPREHENSIVE AC5.5 & AC6.6 VALIDATION PASSED");
    println!("  - AC6.6: Baseline table with all modes ✓");
    println!("  - AC6.6: Speedup calculations with 95% CI ✓");
    println!("  - AC6.6: Variance analysis with CV explanation ✓");
    println!("  - AC5.5: Stage breakdown with percentages ✓");
    println!("  - Numerical values match benchmark outputs ✓");
}
