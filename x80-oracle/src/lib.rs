//! Offline CP/M-style host; oracle artifacts are never embedded or downloaded.
//!
//! Host ABI references below name the pinned files in
//! `oracles/8080-exercisers/` and `oracles/z80-exercisers/` (PROVENANCE.md §2).
//! No oracle implementation is incorporated here. Core contracts are
//! decision-5, items 3–8, and decisions 14–15 for Z80.

#![forbid(unsafe_code)]

mod fingerprints;
mod sha256;
mod z80_oracles;
pub mod z80test;

use fingerprints::fingerprint;

use std::io::{self, Write};

use x80_core::{i8080, i8085, z80};
use x80_core::{Budget, Bus, Control, Exit, MemoryAccess};

const LOAD: usize = 0x0100;
const STACK: u16 = 0xfe00;
const RENDEZVOUS: u8 = 0x08;
// Private 8085 host ABI, not a CPU or CP/M hardware fact (decision-18).
const BDOS_PORT_8085: u8 = 0xff;

// Both pinned exercisers print identical descriptors in identical order:
// 8080EXM.MAC:132–158; 8085EXM.MAC:133–159. Each tuple fingerprints the
// complete 30-byte descriptor, including the dot padding from the tmsg macro
// (8080EXM.MAC:173–182; 8085EXM.MAC:174–183), excluding result separators.
// Per-entry comments cite the defining tmsg lines in the two sources.
const EXERCISER_DESCRIPTORS: [(usize, u64); 25] = [
    (30, 0x6a8543689655dbcf), // 8080EXM.MAC:190; 8085EXM.MAC:191
    (30, 0x93ec283754c7c2e0), // 8080EXM.MAC:198; 8085EXM.MAC:199
    (30, 0xb394619f31c84908), // 8080EXM.MAC:206; 8085EXM.MAC:207
    (30, 0xec29855e7eec21a1), // 8080EXM.MAC:214; 8085EXM.MAC:215
    (30, 0x199fd94aeb853cb6), // 8080EXM.MAC:222; 8085EXM.MAC:223
    (30, 0xfdda10ae0546a28d), // 8080EXM.MAC:230; 8085EXM.MAC:231
    (30, 0xe7827c245ae0a9b9), // 8080EXM.MAC:238; 8085EXM.MAC:239
    (30, 0x393aeeb423042634), // 8080EXM.MAC:246; 8085EXM.MAC:247
    (30, 0xd5917f5f96a1406b), // 8080EXM.MAC:254; 8085EXM.MAC:255
    (30, 0x0ea709227d4f641f), // 8080EXM.MAC:262; 8085EXM.MAC:263
    (30, 0xc7419bba343fcd92), // 8080EXM.MAC:270; 8085EXM.MAC:271
    (30, 0xbe40016157bb7687), // 8080EXM.MAC:278; 8085EXM.MAC:279
    (30, 0xa7e86cd7ad557db3), // 8080EXM.MAC:286; 8085EXM.MAC:287
    (30, 0x21a2495af2d7d5e3), // 8080EXM.MAC:294; 8085EXM.MAC:295
    (30, 0xe8051807c808a22a), // 8080EXM.MAC:302; 8085EXM.MAC:303
    (30, 0x3902748ecc2a4432), // 8080EXM.MAC:310; 8085EXM.MAC:311
    (30, 0xd104e4186b791f77), // 8080EXM.MAC:318; 8085EXM.MAC:319
    (30, 0x4854936177d0a7f4), // 8080EXM.MAC:326; 8085EXM.MAC:327
    (30, 0xc6a3caddd215602b), // 8080EXM.MAC:334; 8085EXM.MAC:335
    (30, 0x2b77e9389b9af418), // 8080EXM.MAC:342; 8085EXM.MAC:343
    (30, 0x74cc6d2c9339ab09), // 8080EXM.MAC:350; 8085EXM.MAC:351
    (30, 0xd8579847983703e1), // 8080EXM.MAC:358; 8085EXM.MAC:359
    (30, 0x092b8032f1e8144b), // 8080EXM.MAC:366; 8085EXM.MAC:367
    (30, 0x4565ff6bdd8c194d), // 8080EXM.MAC:374; 8085EXM.MAC:375
    (30, 0x1a8d57ffef9a5a83), // 8080EXM.MAC:382; 8085EXM.MAC:383
];

#[derive(Clone, Copy)]
struct OutputGrammar {
    banners: [(usize, u64); 2],
    completion: (usize, u64),
    pass_suffix: (usize, u64),
    error_suffix: (usize, u64),
}

// Fingerprint emitted console text, excluding BDOS's unprinted '$' terminator.
// Banner/completion: 8080EXM.MAC:114–130,954–955;
// 8085EXM.MAC:115–131,955–956. Result suffixes (after PASS/ERROR):
// 8080EXM.MAC:956–958; 8085EXM.MAC:957–959.
const EXERCISER_GRAMMAR: OutputGrammar = OutputGrammar {
    banners: [(26, 0x1e4617a203e22129), (26, 0x8ac3394caca413a6)],
    completion: (14, 0x948df6ea30f0dd7d),
    pass_suffix: (9, 0xfeaaade4a981dfd0),
    error_suffix: (19, 0x7a5d757e1ffba533),
};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Model {
    #[default]
    I8080,
    I8085,
    Z80,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Oracle {
    Preliminary,
    Exerciser,
    Exerciser8085,
    PreliminaryZ80,
    ExerciserZ80,
}

impl Oracle {
    /// Identify only the pinned program bytes, independently of their filename.
    pub fn identify(program: &[u8]) -> Option<Self> {
        Self::from_digest(&sha256::digest(program))
    }

    fn from_digest(digest: &str) -> Option<Self> {
        // Mirror oracles/SHA256SUMS.fetched (8085EXM added at f379261).
        match digest {
            "18eb3c79cba42c0718f160be6a1853cb64cdce7aa47d65780189a57bdd98c4e0" => {
                Some(Self::Preliminary)
            }
            "6e3286e11bb1a8f47b8ee1280b4a067be813193363e3223c99b0d21912f44aeb" => {
                Some(Self::Exerciser)
            }
            "95e430cf2d7793df59c4c1d61a58a6a3ff2260b785067fb35abe5486c57102e1" => {
                Some(Self::Exerciser8085)
            }
            "3b3578f19030a4df7e25ce852f763af26053b12582a576c4dffb014aa7c590d1" => {
                Some(Self::PreliminaryZ80)
            }
            "34923a7ed82285d3038b2d54bd64899e12173eebb61f9d07b4fc72e78af2ae8f" => {
                Some(Self::ExerciserZ80)
            }
            _ => None,
        }
    }

    /// Groups excluded before execution under decision-14, in table order.
    pub fn not_run(self) -> &'static [&'static str] {
        if self == Self::ExerciserZ80 {
            &z80_oracles::NOT_RUN
        } else {
            &[]
        }
    }

    pub fn check_model(self, model: Model) -> io::Result<()> {
        match (self, model) {
            (Self::PreliminaryZ80 | Self::ExerciserZ80, Model::I8080 | Model::I8085) => {
                Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "Z80 oracle requires --model z80",
                ))
            }
            (Self::Preliminary | Self::Exerciser | Self::Exerciser8085, Model::Z80) => {
                Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "--model z80 requires a pinned Z80 oracle",
                ))
            }
            (Self::Exerciser, Model::I8085) => Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "8080EXM requires --model 8080",
            )),
            (Self::Exerciser8085, Model::I8080) => Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "8085EXM requires --model 8085",
            )),
            _ => Ok(()),
        }
    }

    /// The unverified alu8r result is identified by descriptor AND position:
    /// 8085EXM.MAC:136,201–207 (PROVENANCE.md §2).
    pub fn reported_not_judged(self, output: &str) -> Option<&str> {
        let line = output
            .split(['\r', '\n'])
            .filter(|line| group_result(line).is_some())
            .nth(2)?;
        let (descriptor, _, _) = group_result(line)?;
        self.is_reported_group(2, fingerprint(descriptor.as_bytes()))
            .then_some(line)
    }

    fn is_reported_group(self, index: usize, descriptor: (usize, u64)) -> bool {
        self == Self::Exerciser8085 && index == 2 && descriptor == EXERCISER_DESCRIPTORS[2]
    }

    fn validate_output(self, output: &[u8]) -> Result<(), String> {
        match self {
            Self::PreliminaryZ80 => return z80_oracles::validate_preliminary(output),
            Self::ExerciserZ80 => return z80_oracles::validate_exerciser(output),
            _ => {}
        }
        if self == Self::Preliminary {
            // 8080PRE.MAC:216–221: match the complete completion message
            // without reproducing its text.
            return output
                .split(|&b| b == b'\r' || b == b'\n')
                .any(|line| fingerprint(line) == (31, 0xc52616baf4bf534f))
                .then_some(())
                .ok_or_else(|| "missing preliminary completion message".into());
        }
        self.validate_exerciser(output, EXERCISER_GRAMMAR)
    }

    fn validate_exerciser(self, output: &[u8], grammar: OutputGrammar) -> Result<(), String> {
        // Result printing: 8080EXM.MAC:436–481,956–959;
        // 8085EXM.MAC:437–482,957–960. Consume every nonempty line in full.
        let text = std::str::from_utf8(output)
            .map_err(|_| "invalid UTF-8 in exerciser output".to_string())?;
        let mut count = 0;
        let mut banner_seen = false;
        let mut completed = false;
        let mut reported_errors = 0;
        let banner = grammar.banners[usize::from(self == Self::Exerciser8085)];
        for (number, line) in text
            .split(['\r', '\n'])
            .filter(|line| !line.is_empty())
            .enumerate()
        {
            let id = fingerprint(line.as_bytes());
            if id == banner {
                if banner_seen || count != 0 || completed {
                    return Err("unexpected exerciser banner".into());
                }
                banner_seen = true;
                continue;
            }
            if id == grammar.completion {
                if count != EXERCISER_DESCRIPTORS.len() {
                    return Err(format!(
                        "completion with missing exerciser groups: expected 25, found {count}"
                    ));
                }
                if !banner_seen || completed {
                    return Err("unexpected exerciser completion message".into());
                }
                completed = true;
                continue;
            }
            let Some((descriptor, passed, result)) = group_result(line) else {
                if line.contains("ERROR") {
                    return Err("ERROR outside a complete group result".into());
                }
                return Err(format!(
                    "nonempty line {} is unknown or an incomplete group result",
                    number + 1
                ));
            };
            if !banner_seen {
                return Err("missing exerciser banner before group results".into());
            }
            let actual = fingerprint(descriptor.as_bytes());
            let Some(expected) = EXERCISER_DESCRIPTORS.get(count) else {
                return Err("extra exerciser group after group 25".into());
            };
            if actual != *expected {
                let problem = match EXERCISER_DESCRIPTORS.iter().position(|&id| id == actual) {
                    Some(index) if index < count => "duplicated or reordered",
                    Some(_) => "missing or reordered",
                    None => "unknown",
                };
                return Err(format!("group {} descriptor is {problem}", count + 1));
            }
            if !complete_result(result, passed, grammar) {
                return Err(format!(
                    "group {} has invalid result syntax or CRC (including trailing text)",
                    count + 1
                ));
            }
            if !passed && !self.is_reported_group(count, actual) {
                return Err(format!("judged group {} reports ERROR", count + 1));
            }
            reported_errors += usize::from(!passed);
            count += 1;
        }
        // Whole-output backstop: only a complete, exempt result may own ERROR.
        if text.matches("ERROR").count() != reported_errors {
            return Err("ERROR outside a complete reported group result".into());
        }
        if count != EXERCISER_DESCRIPTORS.len() {
            return Err(format!(
                "missing exerciser groups: expected 25, found {count}"
            ));
        }
        if !completed {
            return Err("missing exerciser completion message".into());
        }
        Ok(())
    }
}

fn group_result(line: &str) -> Option<(&str, bool, &str)> {
    line.split_once("  PASS")
        .map(|(group, result)| (group, true, result))
        .or_else(|| {
            line.split_once("  ERROR")
                .map(|(group, result)| (group, false, result))
        })
}

fn complete_result(result: &str, passed: bool, grammar: OutputGrammar) -> bool {
    let prefix = if passed {
        grammar.pass_suffix
    } else {
        grammar.error_suffix
    };
    let Some((label, crcs)) = result.split_at_checked(prefix.0) else {
        return false;
    };
    if fingerprint(label.as_bytes()) != prefix {
        return false;
    }
    // phex8/phex2/phex1 emit exactly eight lowercase hexadecimal digits:
    // 8080EXM.MAC:898–936; 8085EXM.MAC:899–937.
    let crc = |value: &str| {
        value.len() == 8
            && value
                .bytes()
                .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    };
    if passed {
        crc(crcs)
    } else {
        // Separator: 8080EXM.MAC:958; 8085EXM.MAC:959.
        crcs.split_once(" found:")
            .is_some_and(|(expected, found)| crc(expected) && crc(found))
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Config {
    pub model: Model,
    pub cycle_cap: u64,
    pub slice_cycles: u64,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            model: Model::default(),
            cycle_cap: 100_000_000_000,
            slice_cycles: 1_000_000,
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum Reason {
    NormalExit,
    UnexpectedHalt {
        pc: u16,
    },
    Unsupported {
        pc: u16,
        opcode: u8,
        preceding: Vec<u8>,
    },
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
    /// Why a known program's output failed validation, independently of CPU exit.
    pub verdict_error: Option<String>,
    pub output: Vec<u8>,
}

impl Outcome {
    pub fn passed(&self) -> bool {
        self.reason == Reason::NormalExit && self.verdict == Some(true)
    }
}

struct Host<'a, W> {
    ram: Box<[u8; 65536]>,
    model: Model,
    console: &'a mut W,
    output: Vec<u8>,
    arguments: Option<(u8, u16)>,
    awaiting_registers: bool,
    bdos_out: bool,
    z80_instruction_start: bool,
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
        // 8080PRE.MAC:216–221,260–271; 8080EXM.MAC:924–958;
        // prelim.z80:269–274,314–324; zexdoc.z80:1187–1210:
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
        if self.model == Model::I8085 {
            // Decision-18 admits every 8085 opcode, including DSUB at 08h.
            // Supply OUT FFh; RET at 0005h via instruction reads only, so
            // Data reads still see the CP/M stack word at 0006h. The OUT
            // arms a boundary stop; servicing BDOS never changes registers.
            match (address, kind) {
                (5, MemoryAccess::OpcodeFetch) => {
                    self.bdos_out = true;
                    return 0xd3;
                }
                (6, MemoryAccess::OperandFetch) if self.bdos_out => return BDOS_PORT_8085,
                (7, MemoryAccess::OpcodeFetch) => return 0xc9,
                _ => return self.ram[usize::from(address)],
            }
        }
        if self.model == Model::Z80 {
            // L host ABI, decision-20 §1/§7: ED70 now executes. As with
            // the 8085 host, supply OUT FFh; RET only on instruction reads;
            // the Data stack word at 0006h remains intact. Z80 OUT (n),A
            // uses full A:FF as the port and costs one extra step/11 states.
            let instruction_start = kind == MemoryAccess::OpcodeFetch
                && std::mem::take(&mut self.z80_instruction_start);
            match (address, kind) {
                (5, MemoryAccess::OpcodeFetch) => {
                    // A prefixed terminal fetch at 0005h is not the stub's
                    // complete OUT instruction and must never arm BDOS.
                    self.bdos_out = instruction_start;
                    return 0xd3;
                }
                (6, MemoryAccess::OperandFetch) if self.bdos_out => return 0xff,
                (7, MemoryAccess::OpcodeFetch) => return 0xc9,
                _ => return self.ram[usize::from(address)],
            }
        }
        if address == 5 && kind == MemoryAccess::OpcodeFetch {
            if let Some((function, pointer)) = self.arguments.take() {
                self.bdos(function, pointer);
            } else {
                // Bus cannot borrow CPU registers. A private fetch response
                // rendezvous returns through decision-5 item 6: no instruction,
                // cycles or register changes. The runner supplies C/DE and
                // retries this fetch, servicing BDOS before the installed RET.
                // 8080 uses unsupported 08h (decision-3).
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
        if self.model == Model::I8085 && port == BDOS_PORT_8085 && self.bdos_out {
            self.bdos_out = false;
            self.awaiting_registers = true;
            return;
        }
        self.error = Some(format!("unexpected output port {port:02X}"));
    }

    fn input16(&mut self, port: u16) -> u8 {
        self.error = Some(format!("unexpected input port {port:04X}"));
        0
    }

    fn output16(&mut self, port: u16, _value: u8) {
        // Only the harness-owned stub arms this stop; a guest OUT FFh
        // elsewhere remains a host error (decision-20 §7).
        if self.model == Model::Z80 && port & 0xff == 0xff && self.bdos_out {
            self.bdos_out = false;
            self.awaiting_registers = true;
            return;
        }
        self.error = Some(format!("unexpected output port {port:04X}"));
    }
}

impl<W> Control for Host<'_, W> {
    fn stop_requested(&mut self) -> bool {
        if self.model == Model::Z80 {
            // The boundary poll precedes the next instruction's first fetch.
            // Discard unused arming before any later guest I/O can consume it,
            // including when a slice budget deferred this poll to a new run.
            self.bdos_out = false;
            self.z80_instruction_start = true;
        }
        matches!(self.model, Model::I8085 | Model::Z80) && self.awaiting_registers
    }
}
impl<W> i8080::Inputs for Host<'_, W> {}
// decision-12: no interrupts, SID low, SOD notifications ignored.
impl<W> i8085::Inputs for Host<'_, W> {}
// decision-15: NMI and INT are never asserted.
impl<W> z80::Inputs for Host<'_, W> {}

// Local adapters keep the CP/M host and run loop shared. No model abstraction
// is added to the core (decision-6).
trait Cpu: Default {
    fn initialize(&mut self);
    fn pc(&self) -> u16;
    fn halted(&self) -> bool;
    fn bdos_arguments(&self) -> (u8, u16);
    fn run<W: Write>(&mut self, host: &mut Host<'_, W>, budget: Budget) -> x80_core::Report;
}

macro_rules! impl_cpu {
    ($cpu:ty) => {
        impl Cpu for $cpu {
            fn initialize(&mut self) {
                self.registers_mut().pc = LOAD as u16;
                self.registers_mut().sp = STACK;
            }
            fn pc(&self) -> u16 {
                self.registers().pc
            }
            fn halted(&self) -> bool {
                self.latches().halted
            }
            fn bdos_arguments(&self) -> (u8, u16) {
                (self.registers().c, self.registers().de())
            }
            fn run<W: Write>(
                &mut self,
                host: &mut Host<'_, W>,
                budget: Budget,
            ) -> x80_core::Report {
                self.run(host, budget)
            }
        }
    };
}

impl_cpu!(i8080::I8080);
impl_cpu!(i8085::I8085);
impl_cpu!(z80::Z80);

pub fn run<W: Write>(program: &[u8], config: Config, console: &mut W) -> io::Result<Outcome> {
    if program.len() > 65536 - LOAD || program.is_empty() || config.slice_cycles == 0 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "invalid program size or zero slice",
        ));
    }
    let oracle = Oracle::identify(program);
    if let Some(oracle) = oracle {
        oracle.check_model(config.model)?;
    } else if config.model == Model::Z80 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "--model z80 refuses unknown digest; requires pinned prelim.com or zexdoc.com",
        ));
    }
    match config.model {
        Model::I8080 => run_cpu::<i8080::I8080, W>(program, config, console, oracle),
        Model::I8085 => run_cpu::<i8085::I8085, W>(program, config, console, oracle),
        Model::Z80 => run_cpu::<z80::Z80, W>(program, config, console, oracle),
    }
}

fn run_cpu<C: Cpu, W: Write>(
    program: &[u8],
    config: Config,
    console: &mut W,
    oracle: Option<Oracle>,
) -> io::Result<Outcome> {
    let mut host = Host {
        ram: Box::new([0; 65536]),
        model: config.model,
        console,
        output: Vec::new(),
        arguments: None,
        awaiting_registers: false,
        bdos_out: false,
        z80_instruction_start: false,
        error: None,
    };
    host.ram[LOAD..LOAD + program.len()].copy_from_slice(program);
    if oracle == Some(Oracle::ExerciserZ80) {
        z80_oracles::patch_table(&mut host.ram[LOAD..LOAD + program.len()])?;
    }
    // Host stubs implement the CALL 5 / JMP 0 ABI, rather than CPU facts.
    // 8080PRE.MAC:44,52,55,216–219,264–266: initial CALL needs a stack;
    // nonzero HLT is an emergency. 8080EXM.MAC:53,112–116,127–130:
    // load at 0100h, get stack from word 6, and return to the boot entry.
    // Z80 host ABI: prelim.z80:269–274; zexdoc.z80:80–98 (PROVENANCE §2).
    host.ram[0] = 0x76;
    host.ram[5] = 0xc9;
    host.ram[6..8].copy_from_slice(&STACK.to_le_bytes());
    let mut cpu = C::default();
    cpu.initialize();
    let mut steps = 0_u64;
    let mut cycles = 0_u64;
    let reason = loop {
        // HLT can retire exactly at a budget boundary, when Budget wins over
        // Halt. Observe the latch before declaring a timeout (decision-5 §5).
        if cpu.halted() {
            break if cpu.pc() == 1 {
                Reason::NormalExit
            } else {
                Reason::UnexpectedHalt { pc: cpu.pc() }
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
            Exit::Stop
                if matches!(config.model, Model::I8085 | Model::Z80) && host.awaiting_registers =>
            {
                host.awaiting_registers = false;
                let (function, pointer) = cpu.bdos_arguments();
                host.bdos(function, pointer);
                if let Some(error) = host.error.take() {
                    break Reason::HostError(error);
                }
            }
            Exit::Unsupported(unsupported)
                if config.model == Model::I8080
                    && host.awaiting_registers
                    && unsupported.pc == 5
                    && !unsupported.acknowledge
                    && unsupported.opcode == RENDEZVOUS
                    && unsupported.preceding().is_empty() =>
            {
                host.awaiting_registers = false;
                host.arguments = Some(cpu.bdos_arguments());
            }
            Exit::Unsupported(unsupported) => {
                break Reason::Unsupported {
                    pc: unsupported.pc,
                    opcode: unsupported.opcode,
                    preceding: unsupported.preceding().to_vec(),
                }
            }
            other => break Reason::HostError(format!("unexpected core exit: {other:?}")),
        }
    };
    let validation = oracle.map(|oracle| oracle.validate_output(&host.output));
    let verdict = validation
        .as_ref()
        .map(|result| reason == Reason::NormalExit && result.is_ok());
    let verdict_error = validation.and_then(Result::err);
    Ok(Outcome {
        steps,
        cycles,
        reason,
        verdict,
        verdict_error,
        output: host.output,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    impl Oracle {
        fn accepts(self, output: &[u8]) -> bool {
            self.validate_synthetic(output).is_ok()
        }

        fn validate_synthetic(self, output: &[u8]) -> Result<(), String> {
            // Independent synthetic labels exercise the same parser without copying
            // oracle messages. Release campaigns witness the pinned real labels.
            self.validate_exerciser(output, SYNTHETIC_GRAMMAR)
        }
    }

    fn execute(program: &[u8], cycle_cap: u64, slice_cycles: u64) -> Outcome {
        run(
            program,
            Config {
                cycle_cap,
                slice_cycles,
                ..Config::default()
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
                opcode: 8,
                preceding: vec![],
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
        let output = synthetic_output(None);
        assert!(Oracle::Exerciser.accepts(output.as_bytes()));
        assert!(!Oracle::Exerciser.accepts(b""));
        for group in 0..25 {
            assert!(!Oracle::Exerciser.accepts(synthetic_output(Some(group)).as_bytes()));
        }
    }

    #[test]
    fn known_programs_require_the_intended_model() {
        for model in [Model::I8080, Model::I8085] {
            assert!(Oracle::Preliminary.check_model(model).is_ok());
        }
        assert!(Oracle::Exerciser.check_model(Model::I8080).is_ok());
        assert!(Oracle::Exerciser8085.check_model(Model::I8085).is_ok());
        for (oracle, model, expected) in [
            (
                Oracle::Exerciser,
                Model::I8085,
                "8080EXM requires --model 8080",
            ),
            (
                Oracle::Exerciser8085,
                Model::I8080,
                "8085EXM requires --model 8085",
            ),
        ] {
            let error = oracle.check_model(model).unwrap_err();
            assert_eq!(error.kind(), io::ErrorKind::InvalidInput);
            assert_eq!(error.to_string(), expected);
        }
    }

    #[test]
    fn identifies_the_pinned_8085_exerciser_digest() {
        assert_eq!(
            Oracle::from_digest("95e430cf2d7793df59c4c1d61a58a6a3ff2260b785067fb35abe5486c57102e1"),
            Some(Oracle::Exerciser8085)
        );
        assert_eq!(Oracle::identify(b"8085EXM.COM"), None);
    }

    // Minimal descriptor tokens only, confined to tests. The source locations
    // are cited alongside EXERCISER_DESCRIPTORS; tmsg adds dots to 30 bytes.
    const DESCRIPTOR_TOKENS: [&str; 25] = [
        "dad <b,d,h,sp>",
        "aluop nn",
        "aluop <b,c,d,e,h,l,m,a>",
        "<daa,cma,stc,cmc>",
        "<inr,dcr> a",
        "<inr,dcr> b",
        "<inx,dcx> b",
        "<inr,dcr> c",
        "<inr,dcr> d",
        "<inx,dcx> d",
        "<inr,dcr> e",
        "<inr,dcr> h",
        "<inx,dcx> h",
        "<inr,dcr> l",
        "<inr,dcr> m",
        "<inx,dcx> sp",
        "lhld nnnn",
        "shld nnnn",
        "lxi <b,d,h,sp>,nnnn",
        "ldax <b,d>",
        "mvi <b,c,d,e,h,l,m,a>,nn",
        "mov <bcdehla>,<bcdehla>",
        "sta nnnn / lda nnnn",
        "<rlc,rrc,ral,rar>",
        "stax <b,d>",
    ];

    fn synthetic_lines(error_group: Option<usize>) -> Vec<String> {
        DESCRIPTOR_TOKENS
            .iter()
            .enumerate()
            .map(|(index, token)| {
                let descriptor = format!("{token:.<30}");
                // Independent test text must match every pinned production fingerprint.
                assert_eq!(
                    fingerprint(descriptor.as_bytes()),
                    EXERCISER_DESCRIPTORS[index]
                );
                let result = if error_group == Some(index) {
                    "ERROR expected:1234abcd found:5678abcd"
                } else {
                    "PASS checksum:1234abcd"
                };
                format!("{descriptor}  {result}")
            })
            .collect()
    }

    const SYNTHETIC_GRAMMAR: OutputGrammar = OutputGrammar {
        banners: [(16, 0x6c6fbed30e96e62d); 2],
        completion: (20, 0xc13cc3415a6f6836),
        pass_suffix: (10, 0x831c2f70febef882),
        error_suffix: (10, 0xf84a2fb15b4d8b7f),
    };

    fn synthetic_output(error_group: Option<usize>) -> String {
        synthetic_transcript(&synthetic_lines(error_group))
    }

    fn synthetic_transcript(lines: &[String]) -> String {
        format!(
            "synthetic header\n\r{}\n\rsynthetic completion",
            lines.join("\n\r")
        )
    }

    #[test]
    fn only_the_source_identified_third_group_is_reported_not_judged() {
        let output = synthetic_output(Some(2));
        assert_eq!(
            Oracle::Exerciser8085.reported_not_judged(&output),
            Some(synthetic_lines(Some(2))[2].as_str())
        );
        assert_eq!(Oracle::Exerciser.reported_not_judged(&output), None);
        assert_eq!(Oracle::Preliminary.reported_not_judged(&output), None);
    }

    #[test]
    fn exerciser_8085_judges_24_groups_and_requires_all_25_results() {
        let oracle = Oracle::Exerciser8085;
        assert!(oracle.accepts(synthetic_output(None).as_bytes()));
        for group in 0..25 {
            assert_eq!(
                oracle.accepts(synthetic_output(Some(group)).as_bytes()),
                group == 2
            );
        }
        assert!(!oracle.accepts(b""));
        assert!(!oracle.accepts(format!("{}\nERROR", synthetic_output(None)).as_bytes()));
    }

    #[test]
    fn swapped_groups_cannot_exempt_a_judged_error() {
        let mut lines = synthetic_lines(Some(3));
        lines.swap(2, 3);
        let output = synthetic_transcript(&lines);
        for oracle in [Oracle::Exerciser, Oracle::Exerciser8085] {
            assert_eq!(
                oracle.validate_synthetic(output.as_bytes()).unwrap_err(),
                "group 3 descriptor is missing or reordered"
            );
            assert_eq!(oracle.reported_not_judged(&output), None);
        }
    }

    #[test]
    fn substituted_unknown_group_is_rejected() {
        let mut lines = synthetic_lines(None);
        lines[24] = "unknown group  PASS checksum:1234abcd".into();
        for oracle in [Oracle::Exerciser, Oracle::Exerciser8085] {
            assert_eq!(
                oracle
                    .validate_synthetic(synthetic_transcript(&lines).as_bytes())
                    .unwrap_err(),
                "group 25 descriptor is unknown"
            );
        }
    }

    #[test]
    fn duplicated_group_is_rejected() {
        let mut lines = synthetic_lines(None);
        lines[24] = lines[23].clone();
        for oracle in [Oracle::Exerciser, Oracle::Exerciser8085] {
            assert_eq!(
                oracle
                    .validate_synthetic(synthetic_transcript(&lines).as_bytes())
                    .unwrap_err(),
                "group 25 descriptor is duplicated or reordered"
            );
        }
    }

    #[test]
    fn missing_group_is_rejected() {
        for missing in 0..25 {
            let mut lines = synthetic_lines(None);
            lines.remove(missing);
            for oracle in [Oracle::Exerciser, Oracle::Exerciser8085] {
                let error = oracle
                    .validate_synthetic(synthetic_transcript(&lines).as_bytes())
                    .unwrap_err();
                assert!(error.contains("missing"), "{error}");
            }
        }
    }

    #[test]
    fn extra_group_is_rejected() {
        let mut lines = synthetic_lines(None);
        lines.push(lines[0].clone());
        for oracle in [Oracle::Exerciser, Oracle::Exerciser8085] {
            assert_eq!(
                oracle
                    .validate_synthetic(synthetic_transcript(&lines).as_bytes())
                    .unwrap_err(),
                "extra exerciser group after group 25"
            );
        }
    }

    #[test]
    fn alu8r_only_error_passes_and_is_reported() {
        let output = synthetic_output(Some(2));
        assert!(Oracle::Exerciser8085.accepts(output.as_bytes()));
        assert!(Oracle::Exerciser8085
            .reported_not_judged(&output)
            .unwrap()
            .contains("ERROR"));
    }

    #[test]
    fn incomplete_pass_and_both_error_crcs_are_rejected() {
        for output in [
            synthetic_output(None).replace("1234abcd", "1234abc"),
            synthetic_output(Some(2)).replace("expected:1234abcd", "expected:1234abc"),
            synthetic_output(Some(2)).replace("5678abcd", "5678abc"),
            synthetic_output(None).replace("1234abcd", "1234abcd trailing"),
        ] {
            for oracle in [Oracle::Exerciser, Oracle::Exerciser8085] {
                assert!(oracle
                    .validate_synthetic(output.as_bytes())
                    .unwrap_err()
                    .contains("CRC"));
            }
        }
    }

    #[test]
    fn error_crc_cannot_swallow_another_descriptor_and_error() {
        let descriptor = format!("{:.<30}", DESCRIPTOR_TOKENS[3]);
        let output = synthetic_output(Some(2)).replace(
            "expected:1234abcd found:",
            &format!("expected:1234abcd {descriptor}  ERROR found:"),
        );
        assert!(!Oracle::Exerciser8085.accepts(output.as_bytes()));
    }

    #[test]
    fn trailing_descriptor_with_err_fragment_is_rejected() {
        let output = format!(
            "{}\n{:.<30}  ERR",
            synthetic_output(None),
            DESCRIPTOR_TOKENS[3]
        );
        assert!(!Oracle::Exerciser8085.accepts(output.as_bytes()));
    }

    #[test]
    fn trailing_descriptor_only_is_rejected() {
        let output = format!("{}\n{:.<30}", synthetic_output(None), DESCRIPTOR_TOKENS[3]);
        assert!(!Oracle::Exerciser8085.accepts(output.as_bytes()));
    }

    #[test]
    fn trailing_err_fragment_is_rejected() {
        let output = format!("{}\nERR", synthetic_output(None));
        assert!(!Oracle::Exerciser8085.accepts(output.as_bytes()));
    }

    #[test]
    fn split_error_marker_is_rejected() {
        let output = format!("{}\nER\nROR", synthetic_output(None));
        assert!(!Oracle::Exerciser8085.accepts(output.as_bytes()));
    }

    #[test]
    fn trailing_garbage_after_pass_crc_is_rejected() {
        let output = synthetic_output(None).replacen("1234abcd", "1234abcd garbage", 1);
        assert!(!Oracle::Exerciser8085.accepts(output.as_bytes()));
    }

    #[test]
    fn framing_must_be_present_once_and_in_order() {
        let output = synthetic_output(None);
        for invalid in [
            output.replace("synthetic header", ""),
            output.replace("synthetic completion", ""),
            format!("synthetic header\n{output}"),
            format!("{output}\nsynthetic completion"),
            output.replace("synthetic header", "synthetic completion"),
            output.replace("synthetic completion", "synthetic header"),
            output.replace("synthetic header", "synthetic header garbage"),
            output.replace("synthetic completion", "synthetic completion garbage"),
        ] {
            for oracle in [Oracle::Exerciser, Oracle::Exerciser8085] {
                assert!(oracle.validate_synthetic(invalid.as_bytes()).is_err());
            }
        }
    }

    #[test]
    fn only_exact_result_syntax_and_lowercase_crcs_are_accepted() {
        let output = synthetic_output(Some(2));
        for invalid in [
            output.replace("  PASS", " PASS"),
            output.replace("  ERROR", " ERROR"),
            output.replace("checksum:", "checksum :"),
            output.replace("expected:", "expected :"),
            output.replace(" found:", "  found:"),
            output.replace("found:", "found :"),
            output.replace("5678abcd", "5678abcd garbage"),
            output.replace("1234abcd", "1234abcD"),
            output.replace("5678abcd", "5678abcg"),
        ] {
            assert!(Oracle::Exerciser8085
                .validate_synthetic(invalid.as_bytes())
                .is_err());
        }
        assert!(Oracle::Exerciser8085.accepts(output.as_bytes()));
    }

    #[test]
    fn unknown_lines_and_extra_error_markers_are_rejected_everywhere() {
        let lines = synthetic_lines(Some(2));
        for index in 0..=lines.len() {
            for fragment in ["ER", "ROR", "ERROR", "ERR", " ", "garbage"] {
                let mut invalid = lines.clone();
                invalid.insert(index, fragment.into());
                assert!(Oracle::Exerciser8085
                    .validate_synthetic(synthetic_transcript(&invalid).as_bytes())
                    .is_err());
            }
        }
        for marker in ["ERROR", "PASS"] {
            let invalid =
                synthetic_output(Some(2)).replace("5678abcd", &format!("5678abcd {marker}"));
            assert!(Oracle::Exerciser8085
                .validate_synthetic(invalid.as_bytes())
                .unwrap_err()
                .contains("invalid result syntax or CRC"));
        }
    }

    #[test]
    fn non_utf8_output_is_rejected() {
        let mut output = synthetic_output(None).into_bytes();
        output.push(0xff);
        assert_eq!(
            Oracle::Exerciser8085
                .validate_synthetic(&output)
                .unwrap_err(),
            "invalid UTF-8 in exerciser output"
        );
    }

    #[test]
    fn i8085_out_rendezvous_preserves_console_and_boot_exit() {
        let program = [
            0x11, 0x12, 0x01, // LXI D,0112h
            0x0e, 9, 0xcd, 5, 0, // MVI C,9; CALL 5
            0x1e, b'!', 0x0e, 2, 0xcd, 5, 0, // MVI E,'!'; MVI C,2; CALL 5
            0xc3, 0, 0, // JMP 0
            b'H', b'i', b'$',
        ];
        let result = run(
            &program,
            Config {
                model: Model::I8085,
                slice_cycles: 17,
                ..Config::default()
            },
            &mut Vec::new(),
        )
        .unwrap();
        assert_eq!(result.output, b"Hi!");
        assert_eq!(result.reason, Reason::NormalExit);
        // Decision-18: one real OUT (10 states) per BDOS call before RET.
        assert_eq!((result.steps, result.cycles), (12, 122));
        assert_eq!(result.verdict, None);
        assert!(!result.passed());
        let dsub = run(
            &[0x08, 0x76],
            Config {
                model: Model::I8085,
                ..Config::default()
            },
            &mut Vec::new(),
        )
        .unwrap();
        // Decision-18: DSUB executes normally away from BDOS too.
        assert_eq!(dsub.reason, Reason::UnexpectedHalt { pc: 0x102 });
        assert_eq!((dsub.steps, dsub.cycles), (2, 15));
    }

    #[test]
    fn invalid_program_or_slice_is_rejected() {
        for (program, slice) in [(&[][..], 1), (&[0x76][..], 0), (&[0; 65536][..], 1)] {
            assert!(run(
                program,
                Config {
                    cycle_cap: 100,
                    slice_cycles: slice,
                    ..Config::default()
                },
                &mut Vec::new()
            )
            .is_err());
        }
    }

    #[test]
    fn z80_digest_pins_and_model_pairings_are_exclusive() {
        for (digest, oracle) in [
            (
                "3b3578f19030a4df7e25ce852f763af26053b12582a576c4dffb014aa7c590d1",
                Oracle::PreliminaryZ80,
            ),
            (
                "34923a7ed82285d3038b2d54bd64899e12173eebb61f9d07b4fc72e78af2ae8f",
                Oracle::ExerciserZ80,
            ),
        ] {
            assert_eq!(Oracle::from_digest(digest), Some(oracle));
            assert!(oracle.check_model(Model::Z80).is_ok());
            for model in [Model::I8080, Model::I8085] {
                assert!(oracle.check_model(model).is_err());
            }
        }
        for oracle in [
            Oracle::Preliminary,
            Oracle::Exerciser,
            Oracle::Exerciser8085,
        ] {
            assert!(oracle.check_model(Model::Z80).is_err());
            assert!(oracle.not_run().is_empty());
        }
        assert_eq!(Oracle::identify(b"prelim.com"), None);
        assert_eq!(
            Oracle::ExerciserZ80.not_run(),
            [
                "alu8rx", "incxh", "incxl", "incyh", "incyl", "ld8ixy", "ld8rrx", "rotxy",
                "rotz80",
            ]
        );
        let mut console = Vec::new();
        assert!(run(
            &[0x76],
            Config {
                model: Model::Z80,
                ..Config::default()
            },
            &mut console
        )
        .is_err());
        assert!(console.is_empty());
    }

    fn z80_synthetic(program: &[u8]) -> Outcome {
        // Deliberately bypass digest selection only inside these host unit tests.
        run_cpu::<z80::Z80, _>(
            program,
            Config {
                model: Model::Z80,
                cycle_cap: 1000,
                slice_cycles: 1,
            },
            &mut Vec::new(),
            None,
        )
        .unwrap()
    }

    #[test]
    fn z80_bdos_uses_out_ret_and_preserves_documented_exchange_and_stack_word() {
        // Read the host stack word as Data, then exercise documented 08h on
        // both sides of output. A synthetic 08h trap would corrupt this run.
        let program = [
            0x2a, 6, 0, 0xf9, // LD HL,(6); LD SP,HL
            0x3e, b'A', 0x08, 0x3e, b'B', 0x08, // A='A', A'='B'
            0x5f, 0x0e, 2, 0xcd, 5, 0, // LD E,A; LD C,2; CALL 5
            0x08, 0x5f, 0x0e, 2, 0xcd, 5, 0, 0x11, 0x22, 1, 0x0e, 9, 0xcd, 5, 0, 0xc3, 0, 0, b'!',
            b'$',
        ];
        let outcome = z80_synthetic(&program);
        assert_eq!(outcome.reason, Reason::NormalExit);
        assert_eq!(outcome.output, b"AB!");
        assert_eq!(outcome.verdict, None);
    }

    #[test]
    fn z80_out_rendezvous_adds_one_step_eleven_states_per_call() {
        // Decision-20 §1/§7 host ABI. The old refusal/RET path took
        // 10 steps/99 states for this two-call fixture; OUT adds 2/22.
        let program = [
            0x11, 0x12, 0x01, // LD DE,0112h
            0x0e, 9, 0xcd, 5, 0, 0x1e, b'!', 0x0e, 2, 0xcd, 5, 0, 0xc3, 0, 0, b'H', b'i', b'$',
        ];
        for slice_cycles in [1, 11, 17, 1_000_000] {
            let result = run_cpu::<z80::Z80, _>(
                &program,
                Config {
                    model: Model::Z80,
                    slice_cycles,
                    ..Config::default()
                },
                &mut Vec::new(),
                None,
            )
            .unwrap();
            assert_eq!(result.reason, Reason::NormalExit);
            assert_eq!(result.output, b"Hi!");
            assert_eq!((result.steps, result.cycles), (12, 121));
        }
    }

    #[test]
    fn z80_stub_out_at_overall_cap_times_out_before_bdos() {
        // Decision-20 §7: reuse the 8085 boundary-stop convention; a slice
        // boundary may defer service, but the overall cap must win.
        let program = [0x0e, 2, 0x1e, b'!', 0x3e, 0xab, 0xcd, 5, 0, 0xc3, 0, 0];
        for slice_cycles in [1, 1_000_000] {
            let result = run_cpu::<z80::Z80, _>(
                &program,
                Config {
                    model: Model::Z80,
                    slice_cycles,
                    cycle_cap: 49,
                },
                &mut Vec::new(),
                None,
            )
            .unwrap();
            assert_eq!(result.reason, Reason::Timeout);
            assert!(result.output.is_empty());
            assert_eq!((result.steps, result.cycles), (5, 49));
        }
    }

    #[test]
    fn z80_ed70_ed71_and_reserved_port_outside_stub_execute_and_report_io_errors() {
        // Decision-20 §1: ED70/71 are executable; only a stub-owned OUT
        // may stop for BDOS. ED71's emitted zero is an L expectation.
        for (tail, message) in [
            (&[0xed, 0x70][..], "unexpected input port ABCD"),
            (&[0xed, 0x71][..], "unexpected output port ABCD"),
            (&[0x3e, 0xab, 0xd3, 0xff][..], "unexpected output port ABFF"),
        ] {
            let mut program = vec![0x01, 0xcd, 0xab];
            program.extend_from_slice(tail);
            assert_eq!(
                z80_synthetic(&program).reason,
                Reason::HostError(message.into())
            );
        }
    }

    #[test]
    fn z80_ed_prefix_before_stub_does_not_arm_bdos_for_later_guest_out() {
        let program = [
            0x3e, 0xed, 0x32, 4, 0, // LD A,EDh; LD (0004h),A
            0x3e, 0xab, 0x0e, 2, 0x1e, b'!', // LD A,ABh; LD C,2; LD E,'!'
            0xcd, 4, 0, // CALL 0004h: ED D3; NOP at 0006h; synthetic RET
            0xd3, 0xff, 0xc3, 0, 0, // guest OUT (FFh),A; JP 0
        ];
        for slice_cycles in [1, 11, 17, 1_000_000] {
            let result = run_cpu::<z80::Z80, _>(
                &program,
                Config {
                    model: Model::Z80,
                    slice_cycles,
                    ..Config::default()
                },
                &mut Vec::new(),
                None,
            )
            .unwrap();
            assert_eq!(
                result.reason,
                Reason::HostError("unexpected output port ABFF".into())
            );
            assert!(result.output.is_empty());
        }
    }

    // Decision-20 §3: remove CB30 and indexed-BIT refusal fixtures.
    // This CP/M host never asserts INT/NMI, so no CPU refusal path remains
    // to fixture. Core interrupt tests retain payload/rollback witnesses.

    #[test]
    fn z80_port_errors_keep_the_full_address() {
        // Decision-15: use Bus's 16-bit callbacks, never truncate diagnostics.
        for (opcode, message) in [
            (0xdb, "unexpected input port AB12"),
            (0xd3, "unexpected output port AB12"),
        ] {
            let outcome = z80_synthetic(&[0x3e, 0xab, opcode, 0x12]);
            assert_eq!(outcome.reason, Reason::HostError(message.into()));
        }
    }
}
