// Compile the private protocol module here to exercise its synthetic seams
// without publishing test-only APIs or depending on the fetched GPL artifacts.
#[path = "../src/fingerprints.rs"]
mod fingerprints;
#[path = "../src/z80_oracles.rs"]
mod z80_protocol;
use fingerprints::fingerprint;

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
    for name in ["8080EXM.COM", "8080PRE.COM", "8085EXM.COM", "renamed.COM"] {
        let path = directory.join(name);
        std::fs::write(&path, &program).unwrap();
        for options in [
            vec![],
            vec!["--oracle", "exm"],
            vec!["--oracle", "pre"],
            vec!["--model", "8085"],
            vec!["--oracle", "exm", "--model", "8085"],
            vec!["--model", "8085", "--oracle", "pre"],
        ] {
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

#[test]
fn help_describes_model_pairing_and_invalid_model_is_rejected() {
    let result = Command::new(env!("CARGO_BIN_EXE_x80-oracle"))
        .arg("--help")
        .output()
        .unwrap();
    assert!(result.status.success());
    let help = String::from_utf8(result.stdout).unwrap();
    for message in [
        "--model 8080|8085|z80",
        "defaults to 8080",
        "8080EXM requires 8080",
        "8085EXM requires 8085",
        "8080PRE supports either",
        "before execution",
    ] {
        assert!(help.contains(message));
    }
    for arguments in [vec!["--model"], vec!["--model", "8086"]] {
        let result = Command::new(env!("CARGO_BIN_EXE_x80-oracle"))
            .args(arguments)
            .output()
            .unwrap();
        assert_eq!(result.status.code(), Some(1));
        assert!(result.stdout.is_empty());
        assert!(String::from_utf8(result.stderr)
            .unwrap()
            .contains("--model requires 8080, 8085 or z80"));
    }
}

#[test]
fn z80_unknown_digests_are_refused_before_any_guest_output() {
    let directory = std::env::temp_dir().join(format!("x80-oracle-z80-{}", std::process::id()));
    std::fs::create_dir(&directory).unwrap();
    // Prints if run, but filename and --oracle cannot confer a pinned identity.
    let program = [0x1e, b'!', 0x0e, 2, 0xcd, 5, 0, 0xc3, 0, 0];
    for name in ["prelim.com", "zexdoc.com", "zexall.com", "renamed.com"] {
        let path = directory.join(name);
        std::fs::write(&path, program).unwrap();
        for options in [vec![], vec!["--oracle", "pre"], vec!["--oracle", "exm"]] {
            let result = Command::new(env!("CARGO_BIN_EXE_x80-oracle"))
                .args(["--model", "z80"])
                .arg(&path)
                .args(options)
                .output()
                .unwrap();
            assert_eq!(result.status.code(), Some(1));
            assert!(result.stdout.is_empty());
            assert!(String::from_utf8(result.stderr)
                .unwrap()
                .contains("refuses unknown digest"));
        }
    }
    std::fs::remove_dir_all(directory).unwrap();
}

#[test]
fn intel_models_keep_console_output_with_model_specific_rendezvous_costs() {
    let directory =
        std::env::temp_dir().join(format!("x80-oracle-old-models-{}", std::process::id()));
    std::fs::create_dir_all(&directory).unwrap();
    let path = directory.join("synthetic.com");
    for model in ["8080", "8085"] {
        std::fs::write(&path, [0x1e, b'!', 0x0e, 2, 0xcd, 5, 0, 0xc3, 0, 0]).unwrap();
        let result = Command::new(env!("CARGO_BIN_EXE_x80-oracle"))
            .args(["--model", model, "--slice-cycles", "1"])
            .arg(&path)
            .output()
            .unwrap();
        assert_eq!(result.status.code(), Some(1));
        assert_eq!(result.stdout, b"!");
        // Decision-18: 8085 uses an extra OUT (one step/10 states).
        let (steps, cycles) = if model == "8080" { (6, 58) } else { (7, 67) };
        assert_eq!(String::from_utf8(result.stderr).unwrap(), format!(
            "\nsummary: steps={steps} cycles={cycles} exit=normal-exit (Halt PC=0001) verdict=unknown program, no verdict\n"));
        std::fs::write(&path, [0x08, 0x76]).unwrap();
        let result = Command::new(env!("CARGO_BIN_EXE_x80-oracle"))
            .args(["--model", model])
            .arg(&path)
            .output()
            .unwrap();
        assert_eq!(result.status.code(), Some(1));
        assert!(result.stdout.is_empty());
        let expected = if model == "8080" {
            "\nsummary: steps=0 cycles=0 exit=unsupported PC=0100 opcode=08 verdict=unknown program, no verdict\n"
        } else {
            // Decision-18: 08h is DSUB; the following HLT executes too.
            "\nsummary: steps=2 cycles=15 exit=unexpected-halt PC=0102 verdict=unknown program, no verdict\n"
        };
        assert_eq!(String::from_utf8(result.stderr).unwrap(), expected);
    }
    std::fs::remove_dir_all(directory).unwrap();
}

#[test]
fn i8085_bdos_preserves_stack_word_registers_and_flags_across_slices() {
    let directory =
        std::env::temp_dir().join(format!("x80-oracle-8085-abi-{}", std::process::id()));
    std::fs::create_dir(&directory).unwrap();
    let path = directory.join("synthetic.com");
    let mut program = vec![
        0x2a, 6, 0,    // LHLD 6: Data reads must still return FE00h.
        0xf9, // SPHL
        0x7c, 0xfe, 0xfe, // MOV A,H; CPI FEh
    ];
    let mut failures = Vec::new();
    // Every failed check jumps to an unexpected HLT, never the boot stub.
    let check = |program: &mut Vec<u8>, failures: &mut Vec<usize>, bytes: &[u8]| {
        program.extend_from_slice(bytes);
        program.push(0xc2); // JNZ failure
        failures.push(program.len());
        program.extend_from_slice(&[0, 0]);
    };
    check(&mut program, &mut failures, &[]);
    check(&mut program, &mut failures, &[0x7d, 0xfe, 0]); // low stack byte
    program.extend_from_slice(&[
        0x01, 0xf7, 0xa6, 0xc5, 0xf1, // load A=A6h, all seven flags set
        0x06, 0x42, 0x0e, 2, // B=42h, C=BDOS 2
        0x11, b'!', 0x34, // DE=3421h
        0x21, 0xef, 0xbe, // HL=BEEFh
        0xcd, 5, 0,    // CALL 5: OUT must preserve all registers/flags
        0xf5, // PUSH PSW before comparisons change flags
    ]);
    check(&mut program, &mut failures, &[0x78, 0xfe, 0x42]);
    check(&mut program, &mut failures, &[0x79, 0xfe, 2]);
    program.push(0xc1); // POP B, to inspect the saved A/flags
    check(&mut program, &mut failures, &[0x79, 0xfe, 0xf7]);
    check(&mut program, &mut failures, &[0x78, 0xfe, 0xa6]);
    check(&mut program, &mut failures, &[0x7a, 0xfe, 0x34]);
    check(&mut program, &mut failures, &[0x7b, 0xfe, b'!']);
    check(&mut program, &mut failures, &[0x7c, 0xfe, 0xbe]);
    check(&mut program, &mut failures, &[0x7d, 0xfe, 0xef]);
    // Repeated calls exercise clearing/rearming the host flag and BDOS 9.
    program.extend_from_slice(&[0x0e, 9, 0x11, 0, 0]);
    let string_address_at = program.len() - 2;
    program.extend_from_slice(&[0xcd, 5, 0, 0xcd, 5, 0, 0xc3, 0, 0]);
    let failure = (0x100 + program.len() as u16).to_le_bytes();
    program.push(0x76);
    let string = (0x100 + program.len() as u16).to_le_bytes();
    program.extend_from_slice(b"ok$");
    program[string_address_at..string_address_at + 2].copy_from_slice(&string);
    for at in failures {
        program[at..at + 2].copy_from_slice(&failure);
    }
    std::fs::write(&path, program).unwrap();
    let mut summary = None;
    for slice in ["1", "10", "17", "1000000"] {
        let result = Command::new(env!("CARGO_BIN_EXE_x80-oracle"))
            .args(["--model", "8085", "--slice-cycles", slice])
            .arg(&path)
            .output()
            .unwrap();
        assert_eq!(result.status.code(), Some(1)); // Unknown digest never passes.
        assert_eq!(result.stdout, b"!okok");
        let error = String::from_utf8(result.stderr).unwrap();
        assert!(
            error.contains("exit=normal-exit (Halt PC=0001) verdict=unknown program, no verdict"),
            "{error}"
        );
        if let Some(summary) = &summary {
            assert_eq!(&error, summary); // Stop polls contribute no steps/cycles.
        } else {
            summary = Some(error);
        }
    }
    std::fs::remove_dir_all(directory).unwrap();
}

#[test]
fn i8085_bdos_errors_ports_and_cap_at_out_boundary_remain_failures() {
    let directory =
        std::env::temp_dir().join(format!("x80-oracle-8085-errors-{}", std::process::id()));
    std::fs::create_dir(&directory).unwrap();
    let path = directory.join("synthetic.com");
    for (program, cap, expected) in [
        (
            vec![0x0e, 7, 0xcd, 5, 0],
            "1000",
            "steps=3 cycles=35 exit=host-error: unsupported BDOS function 7",
        ),
        // Even the reserved port must be reached through the private stub.
        (
            vec![0xd3, 0xff],
            "10",
            "steps=1 cycles=10 exit=host-error: unexpected output port FF",
        ),
        (
            vec![0xd3, 0xfe],
            "10",
            "steps=1 cycles=10 exit=host-error: unexpected output port FE",
        ),
        (
            vec![0xdb, 0xff],
            "10",
            "steps=1 cycles=10 exit=host-error: unexpected input port FF",
        ),
        // Budget wins over Stop: an OUT reaching the cap cannot service BDOS.
        (
            vec![0x1e, b'!', 0x0e, 2, 0xcd, 5, 0],
            "42",
            "steps=4 cycles=42 exit=timeout",
        ),
    ] {
        std::fs::write(&path, program).unwrap();
        for slice in ["1", "1000000"] {
            let result = Command::new(env!("CARGO_BIN_EXE_x80-oracle"))
                .args([
                    "--model",
                    "8085",
                    "--cycle-cap",
                    cap,
                    "--slice-cycles",
                    slice,
                ])
                .arg(&path)
                .output()
                .unwrap();
            assert_eq!(result.status.code(), Some(1));
            assert!(result.stdout.is_empty());
            let error = String::from_utf8(result.stderr).unwrap();
            assert_eq!(
                error,
                format!("\nsummary: {expected} verdict=unknown program, no verdict\n")
            );
        }
    }
    std::fs::remove_dir_all(directory).unwrap();
}

mod z80_protocol_unit {
    use super::fingerprint;
    use super::z80_protocol::*;
    use std::io;

    fn original_image() -> (Vec<u8>, [u64; 67]) {
        // Decision-20 §7: synthetic 67-record image; no GPL vectors/messages.
        let mut image = vec![0xa5; 0xc2 + 67 * 96];
        for (index, slot) in image[0x3a..0xc0]
            .as_chunks_mut::<2>()
            .0
            .iter_mut()
            .enumerate()
        {
            slot.copy_from_slice(&(0x01c2_u16 + index as u16 * 0x60).to_le_bytes());
        }
        image[0xc0..0xc2].fill(0);
        for index in 0..67 {
            image[0xc2 + index * 96..0xc2 + (index + 1) * 96].fill(index as u8);
        }
        let records = std::array::from_fn(|index| {
            fingerprint(&image[0xc2 + index * 96..0xc2 + (index + 1) * 96]).1
        });
        (image, records)
    }

    #[test]
    fn audits_all_67_records_without_filtering_or_mutation() {
        let (image, records) = original_image();
        let before = image.clone();
        audit_image(&image, &records).unwrap();
        audit_image(&image, &records).unwrap();
        assert_eq!(image, before);
        // Each real artifact has its own acceptance records, never a generic layout permit.
        assert!(audit_exerciser(&image, false).is_err());
        assert!(audit_exerciser(&image, true).is_err());
        assert_eq!(completed_group(b"synthetic descriptor  OK\n\r"), None);
        assert_eq!(completed_group(b"partial"), None);
    }

    #[test]
    fn every_pointer_terminator_and_record_byte_is_verified() {
        let (original, records) = original_image();
        for offset in 0x3a..original.len() {
            let mut image = original.clone();
            image[offset] ^= 1;
            let before = image.clone();
            let error = audit_image(&image, &records).unwrap_err();
            assert_eq!(error.kind(), io::ErrorKind::InvalidData);
            assert_eq!(image, before);
        }
        for len in [0, 0x39, 0x3a, 0xc0, 0xc1, 0xc2, original.len() - 1] {
            assert!(audit_image(&original[..len], &records).is_err());
        }
        let mut wrong_artifact = records;
        wrong_artifact[66] ^= 1;
        assert!(audit_image(&original, &wrong_artifact).is_err());
    }

    fn fixture() -> (Vec<(usize, u64)>, Grammar, Vec<String>) {
        let lines: Vec<_> = (0..67)
            .map(|i| format!("synthetic group {i:02}  OK"))
            .collect();
        let descriptors = lines
            .iter()
            .map(|line| fingerprint(&line.as_bytes()[..18]))
            .collect();
        let grammar = Grammar {
            banner: fingerprint(b"synthetic banner"),
            completion: fingerprint(b"synthetic complete"),
            ok: fingerprint(b"  OK"),
            error: fingerprint(b"  ERROR expected:"),
        };
        (descriptors, grammar, lines)
    }

    fn transcript(lines: &[String]) -> Vec<u8> {
        format!(
            "synthetic banner\n\r{}\n\rsynthetic complete",
            lines.join("\n\r")
        )
        .into_bytes()
    }

    #[test]
    fn grammar_judges_all_67_groups_and_rejects_failures_or_malformed_results() {
        let (descriptors, grammar, lines) = fixture();
        let accepts = |lines: &[String]| validate_groups(&transcript(lines), &descriptors, grammar);
        assert!(accepts(&lines).is_ok());
        assert!(validate_exerciser(&transcript(&lines)).is_err());
        for index in 0..67 {
            for suffix in [
                "  ERROR expected:1234abcd found:5678abcd", // Complete judged failure.
                "  ERROR expected:1234abcd",
                "  ERROR expected:1234abcd found:123",
                "  ERROR expected:1234ABCD found:5678abcd",
                "  OK trailing",
                "  O",
                "  PASS",
                "  OK  ERROR expected:1234abcd found:5678abcd",
            ] {
                let mut bad = lines.clone();
                bad[index].truncate(18);
                bad[index].push_str(suffix);
                let error = accepts(&bad).unwrap_err();
                if suffix == "  ERROR expected:1234abcd found:5678abcd" {
                    assert_eq!(
                        error,
                        format!("judged Z80 exerciser group {} reports ERROR", index + 1)
                    );
                }
            }
            let mut missing = lines.clone();
            missing.remove(index);
            assert!(accepts(&missing).is_err());
        }
        let mut reordered = lines.clone();
        reordered.swap(0, 1);
        assert!(accepts(&reordered).is_err());
        let mut bad = lines.clone();
        bad[66] = lines[65].clone();
        assert!(accepts(&bad).is_err());
        bad[66] = "unknown descriptor  OK".into();
        assert!(accepts(&bad).is_err());
        bad = lines.clone();
        bad.push(lines[0].clone());
        assert!(accepts(&bad).is_err());
    }

    #[test]
    fn grammar_requires_ordered_framing_and_consumes_all_nonempty_lines() {
        let (descriptors, grammar, lines) = fixture();
        let valid = transcript(&lines);
        for bad in [
            valid[18..].to_vec(),
            valid[..valid.len() - 18].to_vec(),
            [b"synthetic banner\n".as_slice(), &valid].concat(),
            [valid.as_slice(), b"\nsynthetic complete"].concat(),
            [valid.as_slice(), b"\nextra"].concat(),
            [valid.as_slice(), b"\nERROR"].concat(),
            [valid.as_slice(), b"\n\xff"].concat(),
            [valid.as_slice(), b" trailing"].concat(),
            b"synthetic banner\nsynthetic complete".to_vec(),
        ] {
            assert!(validate_groups(&bad, &descriptors, grammar).is_err());
        }
    }

    #[test]
    fn prelim_requires_exact_success_and_rejects_early_or_printed_errors() {
        assert!(validate_preliminary(b"Preliminary tests complete").is_ok());
        for bad in [
            b"".as_slice(),
            b"0123\r\n",
            b"Preliminary tests complete\n",
            b"0123\r\nPreliminary tests complete",
            b"Preliminary tests completejunk",
            b"\xff",
        ] {
            assert!(validate_preliminary(bad).is_err());
        }
    }
}
