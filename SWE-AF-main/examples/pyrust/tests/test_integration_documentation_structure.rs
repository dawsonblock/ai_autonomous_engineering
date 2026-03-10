//! Integration tests for documentation consolidation
//!
//! Tests documentation structure after issue/13-documentation-consolidation merge
//! Verifies that documentation is properly organized

use std::fs;
use std::path::Path;

#[test]
fn test_docs_directory_exists() {
    assert!(Path::new("docs").is_dir(), "docs/ directory should exist");
}

#[test]
fn test_docs_readme_exists() {
    let path = "docs/README.md";
    assert!(Path::new(path).exists(), "docs/README.md should exist");

    let content = fs::read_to_string(path).expect("Should be able to read docs/README.md");
    assert!(!content.is_empty(), "docs/README.md should not be empty");
}

#[test]
fn test_required_documentation_files_exist() {
    let required_docs = vec![
        "docs/README.md",
        "docs/performance.md",
        "docs/integration-verification.md",
        "docs/validation.md",
    ];

    for doc_path in required_docs {
        assert!(
            Path::new(doc_path).exists(),
            "Required documentation {} should exist",
            doc_path
        );

        let content = fs::read_to_string(doc_path)
            .unwrap_or_else(|_| panic!("Should be able to read {}", doc_path));

        assert!(
            !content.is_empty(),
            "Documentation file {} should not be empty",
            doc_path
        );
    }
}

#[test]
fn test_performance_documentation_structure() {
    let path = "docs/performance.md";
    let content = fs::read_to_string(path).expect("Should be able to read docs/performance.md");

    // Check for key sections in performance documentation
    let expected_sections = vec!["Performance", "Benchmark", "Optimization"];

    let mut found_sections = 0;
    for section in expected_sections {
        if content.contains(section) {
            found_sections += 1;
        }
    }

    assert!(
        found_sections >= 1,
        "performance.md should contain performance-related sections"
    );
}

#[test]
fn test_root_readme_points_to_docs() {
    let readme_path = "README.md";
    assert!(
        Path::new(readme_path).exists(),
        "README.md should exist in root"
    );

    let content = fs::read_to_string(readme_path).expect("Should be able to read README.md");

    // Root README should reference the docs directory
    assert!(
        content.contains("docs/") || content.contains("documentation"),
        "Root README should reference docs/ directory"
    );
}

#[test]
fn test_markdown_files_have_headers() {
    // All documentation files should have proper headers
    let docs_dir = fs::read_dir("docs").expect("Should be able to read docs directory");

    for entry in docs_dir.filter_map(|e| e.ok()) {
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) == Some("md") {
            let content = fs::read_to_string(&path)
                .unwrap_or_else(|_| panic!("Should be able to read {:?}", path));

            // Should have at least one markdown header
            assert!(
                content.contains("# ") || content.contains("## "),
                "Documentation file {:?} should have markdown headers",
                path
            );
        }
    }
}
