//! Offline CP/M-style host; oracle artifacts are never embedded or downloaded.
//!
//! Host ABI references below name the pinned files in
//! `oracles/8080-exercisers/` (PROVENANCE.md §2). No oracle implementation is
//! incorporated here. The core contract is decision-5, items 3–8.

#![forbid(unsafe_code)]

mod sha256;

use std::io::{self, Write};

use x80_core::i8080::{Inputs, Latches, Registers, I8080};
use x80_core::{Budget, Bus, Control, Exit, MemoryAccess};

const LOAD: usize = 0x0100;
const STACK: u16 = 0xfe00;
const RENDEZVOUS: u8 = 0x08;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Oracle {
    Preliminary,
    Exerciser,
}

impl Oracle {
    /// Identify only the pinned program bytes, independently of their filename.
    pub fn identify(program: &[u8]) -> Option<Self> {
        // Mirror oracles/SHA256SUMS.fetched at oracles commit c84b982.
        match sha256::digest(program).as_str() {
            "18eb3c79cba42c0718f160be6a1853cb64cdce7aa47d65780189a57bdd98c4e0" => {
                Some(Self::Preliminary)
            }
            "6e3286e11bb1a8f47b8ee1280b4a067be813193363e3223c99b0d21912f44aeb" => {
                Some(Self::Exerciser)
            }
            _ => None,
        }
    }

    fn accepts(self, output: &[u8]) -> bool {
        match self {
            // 8080PRE.MAC:216–221 announces completion of the preliminary
            // checks. Match its complete message using a length and FNV-1a
            // fingerprint, without reproducing the source's message text.
            Self::Preliminary => output.split(|&b| b == b'\r' || b == b'\n').any(|line| {
                line.len() == 31
                    && line.iter().fold(0xcbf29ce484222325_u64, |hash, &byte| {
                        (hash ^ u64::from(byte)).wrapping_mul(0x100000001b3)
                    }) == 0xc52616baf4bf534f
            }),
            // 8080EXM.MAC:132–158 lists 25 groups; 436–481 and 956–958
            // print each group's PASS/ERROR verdict and checksum. Require
            // all 25 distinct groups, complete CRCs, and no error marker.
            // The program digest pins the group labels; do not embed them.
            Self::Exerciser => {
                let text = String::from_utf8_lossy(output);
                if text.contains("ERROR") {
                    return false;
                }
                let mut groups = std::collections::HashSet::new();
                let mut count = 0;
                for line in text.lines() {
                    if let Some((group, verdict)) = line.split_once("PASS") {
                        let Some((_, crc)) = verdict.rsplit_once(':') else {
                            return false;
                        };
                        if group.trim().is_empty()
                            || crc.len() != 8
                            || !crc.bytes().all(|b| b.is_ascii_hexdigit())
                            || !groups.insert(group.trim())
                        {
                            return false;
                        }
                        count += 1;
                    }
                }
                count == 25
            }
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Config {
    pub cycle_cap: u64,
    pub slice_cycles: u64,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            cycle_cap: 100_000_000_000,
            slice_cycles: 1_000_000,
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum Reason {
    NormalExit,
    UnexpectedHalt { pc: u16 },
    Unsupported { pc: u16, opcode: u8 },
    Timeout,
    HostError(String),
}

#[derive(Debug)]
pub struct Outcome {
    pub steps: u64,
    pub cycles: u64,
    pub reason: Reason,
    /// Unknown digests have no verdict; known programs pass only on normal exit.
    pub verdict: Option<bool>,
    pub output: Vec<u8>,
}

impl Outcome {
    pub fn passed(&self) -> bool {
        self.reason == Reason::NormalExit && self.verdict == Some(true)
    }
}

struct Host<'a, W> {
    ram: Box<[u8; 65536]>,
    console: &'a mut W,
    output: Vec<u8>,
    arguments: Option<(u8, u16)>,
    awaiting_registers: bool,
    error: Option<String>,
}

impl<W: Write> Host<'_, W> {
    fn print(&mut self, bytes: &[u8]) {
        self.output.extend_from_slice(bytes);
        if let Err(error) = self
            .console
            .write_all(bytes)
            .and_then(|()| self.console.flush())
        {
            self.error = Some(format!("console write failed: {error}"));
        }
    }

    fn bdos(&mut self, function: u8, address: u16) {
        // 8080PRE.MAC:216–221,260–271; 8080EXM.MAC:924–958:
        // function 2 consumes E, function 9 consumes a dollar-ended DE string.
        match function {
            2 => self.print(&[address as u8]),
            9 => {
                let mut bytes = Vec::new();
                let mut pointer = address;
                for _ in 0..65536 {
                    let byte = self.ram[usize::from(pointer)];
                    if byte == b'$' {
                        self.print(&bytes);
                        return;
                    }
                    bytes.push(byte);
                    pointer = pointer.wrapping_add(1);
                }
                self.error = Some("BDOS 9 string has no terminator in 64 KiB".into());
            }
            _ => self.error = Some(format!("unsupported BDOS function {function}")),
        }
    }
}

impl<W: Write> Bus for Host<'_, W> {
    fn read(&mut self, address: u16, kind: MemoryAccess) -> u8 {
        if address == 5 && kind == MemoryAccess::OpcodeFetch {
            if let Some((function, pointer)) = self.arguments.take() {
                self.bdos(function, pointer);
            } else {
                // Bus cannot borrow CPU registers. A private fetch response
                // rendezvous returns through decision-5 item 6: no instruction,
                // cycles or register changes. The runner supplies C/DE and
                // retries this fetch, servicing BDOS before the installed RET.
                self.awaiting_registers = true;
                return RENDEZVOUS;
            }
        }
        self.ram[usize::from(address)]
    }

    fn write(&mut self, address: u16, value: u8, _kind: MemoryAccess) {
        self.ram[usize::from(address)] = value;
    }

    fn input(&mut self, port: u8) -> u8 {
        self.error = Some(format!("unexpected input port {port:02X}"));
        0
    }

    fn output(&mut self, port: u8, _value: u8) {
        self.error = Some(format!("unexpected output port {port:02X}"));
    }
}

impl<W> Control for Host<'_, W> {}
impl<W> Inputs for Host<'_, W> {}

pub fn run<W: Write>(program: &[u8], config: Config, console: &mut W) -> io::Result<Outcome> {
    if program.len() > 65536 - LOAD || program.is_empty() || config.slice_cycles == 0 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "invalid program size or zero slice",
        ));
    }
    let oracle = Oracle::identify(program);
    let mut host = Host {
        ram: Box::new([0; 65536]),
        console,
        output: Vec::new(),
        arguments: None,
        awaiting_registers: false,
        error: None,
    };
    host.ram[LOAD..LOAD + program.len()].copy_from_slice(program);
    // Host stubs implement the CALL 5 / JMP 0 ABI, rather than CPU facts.
    // 8080PRE.MAC:44,52,55,216–219,264–266: initial CALL needs a stack;
    // nonzero HLT is an emergency. 8080EXM.MAC:53,112–116,127–130:
    // load at 0100h, get stack from word 6, and return to the boot entry.
    host.ram[0] = 0x76;
    host.ram[5] = 0xc9;
    host.ram[6..8].copy_from_slice(&STACK.to_le_bytes());
    let mut cpu = I8080::new(
        Registers {
            pc: LOAD as u16,
            sp: STACK,
            ..Registers::default()
        },
        Latches::default(),
    );
    let mut steps = 0_u64;
    let mut cycles = 0_u64;
    let reason = loop {
        // HLT can retire exactly at a budget boundary, when Budget wins over
        // Halt. Observe the latch before declaring a timeout (decision-5 §5).
        if cpu.latches().halted {
            break if cpu.registers().pc == 1 {
                Reason::NormalExit
            } else {
                Reason::UnexpectedHalt {
                    pc: cpu.registers().pc,
                }
            };
        }
        if cycles >= config.cycle_cap {
            break Reason::Timeout;
        }
        let report = cpu.run(
            &mut host,
            Budget {
                steps: u64::MAX,
                cycles: config.slice_cycles.min(config.cycle_cap - cycles),
            },
        );
        steps = steps.saturating_add(report.steps);
        cycles = cycles.saturating_add(report.cycles);
        if let Some(error) = host.error.take() {
            break Reason::HostError(error);
        }
        match report.exit {
            Exit::Budget | Exit::Halt => {}
            Exit::Unsupported(unsupported)
                if host.awaiting_registers
                    && unsupported.pc == 5
                    && unsupported.opcode == RENDEZVOUS =>
            {
                host.awaiting_registers = false;
                host.arguments = Some((cpu.registers().c, cpu.registers().de()));
            }
            Exit::Unsupported(unsupported) => {
                break Reason::Unsupported {
                    pc: unsupported.pc,
                    opcode: unsupported.opcode,
                }
            }
            other => break Reason::HostError(format!("unexpected core exit: {other:?}")),
        }
    };
    let verdict = oracle.map(|oracle| reason == Reason::NormalExit && oracle.accepts(&host.output));
    Ok(Outcome {
        steps,
        cycles,
        reason,
        verdict,
        output: host.output,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn execute(program: &[u8], cycle_cap: u64, slice_cycles: u64) -> Outcome {
        run(
            program,
            Config {
                cycle_cap,
                slice_cycles,
            },
            &mut Vec::new(),
        )
        .unwrap()
    }

    #[test]
    fn prints_bdos_9_and_2_then_jumps_to_zero() {
        let program = [
            0x11, 0x12, 0x01, // LXI D,0112h
            0x0e, 9, 0xcd, 5, 0, // MVI C,9; CALL 5
            0x1e, b'!', 0x0e, 2, 0xcd, 5, 0, // MVI E,'!'; MVI C,2; CALL 5
            0xc3, 0, 0, // JMP 0
            b'H', b'i', b'$',
        ];
        let result = execute(&program, 1000, 17);
        assert_eq!(result.output, b"Hi!");
        assert_eq!(result.reason, Reason::NormalExit);
        assert_eq!(result.steps, 10); // Internal rendezvous fetches do not count.
        assert_eq!(result.cycles, 102);
        assert_eq!(result.verdict, None);
        assert!(!result.passed()); // A normal exit alone is not success.
    }

    #[test]
    fn unexpected_halt_is_failure_even_at_slice_boundary() {
        let result = execute(&[0x76], 7, 7);
        assert_eq!(result.reason, Reason::UnexpectedHalt { pc: 0x101 });
        assert!(!result.passed());
    }

    #[test]
    fn undocumented_byte_reports_pc_and_opcode() {
        let result = execute(&[0x08], 100, 100);
        assert_eq!(
            result.reason,
            Reason::Unsupported {
                pc: 0x100,
                opcode: 8
            }
        );
        assert_eq!((result.steps, result.cycles), (0, 0));
    }

    #[test]
    fn timeout_accumulates_slices_with_instruction_overshoot() {
        let result = execute(&[0xc3, 0, 1], 35, 11);
        assert_eq!(result.reason, Reason::Timeout);
        assert_eq!((result.steps, result.cycles), (4, 40));
        assert!(!result.passed());
    }

    #[test]
    fn zero_cap_executes_nothing() {
        let result = execute(&[0x76], 0, 10);
        assert_eq!(result.reason, Reason::Timeout);
        assert_eq!((result.steps, result.cycles), (0, 0));
    }

    #[test]
    fn unknown_bdos_function_is_reported() {
        let result = execute(&[0x0e, 7, 0xcd, 5, 0, 0xc3, 0, 0], 1000, 10);
        assert_eq!(
            result.reason,
            Reason::HostError("unsupported BDOS function 7".into())
        );
    }

    #[test]
    fn exerciser_requires_all_distinct_complete_passes_and_no_error() {
        let mut output = String::new();
        for group in 0..25 {
            output.push_str(&format!(
                "synthetic group {group} PASS checksum:1234abcd\n\r"
            ));
        }
        assert!(Oracle::Exerciser.accepts(output.as_bytes()));
        assert!(!Oracle::Exerciser.accepts(b""));
        assert!(!Oracle::Exerciser.accepts(output.lines().next().unwrap().as_bytes()));
        assert!(!Oracle::Exerciser.accepts(output.replace("1234abcd", "1234abc").as_bytes()));
        assert!(!Oracle::Exerciser.accepts(output.replace("group 24", "group 23").as_bytes()));
        assert!(!Oracle::Exerciser
            .accepts(format!("{output}extra group PASS checksum:1234abcd\n").as_bytes()));
        output.push_str("ERROR");
        assert!(!Oracle::Exerciser.accepts(output.as_bytes()));
    }

    #[test]
    fn invalid_program_or_slice_is_rejected() {
        for (program, slice) in [(&[][..], 1), (&[0x76][..], 0), (&[0; 65536][..], 1)] {
            assert!(run(
                program,
                Config {
                    cycle_cap: 100,
                    slice_cycles: slice
                },
                &mut Vec::new()
            )
            .is_err());
        }
    }
}
