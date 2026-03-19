use assert_cmd::Command;
use tempfile::tempdir;
use std::fs::File;
use std::io::Write;

#[test]
fn test_no_truncate() {
    let dir = tempdir().unwrap();
    let file_path = dir.path().join("many_symbols.rs");
    let content = (1..20).map(|i| format!("fn func{}() {{}}\n", i)).collect::<String>();
    std::fs::write(&file_path, content).unwrap();

    let output = Command::cargo_bin("ai-dir")
        .unwrap()
        .args(["--no-truncate", "--format", "plain"])
        .current_dir(dir.path())
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("fn func1"));
    assert!(stdout.contains("fn func19"));
    assert!(!stdout.contains("…"));
}