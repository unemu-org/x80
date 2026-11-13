//! Spectrum-shaped harness ABI, not Spectrum emulation or CPU semantics.
//! decision-20 §7; v1.2a main.asm:7–14,68–72,102–117;
//! print.asm:8–9,67–80; idea.asm:46–47,87–104,264–289.

use std::time::{Duration, Instant};

use x80_core::{z80, Budget, Bus, Control, Exit, MemoryAccess};

use super::{Config, Result, Variant};

const STACK: u16 = 0x7ff0;
const RETURN: u16 = 0x7ffe;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Service {
    Character,
    Channel,
}

#[derive(Clone, Copy)]
pub(super) struct Sites {
    init: u16,
    character: u16,
}

impl Sites {
    pub(super) fn audit(code: &[u8], variant: Variant) -> Result<Self> {
        // Pinned main.asm:14 and print.asm:8–9,67–80. Variant banners
        // relocate the print module; these are measured TAP addresses.
        let init: u16 = match variant {
            Variant::Full => 0x815b,
            Variant::Doc | Variant::Ccf => 0x815a,
            Variant::Flags => 0x815c,
            Variant::DocFlags => 0x8160,
            Variant::Memptr => 0x815d,
        };
        let offset = usize::from(init - 0x8000);
        let [lo, hi] = init.to_le_bytes();
        if code.get(5..8) != Some(&[0xcd, lo, hi])
            || code.get(offset..offset + 5) != Some(&[0x3e, 2, 0xc3, 1, 0x16])
            || code.get(offset + 0x47..offset + 0x59)
                != Some(&[
                    0xfd, 0xe5, 0xfd, 0x21, 0x3a, 0x5c, 0xd5, 0xc5, 0xd9, 0xfb, 0xd7, 0xf3, 0xd9,
                    0xc1, 0xd1, 0xfd, 0xe1, 0xc9,
                ])
        {
            return Err("TAP ROM call sites do not correspond to main/print".into());
        }
        Ok(Self {
            init,
            character: init + 0x51,
        })
    }
}

enum Transfer {
    Init,
    Channel { sp: u16 },
    Character,
}

struct Host {
    ram: Box<[u8; 65536]>,
    service: Option<Service>,
    armed: Option<Service>,
    sites: Sites,
    instruction_start: bool,
    transfer: Option<Transfer>,
    channel_path: Option<(u16, u16)>, // next instruction, saved SP
    caller_read: bool,
    error: Option<String>,
    output: Vec<u8>,
    fe: Option<u8>,
    reads: u64,
    writes: u64,
    characters: u64,
    channels: u64,
}

impl Host {
    fn new(code: &[u8], sites: Sites) -> Self {
        let mut ram = Box::new([0; 65536]);
        ram[0x8000..0x8000 + code.len()].copy_from_slice(code);
        ram[usize::from(STACK)..usize::from(STACK) + 2].copy_from_slice(&RETURN.to_le_bytes());
        // ERR-NR address carried in IY by print.asm. No source in main/print
        // reads system-variable memory; our ROM responses do not read it.
        // FF is a harness initial value, not a claim about a ROM revision.
        ram[0x5c3a] = 255;
        Self {
            ram,
            service: None,
            armed: None,
            sites,
            instruction_start: false,
            transfer: None,
            channel_path: None,
            caller_read: false,
            error: None,
            output: Vec::new(),
            fe: None,
            reads: 0,
            writes: 0,
            characters: 0,
            channels: 0,
        }
    }

    fn return_word(&self, sp: u16) -> u16 {
        u16::from_le_bytes([
            self.ram[usize::from(sp)],
            self.ram[usize::from(sp.wrapping_add(1))],
        ])
    }

    fn validate_transfer(&mut self, cpu: &z80::Z80, transfer: Transfer) -> Result<()> {
        let regs = cpu.registers();
        let (target, return_pc, service) = match transfer {
            Transfer::Init => (self.sites.init, 0x8008, None),
            Transfer::Channel { sp } => {
                if regs.sp != sp {
                    return Err("CHAN-OPEN stack changed after audited CALL".into());
                }
                (0x1601, 0x8008, Some(Service::Channel))
            }
            Transfer::Character => (0x10, self.sites.character + 1, Some(Service::Character)),
        };
        if regs.pc != target || self.return_word(regs.sp) != return_pc {
            return Err("ROM service requires audited call and expected stack return".into());
        }
        if service.is_none() {
            self.channel_path = Some((self.sites.init, regs.sp));
        }
        self.armed = service;
        Ok(())
    }
}

impl Bus for Host {
    fn read(&mut self, address: u16, kind: MemoryAccess) -> u8 {
        if kind == MemoryAccess::OpcodeFetch {
            let start = std::mem::take(&mut self.instruction_start);
            let service = match address {
                0x10 => Some(Service::Character),
                0x1601 => Some(Service::Channel),
                _ => None,
            };
            if let Some(service) = service {
                if start && self.armed.take() == Some(service) {
                    self.service = Some(service);
                    return 0xc9; // RET, one step / 10 cycles
                }
                self.error = Some(format!(
                    "unaudited ROM entry {address:04X} (instruction_start={start})"
                ));
                return 0; // No RET or output on an invalid entry; stop next boundary.
            }
            self.armed = None;
            if start {
                let opcode = self.ram[usize::from(address)];
                if let Some((next, sp)) = self.channel_path.take() {
                    if address == next
                        && address == self.sites.init
                        && opcode == 0x3e
                        && self.ram[usize::from(address + 1)] == 2
                    {
                        self.channel_path = Some((address + 2, sp));
                    } else if address == next
                        && address == self.sites.init + 2
                        && self.ram[usize::from(address)..usize::from(address) + 3]
                            == [0xc3, 1, 0x16]
                    {
                        self.transfer = Some(Transfer::Channel { sp });
                    }
                }
                if address == 0x8005
                    && self.ram[0x8005..0x8008]
                        == [0xcd, self.sites.init as u8, (self.sites.init >> 8) as u8]
                {
                    self.transfer = Some(Transfer::Init);
                } else if address == self.sites.character && opcode == 0xd7 {
                    self.transfer = Some(Transfer::Character);
                }
            }
        }
        if kind == MemoryAccess::Stack && address == STACK + 1 {
            self.caller_read = true;
        }
        self.ram[usize::from(address)]
    }
    fn write(&mut self, address: u16, value: u8, _kind: MemoryAccess) {
        self.ram[usize::from(address)] = value;
    }
    fn input(&mut self, port: u8) -> u8 {
        self.input16(u16::from(port))
    }
    fn output(&mut self, port: u8, value: u8) {
        self.output16(u16::from(port), value);
    }
    fn input16(&mut self, port: u16) -> u8 {
        self.reads += 1;
        if port as u8 != 0xfe || self.fe != Some(7) {
            self.error = Some(format!(
                "unexpected input {port:04X}, FE latch={:?}; requires suite's OUT FE,07",
                self.fe
            ));
        }
        // Deterministic suite environment: no keys/tape, MIC low. The high
        // address byte is accepted for every input vector, not just precheck.
        0xbf
    }
    fn output16(&mut self, port: u16, value: u8) {
        self.writes += 1;
        if port as u8 == 0xfe {
            self.fe = Some(value);
        } else {
            self.error = Some(format!("unexpected output {port:04X}={value:02X}"));
        }
    }
}
impl Control for Host {
    fn stop_requested(&mut self) -> bool {
        // The core polls before an instruction's first fetch, including after
        // a slice. Later prefix/terminal fetches cannot acquire this marker.
        self.instruction_start = true;
        self.transfer.is_some()
            || self.service.is_some()
            || self.caller_read
            || self.error.is_some()
    }
}
// No 50 Hz interrupt is needed: no waits/HALT in the driver; generated HALT
// sequences are skipped (idea.asm:87–104); tests.asm:229–230 omits RST/HALT.
// EI around ROM printing/exit is supported without scheduling INT or NMI.
impl z80::Inputs for Host {}

#[derive(Debug, PartialEq, Eq)]
pub(super) enum Reason {
    Returned,
    Unsupported {
        step: u64,
        pc: u16,
        encoding: Vec<u8>,
    },
    StepBudget,
    TimeBudget,
    Halt {
        pc: u16,
    },
    HostError(String),
}

impl std::fmt::Display for Reason {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Returned => write!(f, "EI/RET-to-caller"),
            Self::Unsupported { step, pc, encoding } => {
                write!(f, "Unsupported step={step} PC={pc:04X} encoding=[")?;
                for (i, byte) in encoding.iter().enumerate() {
                    if i > 0 {
                        write!(f, " ")?;
                    }
                    write!(f, "{byte:02X}")?;
                }
                write!(f, "]")
            }
            Self::StepBudget => write!(f, "step-budget"),
            Self::TimeBudget => write!(f, "time-budget"),
            Self::Halt { pc } => write!(f, "unexpected-HALT PC={pc:04X}"),
            Self::HostError(e) => write!(f, "host-error: {e}"),
        }
    }
}

pub(super) struct Run {
    pub(super) steps: u64,
    pub(super) cycles: u64,
    pub(super) elapsed: Duration,
    pub(super) reason: Reason,
    pub(super) output: Vec<u8>,
    pub(super) characters: u64,
    pub(super) channels: u64,
    pub(super) reads: u64,
    pub(super) writes: u64,
    #[cfg(test)]
    cpu: z80::Z80,
}

pub(super) fn run(code: &[u8], sites: Sites, config: Config) -> Result<Run> {
    if code.is_empty() || code.len() > 32768 || config.step_cap == 0 || config.time_cap.is_zero() {
        return Err("invalid z80test image or zero budget".into());
    }
    let mut host = Host::new(code, sites);
    let mut cpu = z80::Z80::default();
    let regs = cpu.registers_mut();
    regs.pc = 0x8000;
    regs.sp = STACK;
    regs.iy = 0x5c3a;
    let mut steps = 0_u64;
    let mut cycles = 0_u64;
    let start = Instant::now();
    let reason = loop {
        if cpu.latches().halted {
            break Reason::Halt {
                pc: cpu.registers().pc,
            };
        }
        if steps >= config.step_cap {
            break Reason::StepBudget;
        }
        if start.elapsed() >= config.time_cap {
            break Reason::TimeBudget;
        }
        let report = cpu.run(
            &mut host,
            Budget {
                steps: (config.step_cap - steps).min(100_000),
                cycles: u64::MAX,
            },
        );
        steps += report.steps;
        cycles += report.cycles;
        if let Some(e) = host.error.take() {
            break Reason::HostError(e);
        }
        if let Some(transfer) = host.transfer.take() {
            // Stop after audited transfers so CPU PC/SP and the actual stack
            // word can be checked BEFORE the service's RET is supplied. The
            // channel path also requires CALL printinit; LD A,2; JP 1601.
            if let Err(e) = host.validate_transfer(&cpu, transfer) {
                break Reason::HostError(e);
            }
        }
        // These flags are serviced even when the exact boundary is also a
        // slice/cap boundary. The synthetic RET has already retired in full.
        if let Some(service) = host.service.take() {
            match service {
                Service::Character => {
                    host.characters += 1;
                    if host.output.len() >= 2 * 1024 * 1024 {
                        break Reason::HostError("console limit".into());
                    }
                    host.output.push(cpu.registers().a);
                }
                Service::Channel => {
                    host.channels += 1;
                    if cpu.registers().a != 2 {
                        break Reason::HostError("CHAN-OPEN requires A=2".into());
                    }
                }
            }
        }
        if host.caller_read {
            host.caller_read = false;
            if cpu.registers().pc == RETURN && cpu.registers().sp == STACK + 2 {
                break if cpu.latches().iff1 {
                    Reason::Returned
                } else {
                    Reason::HostError("caller return without EI".into())
                };
            }
        }
        match report.exit {
            Exit::Stop | Exit::Budget | Exit::Halt => {}
            Exit::Unsupported(u) => {
                let mut encoding = u.preceding().to_vec();
                encoding.push(u.opcode);
                break Reason::Unsupported {
                    step: steps + 1,
                    pc: u.pc,
                    encoding,
                };
            }
            e => break Reason::HostError(format!("unexpected core exit {e:?}")),
        }
    };
    Ok(Run {
        steps,
        cycles,
        elapsed: start.elapsed(),
        reason,
        output: host.output,
        characters: host.characters,
        channels: host.channels,
        reads: host.reads,
        writes: host.writes,
        #[cfg(test)]
        cpu,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    const SITES: Sites = Sites {
        init: 0x8020,
        character: 0x8010,
    };

    fn run(code: &[u8], config: Config) -> Result<Run> {
        super::run(code, SITES, config)
    }
    fn config(steps: u64) -> Config {
        Config {
            step_cap: steps,
            time_cap: Duration::from_secs(5),
        }
    }
    #[test]
    fn tiny_program_ports_rom_returns_and_exact_cost() {
        let mut code = vec![
            0x3e, 7, 0xd3, 0xfe, // LD A,7; OUT (FE),A
            0, 0xcd, 0x20, 0x80, // NOP; audited CALL printinit
            0xaf, 0xdb, 0xfe, // XOR A; IN A,(FE)
            0x32, 0, 0x60, // LD (6000),A
            0x3e, b'!', 0xd7, // LD A,'!'; RST 10
            0xfb, 0xc9,
        ]; // EI; RET (no caller HALT)
        code.resize(0x20, 0);
        code.extend_from_slice(&[0x3e, 2, 0xc3, 1, 0x16]); // printinit tail call
        let r = run(&code, config(100)).unwrap();
        assert_eq!(r.reason, Reason::Returned);
        assert_eq!(r.output, b"!");
        assert_eq!((r.channels, r.characters, r.reads, r.writes), (1, 1, 1, 1));
        // 7+11+4+17+7+10+10+4+11+13+7+11+10+4+10.
        assert_eq!((r.steps, r.cycles), (15, 136));
        assert_eq!(r.cpu.registers().iy, 0x5c3a);
        assert_eq!(r.cpu.registers().sp, STACK + 2);
        // Only the two actual RET responses add 2 steps and 20 cycles.
        assert_eq!(run(&code, config(14)).unwrap().reason, Reason::StepBudget);
        assert_eq!(run(&code, config(15)).unwrap().reason, Reason::Returned);
    }
    #[test]
    fn refusal_halt_bad_ports_and_bad_channel_never_return_success() {
        let r = run(&[0xdd, 0xdd], config(100)).unwrap();
        assert_eq!(
            r.reason,
            Reason::Unsupported {
                step: 1,
                pc: 0x8000,
                encoding: vec![0xdd, 0xdd],
            }
        );
        assert_eq!((r.steps, r.cycles), (0, 0));
        assert!(matches!(
            run(&[0x76], config(1)).unwrap().reason,
            Reason::Halt { .. }
        ));
        for code in [&[0xd3, 0xff][..], &[0xdb, 0xfe], &[0x3e, 1, 0xcd, 1, 0x16]] {
            assert!(matches!(
                run(code, config(100)).unwrap().reason,
                Reason::HostError(_)
            ));
        }
    }

    #[test]
    fn unaudited_rom_entries_never_supply_ret_or_capture_output() {
        for (code, expected_sp, instruction_start) in [
            (&[0xc3, 0x10, 0][..], STACK, true),          // stray JP 0010
            (&[0x3e, 2, 0xcd, 1, 0x16], STACK - 2, true), // unaudited CALL
            // Place DD at 000F, then JP there: terminal fetch at 0010.
            (&[0x3e, 0xdd, 0x32, 0x0f, 0, 0xc3, 0x0f, 0], STACK, false),
            // Even RST 10 from another site cannot arm the service.
            (&[0xd7], STACK - 2, true),
        ] {
            let r = run(code, config(100)).unwrap();
            assert!(
                matches!(r.reason, Reason::HostError(ref e) if e.contains("unaudited ROM entry")
                    && e.contains(&format!("instruction_start={instruction_start}"))),
                "{:?}",
                r.reason
            );
            assert_eq!((r.characters, r.channels), (0, 0));
            assert!(r.output.is_empty());
            // Invalid entries see NOP, never a synthetic stack pop.
            assert_eq!(r.cpu.registers().sp, expected_sp);
        }
    }

    #[test]
    fn forged_stack_and_jump_to_printinit_cannot_authorize_channel() {
        let mut code = vec![0x21, 8, 0x80, 0xe5, 0xc3, 0x20, 0x80];
        code.resize(0x20, 0);
        code.extend_from_slice(&[0x3e, 2, 0xc3, 1, 0x16]);
        let r = run(&code, config(100)).unwrap();
        assert!(matches!(r.reason, Reason::HostError(_)));
        assert_eq!((r.channels, r.characters), (0, 0));
    }

    #[test]
    fn service_authorization_requires_the_expected_stack_word_and_pointer() {
        let mut host = Host::new(&[], SITES);
        let mut cpu = z80::Z80::default();
        cpu.registers_mut().pc = 0x10;
        cpu.registers_mut().sp = STACK;
        assert!(host.validate_transfer(&cpu, Transfer::Character).is_err());
        assert_eq!(host.armed, None);
        host.ram[usize::from(STACK)..usize::from(STACK) + 2]
            .copy_from_slice(&0x8011_u16.to_le_bytes());
        host.validate_transfer(&cpu, Transfer::Character).unwrap();
        assert_eq!(host.armed, Some(Service::Character));

        host.armed = None;
        cpu.registers_mut().pc = 0x1601;
        host.ram[usize::from(STACK)..usize::from(STACK) + 2]
            .copy_from_slice(&0x8008_u16.to_le_bytes());
        assert!(host
            .validate_transfer(&cpu, Transfer::Channel { sp: STACK - 2 })
            .is_err());
        assert_eq!(host.armed, None);
    }
}
