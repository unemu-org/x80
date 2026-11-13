#![forbid(unsafe_code)]

use std::ffi::OsString;
use std::io::{self, Write};
use std::path::PathBuf;
use std::process::ExitCode;

use x80_oracle::{run, Config, Model, Oracle, Reason};

fn main() -> ExitCode {
    match execute(std::env::args_os().skip(1).collect()) {
        Ok(code) => ExitCode::from(code),
        Err(message) => {
            eprintln!("{message}");
            ExitCode::FAILURE
        }
    }
}

fn execute(args: Vec<OsString>) -> Result<u8, String> {
    if args.first().is_some_and(|arg| arg == "z80test") {
        return x80_oracle::z80test::execute(args.into_iter().skip(1).collect());
    }
    let mut config = Config::default();
    let mut path = None;
    let mut oracle = None;
    let mut args = args.into_iter();
    while let Some(arg) = args.next() {
        match arg.to_str() {
            Some("--cycle-cap" | "--slice-cycles") => {
                let value = args
                    .next()
                    .and_then(|v| v.to_str().and_then(|s| s.parse::<u64>().ok()))
                    .ok_or("cycle limits require an unsigned integer")?;
                if arg == "--cycle-cap" {
                    config.cycle_cap = value;
                } else {
                    config.slice_cycles = value;
                }
            }
            Some("--model") => {
                config.model = match args.next().as_deref().and_then(|v| v.to_str()) {
                    Some("8080") => Model::I8080,
                    Some("8085") => Model::I8085,
                    Some("z80") => Model::Z80,
                    _ => return Err("--model requires 8080, 8085 or z80".into()),
                };
            }
            Some("--oracle") => {
                oracle = Some(match args.next().as_deref().and_then(|v| v.to_str()) {
                    Some("pre") => false,
                    Some("exm") => true,
                    _ => return Err("--oracle requires pre or exm".into()),
                });
            }
            Some("--help" | "-h") => {
                println!("Usage: x80-oracle [--model 8080|8085|z80] [--cycle-cap N] [--slice-cycles N] [--oracle pre|exm] <program.COM>");
                println!("       x80-oracle z80test --help");
                println!("Model defaults to 8080. 8080EXM requires 8080; 8085EXM requires 8085; 8080PRE supports either. Z80 requires pinned prelim.com or zexdoc.com; unknown Z80 digests are refused. A known program/model mismatch is an error before execution.");
                return Ok(0);
            }
            Some(value) if value.starts_with('-') => {
                return Err(format!("unknown option: {value}"))
            }
            _ if path.is_none() => path = Some(PathBuf::from(arg)),
            _ => return Err("expected one program path".into()),
        }
    }
    let path = path.ok_or("expected <program.COM>; use --help for usage")?;
    let program = match std::fs::read(&path) {
        Ok(program) => program,
        Err(error) if error.kind() == io::ErrorKind::NotFound => {
            eprintln!("program is missing; run ./oracles/fetch-oracles.sh fetch");
            eprintln!("summary: steps=0 cycles=0 exit=missing-file verdict=FAIL");
            return Ok(2);
        }
        Err(error) => return Err(format!("cannot read program: {error}")),
    };
    let oracle = oracle.map(|exm| match (exm, config.model) {
        (false, Model::Z80) => Oracle::PreliminaryZ80,
        (false, _) => Oracle::Preliminary,
        (true, Model::I8080) => Oracle::Exerciser,
        (true, Model::I8085) => Oracle::Exerciser8085,
        (true, Model::Z80) => Oracle::ExerciserZ80,
    });
    if let (Some(expected), Some(actual)) = (oracle, Oracle::identify(&program)) {
        if expected != actual {
            return Err("--oracle does not match the program digest".into());
        }
    }
    let stdout = io::stdout();
    let mut console = stdout.lock();
    let outcome = run(&program, config, &mut console).map_err(|error| error.to_string())?;
    console.flush().map_err(|error| error.to_string())?;
    if let Some(oracle) = Oracle::identify(&program) {
        let not_run = oracle.not_run();
        if !not_run.is_empty() {
            // The guest's last line may lack a newline; keep the list on its own lines.
            eprintln!();
        }
        for label in not_run {
            eprintln!("{label}: not run (decision-14)");
        }
        let output = String::from_utf8_lossy(&outcome.output);
        if let Some(line) = oracle.reported_not_judged(&output) {
            eprintln!("\nreported, not judged: {}", line.trim());
        }
    }
    if let Some(error) = &outcome.verdict_error {
        eprintln!("\nverdict failure: {error}");
    }
    let reason = format_reason(&outcome.reason, config.model);
    eprintln!(
        "\nsummary: steps={} cycles={} exit={} verdict={}",
        outcome.steps,
        outcome.cycles,
        reason,
        match outcome.verdict {
            Some(true) => "PASS",
            Some(false) => "FAIL",
            None => "unknown program, no verdict",
        }
    );
    Ok(if outcome.passed() { 0 } else { 1 })
}

fn format_reason(reason: &Reason, model: Model) -> String {
    match reason {
        Reason::NormalExit => "normal-exit (Halt PC=0001)".into(),
        Reason::UnexpectedHalt { pc } => format!("unexpected-halt PC={pc:04X}"),
        Reason::Unsupported {
            pc,
            opcode,
            preceding,
        } => {
            let mut text = format!("unsupported PC={pc:04X} opcode={opcode:02X}");
            if model == Model::Z80 {
                let bytes = preceding
                    .iter()
                    .map(|byte| format!("{byte:02X}"))
                    .collect::<Vec<_>>()
                    .join(" ");
                text.push_str(&format!(" preceding=[{bytes}]"));
            }
            text
        }
        Reason::Timeout => "timeout".into(),
        Reason::HostError(error) => format!("host-error: {error}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn refusal_diagnostics_show_all_preceding_z80_bytes_only() {
        for preceding in [vec![0xed], vec![0xfd, 0xcb, 0x80], vec![]] {
            let reason = Reason::Unsupported {
                pc: 0x1234,
                opcode: 0x70,
                preceding: preceding.clone(),
            };
            let expected = match preceding.as_slice() {
                [0xed] => "ED",
                [0xfd, 0xcb, 0x80] => "FD CB 80",
                [] => "",
                _ => unreachable!(),
            };
            assert_eq!(
                format_reason(&reason, Model::Z80),
                format!("unsupported PC=1234 opcode=70 preceding=[{expected}]")
            );
            for model in [Model::I8080, Model::I8085] {
                assert_eq!(
                    format_reason(&reason, model),
                    "unsupported PC=1234 opcode=70"
                );
            }
        }
    }
}
