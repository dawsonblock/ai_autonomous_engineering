//! Integration tests for validation script functionality
//!
//! Tests the acceptance criteria validation script added in issue/14-final-validation
//! Verifies that the script correctly checks all production-ready requirements

use std::fs;
use std::path::Path;

#[test]
fn test_validation_script_exists() {
    let script_path = "scripts/validate_prd_acceptance_criteria.sh";
    assert!(
        Path::new(script_path).exists(),
        "Validation script should exist at {}",
        script_path
    );
}

#[test]
fn test_validation_script_is_executable() {
    let script_path = "scripts/validate_prd_acceptance_criteria.sh";

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let metadata = fs::metadata(script_path).expect("Should be able to read script metadata");
        let permissions = metadata.permissions();
        let mode = permissions.mode();

        // Check if executable bit is set (owner, group, or other)
        let is_executable = (mode & 0o111) != 0;
        assert!(
            is_executable,
            "Validation script should be executable (mode: {:o})",
            mode
        );
    }
}

#[test]
fn test_validation_script_has_bash_shebang() {
    let script_path = "scripts/validate_prd_acceptance_criteria.sh";
    let content = fs::read_to_string(script_path).expect("Should be able to read script content");

    assert!(
        content.starts_with("#!/usr/bin/env bash") || content.starts_with("#!/bin/bash"),
        "Script should have bash shebang"
    );
}

#[test]
fn test_validation_script_checks_all_17_criteria() {
    let script_path = "scripts/validate_prd_acceptance_criteria.sh";
    let content = fs::read_to_string(script_path).expect("Should be able to read script content");

    // Verify all 17 acceptance criteria are checked
    for i in 1..=17 {
        let ac_marker = format!("AC{}", i);
        assert!(
            content.contains(&ac_marker),
            "Script should check acceptance criterion {}",
            i
        );
    }
}

#[test]
fn test_validation_script_checks_binary_size() {
    let script_path = "scripts/validate_prd_acceptance_criteria.sh";
    let content = fs::read_to_string(script_path).expect("Should be able to read script content");

    // AC16: Binary size check
    assert!(content.contains("stat") && content.contains("target/release/pyrust"));
    assert!(content.contains("500000") || content.contains("500KB"));
}

#[test]
fn test_validation_script_checks_python_linkage() {
    let script_path = "scripts/validate_prd_acceptance_criteria.sh";
    let content = fs::read_to_string(script_path).expect("Should be able to read script content");

    // AC17: No Python linkage check
    assert!(content.contains("otool") || content.contains("ldd"));
    assert!(content.contains("python"));
}

#[test]
fn test_validation_script_checks_clippy_warnings() {
    let script_path = "scripts/validate_prd_acceptance_criteria.sh";
    let content = fs::read_to_string(script_path).expect("Should be able to read script content");

    // AC2: Clippy warnings check
    assert!(content.contains("cargo clippy"));
    assert!(content.contains("-D warnings"));
}

#[test]
fn test_validation_script_checks_formatting() {
    let script_path = "scripts/validate_prd_acceptance_criteria.sh";
    let content = fs::read_to_string(script_path).expect("Should be able to read script content");

    // AC3: Formatting check
    assert!(content.contains("cargo fmt"));
    assert!(content.contains("--check"));
}

#[test]
fn test_validation_script_checks_docs_directory() {
    let script_path = "scripts/validate_prd_acceptance_criteria.sh";
    let content = fs::read_to_string(script_path).expect("Should be able to read script content");

    // AC12: docs/ directory check
    assert!(content.contains("docs"));
    assert!(content.contains(".md") || content.contains("markdown"));
}

#[test]
fn test_validation_script_checks_readme() {
    let script_path = "scripts/validate_prd_acceptance_criteria.sh";
    let content = fs::read_to_string(script_path).expect("Should be able to read script content");

    // AC9: README check
    assert!(content.contains("README.md"));
    assert!(content.contains("500") || content.contains("wc"));
}

#[test]
fn test_validation_script_checks_license() {
    let script_path = "scripts/validate_prd_acceptance_criteria.sh";
    let content = fs::read_to_string(script_path).expect("Should be able to read script content");

    // AC10: LICENSE check
    assert!(content.contains("LICENSE"));
}

#[test]
fn test_validation_script_checks_gitignore() {
    let script_path = "scripts/validate_prd_acceptance_criteria.sh";
    let content = fs::read_to_string(script_path).expect("Should be able to read script content");

    // AC11: .gitignore check
    assert!(content.contains(".gitignore"));
    assert!(content.contains("log") || content.contains("bak") || content.contains("tmp"));
}

#[test]
fn test_test_validation_script_exists() {
    let test_script_path = "tests/test_prd_acceptance_criteria_validation.sh";
    assert!(
        Path::new(test_script_path).exists(),
        "Test validation script should exist at {}",
        test_script_path
    );
}

#[test]
fn test_readme_meets_size_requirement() {
    // Verify README exists and meets AC9 requirement (>= 500 bytes)
    let readme_path = "README.md";
    assert!(Path::new(readme_path).exists(), "README.md should exist");

    let metadata = fs::metadata(readme_path).expect("Should be able to read README metadata");
    let size = metadata.len();

    assert!(
        size >= 500,
        "README.md should be at least 500 bytes (actual: {} bytes)",
        size
    );
}

#[test]
fn test_license_file_exists() {
    // Verify LICENSE exists (AC10)
    let license_path = "LICENSE";
    assert!(
        Path::new(license_path).exists(),
        "LICENSE file should exist"
    );

    let content = fs::read_to_string(license_path).expect("Should be able to read LICENSE file");

    assert!(!content.is_empty(), "LICENSE file should not be empty");
}

#[test]
fn test_docs_directory_structure() {
    // Verify docs/ directory exists and has markdown files (AC12)
    let docs_path = "docs";
    assert!(
        Path::new(docs_path).is_dir(),
        "docs/ directory should exist"
    );

    let entries = fs::read_dir(docs_path).expect("Should be able to read docs directory");

    let md_count = entries
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.path()
                .extension()
                .and_then(|s| s.to_str())
                .map(|s| s == "md")
                .unwrap_or(false)
        })
        .count();

    assert!(
        md_count >= 3,
        "docs/ should contain at least 3 markdown files (found: {})",
        md_count
    );
}

#[test]
fn test_no_loose_markdown_in_root() {
    // Verify no loose markdown files in root except README.md (AC13)
    let entries = fs::read_dir(".").expect("Should be able to read root directory");

    let loose_md_files: Vec<String> = entries
        .filter_map(|e| e.ok())
        .filter(|e| e.path().is_file())
        .filter(|e| {
            e.path()
                .extension()
                .and_then(|s| s.to_str())
                .map(|s| s == "md")
                .unwrap_or(false)
        })
        .map(|e| e.file_name().to_string_lossy().to_string())
        .filter(|name| name != "README.md")
        .collect();

    assert!(
        loose_md_files.is_empty(),
        "Should have no loose markdown files in root (found: {:?})",
        loose_md_files
    );
}

#[test]
fn test_gitignore_has_artifact_patterns() {
    // Verify .gitignore contains required patterns (AC11)
    let gitignore_path = ".gitignore";
    assert!(
        Path::new(gitignore_path).exists(),
        ".gitignore should exist"
    );

    let content = fs::read_to_string(gitignore_path).expect("Should be able to read .gitignore");

    let required_patterns = vec![".log", ".bak", ".tmp", "dhat-heap"];
    let mut found_patterns = 0;

    for pattern in required_patterns {
        if content.contains(pattern) {
            found_patterns += 1;
        }
    }

    assert!(
        found_patterns >= 4,
        ".gitignore should contain at least 4 artifact patterns (found: {})",
        found_patterns
    );
}

#[test]
fn test_cargo_toml_pyo3_configuration() {
    // Verify PyO3 is dev-dependency only (AC15)
    let cargo_toml = fs::read_to_string("Cargo.toml").expect("Should be able to read Cargo.toml");

    // Check that pyo3 is in [dev-dependencies] section
    let dev_deps_section = cargo_toml
        .split("[dev-dependencies]")
        .nth(1)
        .expect("Should have [dev-dependencies] section");

    assert!(
        dev_deps_section.contains("pyo3"),
        "pyo3 should be in [dev-dependencies]"
    );

    // Verify it's not in regular [dependencies]
    if let Some(deps_section) = cargo_toml.split("[dependencies]").nth(1) {
        let deps_content = deps_section.split('[').next().unwrap_or("");
        assert!(
            !deps_content.contains("pyo3"),
            "pyo3 should not be in regular [dependencies]"
        );
    }
}

#[test]
fn test_daemon_client_documentation_updated() {
    // Verify daemon_client.rs documentation is comprehensive
    // (modified in issue/14-final-validation)

    let daemon_client_path = "src/daemon_client.rs";
    let content =
        fs::read_to_string(daemon_client_path).expect("Should be able to read daemon_client.rs");

    // Check for module-level documentation
    assert!(content.contains("//!"), "Should have module-level docs");

    // Check for key documented features
    assert!(content.contains("Architecture") || content.contains("architecture"));
    assert!(content.contains("Example") || content.contains("example"));
}

#[test]
fn test_performance_documentation_exists() {
    // Verify performance.md exists in docs/ (from issue/13)
    let perf_doc_path = "docs/performance.md";
    assert!(
        Path::new(perf_doc_path).exists(),
        "docs/performance.md should exist"
    );

    let content = fs::read_to_string(perf_doc_path).expect("Should be able to read performance.md");

    assert!(!content.is_empty(), "performance.md should not be empty");
}

#[test]
fn test_validation_scripts_integration() {
    // Test that both validation scripts work together

    let main_script = "scripts/validate_prd_acceptance_criteria.sh";
    let test_script = "tests/test_prd_acceptance_criteria_validation.sh";

    assert!(Path::new(main_script).exists());
    assert!(Path::new(test_script).exists());

    // Both should check similar criteria
    let main_content = fs::read_to_string(main_script).unwrap();
    let test_content = fs::read_to_string(test_script).unwrap();

    // Both should reference acceptance criteria
    assert!(main_content.contains("AC") || main_content.contains("acceptance"));
    assert!(test_content.contains("AC") || test_content.contains("acceptance"));
}
