//! Architectural boundary tripwire: plan §2.1 and PROVENANCE.md §4.
//! Cargo resolves every dependency form, including optional and target-specific
//! normal, build and development dependencies; no TOML parser crate is needed.

use std::process::Command;

#[test]
fn core_has_no_dependencies() {
    let output = Command::new(env!("CARGO"))
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .args([
            "tree",
            "--offline",
            "--locked",
            "--package",
            "x80-core",
            "--all-features",
            "--target",
            "all",
            "--edges",
            "normal,build,dev",
            "--depth",
            "1",
            "--prefix",
            "none",
            "--color",
            "never",
        ])
        .output()
        .expect("run Cargo's dependency boundary check");
    assert!(
        output.status.success(),
        "cargo tree failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let tree = String::from_utf8(output.stdout).expect("Cargo emits UTF-8");
    let mut lines = tree.lines().filter(|line| !line.trim().is_empty());
    assert!(
        lines
            .next()
            .is_some_and(|line| line.starts_with("x80-core v")),
        "expected x80-core as dependency-tree root: {tree}"
    );
    assert!(
        lines.next().is_none(),
        "x80-core must have zero dependencies of any kind; put machine services \
         and oracle tooling in embedders or separate harness crates. Found:\n{tree}"
    );
}

#[test]
fn crate_root_sources_cite_no_manual() {
    // Scope: literal manual identifiers in these five crate-root source files,
    // including comments and strings. This does not parse Rust or follow modules.
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    for file in ["lib.rs", "bus.rs", "control.rs", "budget.rs", "exit.rs"] {
        let source = std::fs::read_to_string(root.join(file)).expect("read crate-root source");
        for citation in ["98-153B", "9800301"] {
            assert!(
                !source.contains(citation),
                "{file} contains manual citation {citation}"
            );
        }
    }
}
