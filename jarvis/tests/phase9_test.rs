use std::path::Path;
use std::fs;

#[test]
fn test_phase9_installation_scripts_exist() {
    let root = Path::new("..");
    assert!(root.join("scripts/install_linux.sh").exists());
    assert!(root.join("scripts/install_macos.sh").exists());
    assert!(root.join("scripts/install_windows.ps1").exists());
}

#[test]
fn test_phase9_cargo_dist_config() {
    let cargo_toml_path = Path::new("Cargo.toml");
    let content = fs::read_to_string(cargo_toml_path).expect("Failed to read Cargo.toml");
    
    assert!(content.contains("[package.metadata.dist]"));
    assert!(content.contains("cargo-dist-version"));
    assert!(content.contains("installers = [\"shell\", \"powershell\"]"));
    assert!(content.contains("targets = [\"x86_64-unknown-linux-gnu\", \"x86_64-apple-darwin\", \"aarch64-apple-darwin\", \"x86_64-pc-windows-msvc\"]"));
}

#[test]
fn test_phase9_linux_script_content() {
    let script_path = Path::new("../scripts/install_linux.sh");
    let content = fs::read_to_string(script_path).expect("Failed to read install_linux.sh");
    
    assert!(content.contains("ollama"));
    assert!(content.contains("postgresql"));
    assert!(content.contains("pgvector"));
}

#[test]
fn test_phase9_macos_script_content() {
    let script_path = Path::new("../scripts/install_macos.sh");
    let content = fs::read_to_string(script_path).expect("Failed to read install_macos.sh");
    
    assert!(content.contains("brew"));
    assert!(content.contains("ollama"));
    assert!(content.contains("postgresql"));
}
