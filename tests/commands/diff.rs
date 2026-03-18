use assert_cmd::Command;
use std::fs::File;
use std::io::Write;
use tempfile::tempdir;

fn setup_git_repo() -> tempfile::TempDir {
    let dir = tempdir().unwrap();
    // инициализируем git
    Command::new("git")
        .args(["init"])
        .current_dir(&dir)
        .output()
        .unwrap();
    // настроим user (чтобы не было ошибок)
    Command::new("git")
        .args(["config", "user.email", "test@example.com"])
        .current_dir(&dir)
        .output()
        .unwrap();
    Command::new("git")
        .args(["config", "user.name", "Test User"])
        .current_dir(&dir)
        .output()
        .unwrap();
    dir
}

#[test]
fn test_diff_staged() {
    let dir = setup_git_repo();
    let file_path = dir.path().join("test.txt");
    let mut f = File::create(&file_path).unwrap();
    writeln!(f, "initial content").unwrap();

    // add and commit
    Command::new("git")
        .args(["add", "test.txt"])
        .current_dir(&dir)
        .output()
        .unwrap();
    Command::new("git")
        .args(["commit", "-m", "initial"])
        .current_dir(&dir)
        .output()
        .unwrap();

    // modify file
    let mut f = File::create(&file_path).unwrap();
    writeln!(f, "modified content").unwrap();
    Command::new("git")
        .args(["add", "test.txt"])
        .current_dir(&dir)
        .output()
        .unwrap();

    // run diff --staged
    let output = Command::cargo_bin("ai-dir")
        .unwrap()
        .args(["--diff", "--staged"])
        .current_dir(dir.path())
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("diff --git a/test.txt b/test.txt"));
    assert!(stdout.contains("+modified content"));
}

#[test]
fn test_diff_unstaged() {
    let dir = setup_git_repo();
    let file_path = dir.path().join("test.txt");
    let mut f = File::create(&file_path).unwrap();
    writeln!(f, "initial").unwrap();
    Command::new("git")
        .args(["add", "test.txt"])
        .current_dir(&dir)
        .output()
        .unwrap();
    Command::new("git")
        .args(["commit", "-m", "initial"])
        .current_dir(&dir)
        .output()
        .unwrap();

    // modify without adding
    let mut f = File::create(&file_path).unwrap();
    writeln!(f, "modified").unwrap();

    let output = Command::cargo_bin("ai-dir")
        .unwrap()
        .args(["--diff"])
        .current_dir(dir.path())
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("diff --git a/test.txt b/test.txt"));
    assert!(stdout.contains("+modified"));
}