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
    // Scope: literal manual identifiers in these crate-root source files,
    // including comments and strings. This does not parse Rust or follow modules.
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    for file in [
        "lib.rs",
        "bus.rs",
        "control.rs",
        "budget.rs",
        "exit.rs",
        "decode.rs",
        "progress.rs",
    ] {
        let source = std::fs::read_to_string(root.join(file)).expect("read crate-root source");
        for citation in [
            "98-153B", "9800301", "205775", "231718", "UM008011", "03-0029", "03-0002",
        ] {
            assert!(
                !source.contains(citation),
                "{file} contains manual citation {citation}"
            );
        }
    }
}

#[test]
fn intel_family_sources_cite_both_models_and_use_no_model_paths() {
    // Decision-16: inspect literal identifiers/paths, including comments and
    // strings, in src/intel/*.rs. This does not parse Rust or follow modules;
    // reviewers still check that both primary citations accompany each fact.
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src/intel");
    let files = match std::fs::read_dir(&root) {
        Ok(files) => files,
        // The planned family tier has no files yet: pass vacuously until it
        // is introduced. An empty directory likewise has no files to check.
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return,
        Err(error) => panic!("read {}: {error}", root.display()),
    };
    for file in files {
        let path = file.expect("read Intel-family directory entry").path();
        if path.extension().is_none_or(|extension| extension != "rs") {
            continue;
        }
        let source = std::fs::read_to_string(&path).expect("read Intel-family source");
        for citation in ["98-153B", "205775-002"] {
            assert!(
                source.contains(citation),
                "{} is missing primary citation {citation}",
                path.display()
            );
        }
        for forbidden in [
            "UM008011",
            "03-0029",
            "crate::i8080",
            "crate::i8085",
            "crate::z80",
        ] {
            assert!(
                !source.contains(forbidden),
                "{} contains forbidden identifier or model path {forbidden}",
                path.display()
            );
        }
    }
}
