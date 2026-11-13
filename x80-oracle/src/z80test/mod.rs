//! Fetch-only z80test v1.2a Spectrum-style host (decision-20 §7).
//! No members are embedded/extracted; ZIP identity precedes all parsing.

mod host;
mod output;
mod tap;
mod zip;

use std::{
    ffi::OsString,
    io::{Read, Write},
    path::PathBuf,
    time::Duration,
};

type Result<T> = std::result::Result<T, String>;
const ZIP_SHA256: &str = "7df0443d703e6b3114ea04b4cdef3e13b91421c62e37185c1036d06864cacbaf";
const ZIP_SIZE: u64 = 61_595;

fn read_zip(reader: impl Read) -> Result<Vec<u8>> {
    let mut bytes = Vec::new();
    reader
        .take(ZIP_SIZE + 1)
        .read_to_end(&mut bytes)
        .map_err(|e| format!("cannot read z80test ZIP: {e}"))?;
    if bytes.len() as u64 > ZIP_SIZE {
        return Err("z80test ZIP exceeds pinned archive size".into());
    }
    Ok(bytes)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Variant {
    Full,
    Doc,
    Flags,
    DocFlags,
    Ccf,
    Memptr,
}
impl Variant {
    const ALL: [Self; 6] = [
        Self::Full,
        Self::Doc,
        Self::Flags,
        Self::DocFlags,
        Self::Ccf,
        Self::Memptr,
    ];
    fn name(self) -> &'static str {
        match self {
            Self::Full => "z80full",
            Self::Doc => "z80doc",
            Self::Flags => "z80flags",
            Self::DocFlags => "z80docflags",
            Self::Ccf => "z80ccf",
            Self::Memptr => "z80memptr",
        }
    }
    fn label(self) -> &'static str {
        match self {
            Self::Full => "full",
            Self::Doc => "doc",
            Self::Flags => "flags",
            Self::DocFlags => "doc flags",
            Self::Ccf => "CCF",
            Self::Memptr => "MEMPTR",
        }
    }
}

#[derive(Clone, Copy)]
struct Config {
    step_cap: u64,
    time_cap: Duration,
}
impl Default for Config {
    fn default() -> Self {
        Self {
            step_cap: 10_000_000_000,
            time_cap: Duration::from_secs(600),
        }
    }
}

/// CLI entry point, kept separate from the existing CP/M model paths.
pub fn execute(args: Vec<OsString>) -> Result<u8> {
    let mut path = PathBuf::from("oracles/z80-exercisers/z80test-1.2a.zip");
    let mut variants = Variant::ALL.to_vec();
    let mut selected = false;
    let mut config = Config::default();
    let mut args = args.into_iter();
    while let Some(arg) = args.next() {
        match arg.to_str() {
            Some("--zip") => path = PathBuf::from(args.next().ok_or("--zip needs a path")?),
            Some("--step-cap" | "--time-cap") => {
                let value = args
                    .next()
                    .and_then(|s| s.to_str().and_then(|s| s.parse::<u64>().ok()))
                    .filter(|&n| n > 0)
                    .ok_or("budgets require a nonzero unsigned integer")?;
                if arg == "--step-cap" {
                    config.step_cap = value;
                } else {
                    config.time_cap = Duration::from_secs(value);
                }
            }
            Some("--help" | "-h") => {
                println!("Usage: x80-oracle z80test [all|z80full|z80doc|z80flags|z80docflags|z80ccf|z80memptr] [--zip PATH] [--step-cap N] [--time-cap SECONDS]");
                println!("Defaults: all, pinned fetch-only ZIP, 10000000000 steps and 600 seconds per variant. Results are non-final baseline observations, never conformance.");
                return Ok(0);
            }
            Some(value) if !selected => {
                variants = if value == "all" {
                    Variant::ALL.to_vec()
                } else {
                    vec![*Variant::ALL
                        .iter()
                        .find(|v| v.name() == value)
                        .ok_or("unknown z80test variant (z80ccfscr is out of scope)")?]
                };
                selected = true;
            }
            _ => return Err("unknown or duplicate z80test argument; use z80test --help".into()),
        }
    }
    let file = match std::fs::File::open(path) {
        Ok(f) => f,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            eprintln!("z80test ZIP missing; run ./oracles/fetch-oracles.sh fetch");
            return Ok(2);
        }
        Err(e) => return Err(format!("cannot read z80test ZIP: {e}")),
    };
    let bytes = read_zip(file)?;
    let archive = zip::Archive::pinned(&bytes)?;
    eprintln!("z80test v1.2a ZIP SHA-256 verified; non-final baseline, no conformance claim. INT/NMI unasserted; input FE=BF after OUT FE=07. No groups patched out.");
    // Audit every requested image before executing the first one.
    let images = variants
        .iter()
        .map(|&v| tap::load(&archive, v))
        .collect::<Result<Vec<_>>>()?;
    let mut passed = true;
    for (variant, image) in variants.into_iter().zip(images) {
        eprintln!("{} audit: TAP SHA-256={} CODE=8000h length={} entry=8000h data=8800h table={:04X}h groups={} correspondence=OK (layout/descriptors/CRCs, no assembler rebuild)",variant.name(),image.tap_sha256,image.code.len(),image.table,image.groups.len());
        let run = host::run(&image.code, image.sites, config)?;
        let parsed = output::parse(&run.output, variant, &image.groups);
        let stdout = std::io::stdout();
        let mut out = stdout.lock();
        writeln!(out, "--- {} captured output ---", variant.name()).map_err(|e| e.to_string())?;
        // Tabs remain visible separators; no guest screen control reaches the terminal.
        writeln!(
            out,
            "{}",
            output::screen_text(&run.output).unwrap_or_else(|_| format!("{:02X?}", run.output))
        )
        .map_err(|e| e.to_string())?;
        out.flush().map_err(|e| e.to_string())?;
        for line in &parsed.lines {
            let kind = if line.diagnostic {
                "diagnostic, not judged"
            } else {
                "judged"
            };
            match line.status {
                output::Status::Failed { found, expected } => eprintln!(
                    "{} {:03} {}: FAILED CRC={found:08X} Expected={expected:08X} ({kind})",
                    variant.name(),
                    line.number,
                    line.name
                ),
                output::Status::Incheck { found } => eprintln!(
                    "{} {:03} {}: .incheck FAILED IN FE={found:02X} Expected=BF (not CRC-judged)",
                    variant.name(),
                    line.number,
                    line.name
                ),
                output::Status::Skipped => eprintln!(
                    "{} {:03} {}: Skipped ({kind})",
                    variant.name(),
                    line.number,
                    line.name
                ),
                output::Status::Ok if line.diagnostic => eprintln!(
                    "{} {:03} {}: OK ({kind})",
                    variant.name(),
                    line.number,
                    line.name
                ),
                _ => {}
            }
        }
        if let Some(error) = &parsed.error {
            eprintln!("{} output error: {error}", variant.name());
        }
        if let Some(pending) = &parsed.pending {
            eprintln!("{} incomplete: {pending}", variant.name());
        }
        let complete = run.reason == host::Reason::Returned && parsed.complete(image.groups.len());
        passed &= complete;
        let calls = run.channels + run.characters;
        eprintln!("{} MEASURED: groups_judged={} OK={} FAILED={} skipped={} diagnostic={} .incheck_failures={} completed_groups={}/{} final_counts={:?} steps={} cycles={} elapsed_ms={} exit={} baseline={} (not conformance)",variant.name(),parsed.judged(),parsed.ok(),parsed.failed(),parsed.skipped(),parsed.diagnostics(),parsed.incheck(),parsed.lines.len(),image.groups.len(),parsed.final_counts,run.steps,run.cycles,run.elapsed.as_millis(),run.reason,if complete {"complete, all judged OK"} else {"incomplete or failing"});
        eprintln!("{} MEASURED host: CHAN-OPEN={} RST10={} RET-response overhead_steps={} overhead_cycles={} (+1 step/+10 cycles per call; caller return +0); port_reads={} port_writes={}",variant.name(),run.channels,run.characters,calls,calls*10,run.reads,run.writes);
    }
    Ok(u8::from(!passed))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn archive_read_stops_at_pinned_size_plus_one() {
        // An endless reader must consume only the bound, without seeking or
        // trusting file metadata. An exact-sized input proceeds to pinning.
        let mut endless = std::io::repeat(0);
        let mut counted = (&mut endless).take(ZIP_SIZE + 10);
        assert_eq!(
            read_zip(&mut counted).unwrap_err(),
            "z80test ZIP exceeds pinned archive size"
        );
        assert_eq!(counted.limit(), 9);
        assert_eq!(
            read_zip(std::io::repeat(0).take(ZIP_SIZE)).unwrap().len() as u64,
            ZIP_SIZE
        );
    }
}
