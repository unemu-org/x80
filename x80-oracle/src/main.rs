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
                    _ => return Err("--model requires 8080 or 8085".into()),
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
                println!("Usage: x80-oracle [--model 8080|8085] [--cycle-cap N] [--slice-cycles N] [--oracle pre|exm] <program.COM>");
                println!("Model defaults to 8080. 8080EXM requires 8080; 8085EXM requires 8085; 8080PRE supports either. A known program/model mismatch is an error before execution.");
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
        (false, _) => Oracle::Preliminary,
        (true, Model::I8080) => Oracle::Exerciser,
        (true, Model::I8085) => Oracle::Exerciser8085,
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
        let output = String::from_utf8_lossy(&outcome.output);
        if let Some(line) = oracle.reported_not_judged(&output) {
            eprintln!("\nreported, not judged: {}", line.trim());
        }
    }
    if let Some(error) = &outcome.verdict_error {
        eprintln!("\nverdict failure: {error}");
    }
    let reason = match &outcome.reason {
        Reason::NormalExit => "normal-exit (Halt PC=0001)".into(),
        Reason::UnexpectedHalt { pc } => format!("unexpected-halt PC={pc:04X}"),
        Reason::Unsupported { pc, opcode } => {
            format!("unsupported PC={pc:04X} opcode={opcode:02X}")
        }
        Reason::Timeout => "timeout".into(),
        Reason::HostError(error) => format!("host-error: {error}"),
    };
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
