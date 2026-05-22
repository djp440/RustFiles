use std::path::PathBuf;
use std::process::Command;

fn script_path() -> PathBuf {
    let mut p = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    p.pop();
    p.push("scripts");
    p.push("create-fixtures.ps1");
    p
}

fn run_script(root: &str) -> std::process::Output {
    Command::new("powershell")
        .args([
            "-ExecutionPolicy",
            "Bypass",
            "-File",
            &script_path().to_string_lossy(),
            "-Root",
            root,
        ])
        .output()
        .expect("failed to execute create-fixtures.ps1")
}

#[test]
fn fixture_script_creates_expected_directories() {
    let root = ".tmp\\test_fixtures";
    let output = run_script(root);
    assert!(output.status.success(), "script failed: {}", String::from_utf8_lossy(&output.stderr));

    let expected_dirs = [
        "small-dir",
        "large-10k-dir",
        "media-dir",
        "conflict-source",
        "conflict-target",
        "deep-tree",
        "permission-cases",
    ];

    for dir in &expected_dirs {
        let path = PathBuf::from(root).join(dir);
        assert!(path.exists(), "expected directory {} to exist", path.display());
    }
}

#[test]
fn fixture_script_rejects_empty_root() {
    let output = Command::new("powershell")
        .args([
            "-ExecutionPolicy",
            "Bypass",
            "-File",
            &script_path().to_string_lossy(),
            "-Root",
            "",
        ])
        .output()
        .expect("failed to execute create-fixtures.ps1");
    assert!(!output.status.success(), "script should reject empty root");
}

#[test]
fn fixture_script_rejects_user_profile_root() {
    let home = std::env::var("USERPROFILE").expect("USERPROFILE not set");
    let output = run_script(&home);
    assert!(!output.status.success(), "script should reject USERPROFILE root");
}

#[test]
fn fixture_script_large_10k_has_correct_file_count() {
    let root = ".tmp\\test_fixtures_10k";
    let output = run_script(root);
    assert!(output.status.success(), "script failed: {}", String::from_utf8_lossy(&output.stderr));

    let large_dir = PathBuf::from(root).join("large-10k-dir");
    assert!(large_dir.exists(), "large-10k-dir should exist");

    let count = std::fs::read_dir(&large_dir)
        .expect("should read large-10k-dir")
        .count();
    assert_eq!(count, 10000, "large-10k-dir should contain exactly 10000 files");
}
