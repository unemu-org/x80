use std::process::Command;

#[test]
fn synthetic_25_passes_are_unknown_even_with_oracle_names_and_overrides() {
    let directory =
        std::env::temp_dir().join(format!("x80-oracle-synthetic-{}", std::process::id()));
    std::fs::create_dir(&directory).unwrap();
    let mut output = String::new();
    for group in 0..25 {
        output.push_str(&format!(
            "synthetic group {group} PASS checksum:1234abcd\r\n"
        ));
    }
    let mut program = vec![
        0x11, 0x0b, 0x01, // LXI D,010Bh
        0x0e, 9, 0xcd, 5, 0, // MVI C,9; CALL 5
        0xc3, 0, 0, // JMP 0
    ];
    program.extend_from_slice(output.as_bytes());
    program.push(b'$');
    for name in ["8080EXM.COM", "8080PRE.COM", "renamed.COM"] {
        let path = directory.join(name);
        std::fs::write(&path, &program).unwrap();
        for options in [vec![], vec!["--oracle", "exm"], vec!["--oracle", "pre"]] {
            let result = Command::new(env!("CARGO_BIN_EXE_x80-oracle"))
                .arg(&path)
                .args(options)
                .output()
                .unwrap();
            assert_eq!(result.status.code(), Some(1));
            assert_eq!(result.stdout, output.as_bytes());
            let error = String::from_utf8(result.stderr).unwrap();
            assert!(error.contains("exit=normal-exit (Halt PC=0001)"));
            assert!(error.contains("verdict=unknown program, no verdict"));
            assert!(!error.contains("verdict=PASS"));
        }
    }
    std::fs::remove_dir_all(directory).unwrap();
}

#[test]
fn missing_file_exits_two_with_fetch_prerequisite() {
    let directory = std::env::temp_dir().join(format!("x80-oracle-missing-{}", std::process::id()));
    std::fs::create_dir(&directory).unwrap();
    let result = Command::new(env!("CARGO_BIN_EXE_x80-oracle"))
        .arg(directory.join("8080PRE.COM"))
        .output()
        .unwrap();
    std::fs::remove_dir(directory).unwrap();
    assert_eq!(result.status.code(), Some(2));
    assert!(result.stdout.is_empty());
    let error = String::from_utf8(result.stderr).unwrap();
    assert!(error.contains("./oracles/fetch-oracles.sh fetch"));
    assert!(error.contains("steps=0 cycles=0 exit=missing-file verdict=FAIL"));
}
