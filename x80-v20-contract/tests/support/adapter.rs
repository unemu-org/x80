//! Reference loop for decision-9 / plan §3, built on the public x80 API.
//! NEC citations: Stock No. 500350, October 1986 (see native.rs).

use super::assumptions;
use super::native::{offset_add, Memory, Native, IE};
use x80_core::i8080::{Inputs, Latches, I8080};
use x80_core::{Budget, Bus, Control, Exit, MemoryAccess};

#[derive(Clone, Copy, Debug, Default)]
pub struct Lines {
    pub int: bool,
    pub nmi: bool,
    pub hold: bool,
    pub reset: bool,
    pub vector: u8,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Outcome {
    Budget,
    Halt,
    Retem,
    Serviced,
    Wake,
    Reset,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Refusal {
    Opcode { pc: u16, opcode: u8 },
    Escape { pc: u16, second: u8 },
    UnexpectedCoreExit,
    Ended,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Access {
    pub kind: MemoryAccess,
    pub segment: u16,
    pub offset: u16,
    pub value: u8,
    pub write: bool,
}

struct SegmentedBus<'a> {
    memory: &'a mut Memory,
    accesses: &'a mut Vec<Access>,
    ports: &'a mut Vec<(u16, u8, bool)>,
    ps: u16,
    ds0: u16,
    stop: bool,
}

impl Bus for SegmentedBus<'_> {
    fn read(&mut self, offset: u16, kind: MemoryAccess) -> u8 {
        // NEC p. 8-1: code PS, memory operands (including stack) DS0.
        let segment = match kind {
            MemoryAccess::OpcodeFetch | MemoryAccess::OperandFetch => self.ps,
            MemoryAccess::Data | MemoryAccess::Stack => self.ds0,
            _ => panic!("V20 profile must never issue an interrupt acknowledge"),
        };
        let value = self.memory.read(segment, offset);
        self.accesses.push(Access {
            kind,
            segment,
            offset,
            value,
            write: false,
        });
        value
    }

    fn write(&mut self, offset: u16, value: u8, kind: MemoryAccess) {
        assert!(matches!(kind, MemoryAccess::Data | MemoryAccess::Stack));
        self.memory.write(self.ds0, offset, value);
        self.accesses.push(Access {
            kind,
            segment: self.ds0,
            offset,
            value,
            write: true,
        });
    }

    fn input(&mut self, port: u8) -> u8 {
        let address = u16::from_be_bytes([assumptions::IO_HIGH_BYTE, port]);
        self.ports.push((address, 0xA6, false)); // fixture device payload
        0xA6
    }

    fn output(&mut self, port: u8, value: u8) {
        let address = u16::from_be_bytes([assumptions::IO_HIGH_BYTE, port]);
        self.ports.push((address, value, true));
    }
}

impl Control for SegmentedBus<'_> {
    fn stop_requested(&mut self) -> bool {
        self.stop
    }
}

impl Inputs for SegmentedBus<'_> {
    // decision-9: the adapter NEVER asserts x80's INT input.
    fn interrupt_requested(&mut self) -> bool {
        false
    }
}

pub struct Adapter {
    pub native: Native,
    pub memory: Memory,
    pub cpu: I8080,
    pub accesses: Vec<Access>,
    pub ports: Vec<(u16, u8, bool)>,
    /// State at native routine entry, before the scripted mock payload.
    pub services: Vec<Native>,
    pub routine_psws: Vec<u16>,
    /// Copy-out at EVERY core exit, including budget, halt and refusal.
    pub boundaries: Vec<Native>,
    pub ended: bool,
}

impl Adapter {
    pub fn enter(mut native: Native, mut memory: Memory, vector: u8) -> Self {
        native.brkem(&mut memory, vector);
        let cpu = native.copy_in(Latches::default());
        Self {
            native,
            memory,
            cpu,
            accesses: Vec::new(),
            ports: Vec::new(),
            services: Vec::new(),
            routine_psws: Vec::new(),
            boundaries: Vec::new(),
            ended: false,
        }
    }

    pub fn run(&mut self, steps: u64, lines: Lines) -> Result<Outcome, Refusal> {
        if self.ended {
            return Err(Refusal::Ended);
        }
        // NEC p. 6-2: high INT at a boundary, IE=1, no NMI or hold.
        // Native EI delay applicability is an ASSUMPTION, not a NEC fact.
        let int_accepted = lines.int
            && self.native.psw & IE != 0
            && (!assumptions::EMULATION_EI_DELAY || !self.cpu.latches().ei_delay)
            && !lines.nmi
            && !lines.hold;
        // NEC pp. 8-2, 9-1: NMI unconditional; masked INT wakes HLT.
        let wake = self.cpu.latches().halted && lines.int && self.native.psw & IE == 0;
        let stop = lines.reset || lines.nmi || int_accepted || wake;
        let report = {
            let mut bus = SegmentedBus {
                memory: &mut self.memory,
                accesses: &mut self.accesses,
                ports: &mut self.ports,
                ps: self.native.ps,
                ds0: self.native.ds0,
                stop,
            };
            self.cpu.run(
                &mut bus,
                Budget {
                    steps,
                    cycles: u64::MAX,
                },
            )
        };
        self.native.copy_out(&self.cpu);
        self.boundaries.push(self.native);
        match report.exit {
            Exit::Budget => Ok(Outcome::Budget),
            Exit::Halt => Ok(Outcome::Halt),
            Exit::Stop => {
                if lines.reset {
                    self.native.reset();
                    self.ended = true;
                    Ok(Outcome::Reset)
                } else if lines.nmi || int_accepted {
                    // NMI vector 2: NEC p. 6-1 Table 6-1. A Lines.nmi
                    // token is an already-latched rising-edge request.
                    let (source, vector) = if lines.nmi {
                        ("nmi", 2)
                    } else {
                        ("int", lines.vector)
                    };
                    self.service(source, vector);
                    Ok(Outcome::Serviced)
                } else {
                    // NEC p. 9-1: masked INT must stay high through the
                    // instruction following HLT. The script token spans
                    // wake AND that instruction; no clocks are inferred.
                    self.cpu.latches_mut().halted = false;
                    match self.run(1, lines)? {
                        Outcome::Budget => Ok(Outcome::Wake),
                        outcome => Ok(outcome),
                    }
                }
            }
            Exit::Unsupported(u) if !u.acknowledge && u.opcode == 0xED => {
                let second = self.escape_byte(offset_add(u.pc, 1));
                match second {
                    0xFD => {
                        // NEC p. 12-177: restore original native PSW,
                        // discarding emulated flags; registers carry out.
                        self.native.return_frame(&self.memory);
                        self.ended = true;
                        Ok(Outcome::Retem)
                    }
                    0xED => {
                        // NEC p. 12-176; saved continuation is ASSUMED.
                        let vector = self.escape_byte(offset_add(u.pc, 2));
                        self.native.pc = offset_add(u.pc, i32::from(assumptions::CALLN_PC_ADVANCE));
                        self.service("calln", vector);
                        Ok(Outcome::Serviced)
                    }
                    second => Err(Refusal::Escape { pc: u.pc, second }),
                }
            }
            Exit::Unsupported(u) => Err(Refusal::Opcode {
                pc: u.pc,
                opcode: u.opcode,
            }),
            _ => Err(Refusal::UnexpectedCoreExit),
        }
    }

    fn escape_byte(&mut self, offset: u16) -> u8 {
        let value = self.memory.read(self.native.ps, offset);
        self.accesses.push(Access {
            kind: MemoryAccess::OperandFetch,
            segment: self.native.ps,
            offset,
            value,
            write: false,
        });
        value
    }

    fn service(&mut self, source: &'static str, vector: u8) {
        let mut latches = *self.cpu.latches();
        let saved_psw = self.native.psw;
        self.native.enter_service(&mut self.memory, source, vector);
        self.services.push(self.native);
        self.native.mock_routine(saved_psw);
        self.routine_psws.push(self.native.psw);
        // RETI is an operation, not a decoded/executed native instruction.
        // NEC p. 12-168 / p. 8-4 Fig. 8-6: restores MD=0 and returns.
        self.native.return_frame(&self.memory);
        assert_eq!(self.native.psw, saved_psw);
        latches.halted = false; // NEC p. 9-1: return after HLT
        self.cpu = self.native.copy_in(latches);
    }

    pub fn until(&mut self, expected: Outcome) {
        // Fixture cap, not a V20 execution budget or clock claim.
        for _ in 0..64 {
            let outcome = self
                .run(1, Lines::default())
                .expect("reference adapter refusal");
            if outcome == expected {
                return;
            }
            assert!(
                matches!(outcome, Outcome::Budget | Outcome::Serviced),
                "{outcome:?}"
            );
        }
        panic!("fixture exceeded instruction cap");
    }
}
