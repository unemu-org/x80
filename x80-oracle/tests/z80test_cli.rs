use std::process::Command;

fn command() -> Command {
    let mut c = Command::new(env!("CARGO_BIN_EXE_x80-oracle"));
    c.arg("z80test");
    c
}

#[test]
fn help_variants_budgets_and_missing_archive() {
    let out = command().arg("--help").output().unwrap();
    assert!(out.status.success());
    assert!(String::from_utf8(out.stdout)
        .unwrap()
        .contains("--step-cap"));
    for args in [
        vec!["z80ccfscr"],
        vec!["all", "--step-cap", "0"],
        vec!["z80full", "--time-cap", "no"],
        vec!["z80doc", "z80flags"],
    ] {
        let out = command().args(args).output().unwrap();
        assert_eq!(out.status.code(), Some(1));
    }
    let out = command()
        .args(["all", "--zip", "/nonexistent-x80-oracle-test.zip"])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(2));
    assert!(String::from_utf8(out.stderr)
        .unwrap()
        .contains("fetch-oracles.sh fetch"));
}

#[test]
fn verifies_archive_pin_before_parsing() {
    let path = std::env::temp_dir().join(format!("x80-z80test-cli-{}.zip", std::process::id()));
    std::fs::write(&path, b"not a ZIP").unwrap();
    let out = command()
        .arg("z80full")
        .arg("--zip")
        .arg(&path)
        .output()
        .unwrap();
    std::fs::remove_file(path).unwrap();
    assert_eq!(out.status.code(), Some(1));
    assert!(String::from_utf8(out.stderr)
        .unwrap()
        .contains("ZIP SHA-256 mismatch"));
    assert!(out.stdout.is_empty());
}

#[test]
fn rejects_oversized_archive_before_hashing() {
    let path =
        std::env::temp_dir().join(format!("x80-z80test-oversize-{}.zip", std::process::id()));
    // A sparse file far larger than the pin exercises the CLI's bounded read.
    std::fs::File::create(&path)
        .unwrap()
        .set_len(1 << 30)
        .unwrap();
    let out = command()
        .arg("z80full")
        .arg("--zip")
        .arg(&path)
        .output()
        .unwrap();
    std::fs::remove_file(path).unwrap();
    assert_eq!(out.status.code(), Some(1));
    let stderr = String::from_utf8(out.stderr).unwrap();
    assert!(stderr.contains("ZIP exceeds pinned archive size"));
    assert!(!stderr.contains("SHA-256"));
    assert!(out.stdout.is_empty());
}
