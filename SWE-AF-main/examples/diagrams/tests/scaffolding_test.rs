use std::fs;
use std::path::Path;
use std::process::Command;

#[test]
fn test_cargo_toml_exists() {
    assert!(
        Path::new("Cargo.toml").exists(),
        "Cargo.toml file should exist"
    );
}

#[test]
fn test_cargo_toml_parses() {
    let cargo_toml_content = fs::read_to_string("Cargo.toml").expect("Failed to read Cargo.toml");

    // Verify it contains required sections
    assert!(
        cargo_toml_content.contains("[package]"),
        "Cargo.toml should contain [package] section"
    );
    assert!(
        cargo_toml_content.contains("name = \"diagrams\""),
        "Cargo.toml should contain name = \"diagrams\""
    );
    assert!(
        cargo_toml_content.contains("edition = \"2021\""),
        "Cargo.toml should contain edition = \"2021\""
    );
    assert!(
        cargo_toml_content.contains("rust-version = \"1.70\""),
        "Cargo.toml should contain rust-version = \"1.70\""
    );
}

#[test]
fn test_cargo_toml_has_clap_dependency() {
    let cargo_toml_content = fs::read_to_string("Cargo.toml").expect("Failed to read Cargo.toml");

    assert!(
        cargo_toml_content.contains("[dependencies]"),
        "Cargo.toml should contain [dependencies] section"
    );
    assert!(
        cargo_toml_content.contains("clap"),
        "Cargo.toml should contain clap dependency"
    );
    assert!(
        cargo_toml_content.contains("derive"),
        "clap dependency should have derive feature"
    );
}

#[test]
fn test_cargo_toml_has_dev_dependencies() {
    let cargo_toml_content = fs::read_to_string("Cargo.toml").expect("Failed to read Cargo.toml");

    assert!(
        cargo_toml_content.contains("[dev-dependencies]"),
        "Cargo.toml should contain [dev-dependencies] section"
    );
    assert!(
        cargo_toml_content.contains("insta"),
        "Cargo.toml should contain insta dev dependency"
    );
    assert!(
        cargo_toml_content.contains("tempfile"),
        "Cargo.toml should contain tempfile dev dependency"
    );
}

#[test]
fn test_cargo_toml_has_release_profile() {
    let cargo_toml_content = fs::read_to_string("Cargo.toml").expect("Failed to read Cargo.toml");

    assert!(
        cargo_toml_content.contains("[profile.release]"),
        "Cargo.toml should contain [profile.release] section"
    );
    assert!(
        cargo_toml_content.contains("lto = true"),
        "Release profile should have lto = true"
    );
    assert!(
        cargo_toml_content.contains("strip = true"),
        "Release profile should have strip = true"
    );
    assert!(
        cargo_toml_content.contains("codegen-units = 1"),
        "Release profile should have codegen-units = 1"
    );
}

#[test]
fn test_src_directory_exists() {
    assert!(Path::new("src").is_dir(), "src/ directory should exist");
}

#[test]
fn test_main_rs_exists() {
    assert!(
        Path::new("src/main.rs").exists(),
        "src/main.rs file should exist"
    );
}

#[test]
fn test_project_builds_successfully() {
    let output = Command::new("cargo")
        .args(["build", "--release"])
        .output()
        .expect("Failed to execute cargo build");

    assert!(
        output.status.success(),
        "cargo build --release should exit with status 0. stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn test_cargo_fmt_check_passes() {
    let output = Command::new("cargo")
        .args(["fmt", "--check"])
        .output()
        .expect("Failed to execute cargo fmt");

    assert!(
        output.status.success(),
        "cargo fmt --check should exit with status 0. stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn test_cargo_clippy_passes() {
    let output = Command::new("cargo")
        .args(["clippy", "--", "-D", "warnings"])
        .output()
        .expect("Failed to execute cargo clippy");

    assert!(
        output.status.success(),
        "cargo clippy -- -D warnings should exit with status 0. stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

// Edge case tests
#[test]
fn test_cargo_toml_is_valid_toml() {
    let cargo_toml_content = fs::read_to_string("Cargo.toml").expect("Failed to read Cargo.toml");

    // Try to parse it as TOML to ensure it's valid
    let _parsed: toml::Value =
        toml::from_str(&cargo_toml_content).expect("Cargo.toml should be valid TOML");
}

#[test]
fn test_main_rs_is_valid_rust() {
    let main_rs_content = fs::read_to_string("src/main.rs").expect("Failed to read src/main.rs");

    // Basic sanity check - should contain a main function
    assert!(
        main_rs_content.contains("fn main()"),
        "src/main.rs should contain a main function"
    );
}

#[test]
fn test_tests_directory_exists() {
    assert!(Path::new("tests").is_dir(), "tests/ directory should exist");
}
