//! Reference loop for decision-9 and its amendments / plan §3, built on the
//! public x80 API. NEC citations: Stock No. 500350, October 1986 (see
//! native.rs).
//!
//! Option B and the view exchange (decision-9 amendment, 2026-11-15): every
//! run copies x80 in from the native model and out again at every exit. The
//! adapter keeps no 8080 register or latch of its own between runs. On ED it
//! hands back `EdEscape` (x86jit: `I8080Handback::EdEscape`) and the native
//! model decodes the escape itself.

use super::assumptions;
use super::native::{Escape, Memory, Native, BRK, IE};
use x80_core::i8080::Inputs;
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
    /// x80 refused a non-ED opcode (decision-3). The adapter hands back a
    /// typed refusal: MD stays 0 and nothing is decoded natively.
    Opcode {
        pc: u16,
        opcode: u8,
    },
    /// x80 stopped on ED and the native core's decode of ED at PS:PC found
    /// neither ED FD nor ED ED nn: the native side's unknown instruction.
    NativeUnknown {
        pc: u16,
        bytes: [u8; 2],
    },
    /// The native core refused BRKEM with BRK = 1 (decision-9 amendment,
    /// 2026-11-15): no effect, emulation never starts and x80 never runs.
    NativeSingleStep,
    UnexpectedCoreExit,
    Ended,
}

/// One access through x80's bus. The native model's own reads and writes
/// (frames, vectors and the ED decode) never appear here.
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
        // One flat image: 8080 stores are visible to the native side at
        // once (an embedder must also invalidate native translations).
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
    // decision-9 amendment, 2026-11-15: an embedder's stop request must also
    // carry the host's own stop or cancel; the fixture scripts none.
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
    pub fn enter(mut native: Native, mut memory: Memory, vector: u8) -> Result<Self, Refusal> {
        native
            .brkem(&mut memory, vector)
            .map_err(|_| Refusal::NativeSingleStep)?;
        Ok(Self {
            native,
            memory,
            accesses: Vec::new(),
            ports: Vec::new(),
            services: Vec::new(),
            routine_psws: Vec::new(),
            boundaries: Vec::new(),
            ended: false,
        })
    }

    pub fn run(&mut self, steps: u64, lines: Lines) -> Result<Outcome, Refusal> {
        if self.ended {
            return Err(Refusal::Ended);
        }
        // NEC p. 6-2: high INT at a boundary, IE=1, no NMI or hold.
        // Native EI delay applicability is an ASSUMPTION, not a NEC fact.
        let int_accepted = lines.int
            && self.native.psw & IE != 0
            && (!assumptions::EMULATION_EI_DELAY || !self.native.ei_delay)
            && !lines.nmi
            && !lines.hold;
        // NEC pp. 8-2, 9-1: NMI unconditional; masked INT wakes HLT.
        let wake = self.native.halted && lines.int && self.native.psw & IE == 0;
        let stop = lines.reset || lines.nmi || int_accepted || wake;
        // Copy-in: x80 is rebuilt from the native model alone.
        let mut cpu = self.native.copy_in();
        let report = {
            let mut bus = SegmentedBus {
                memory: &mut self.memory,
                accesses: &mut self.accesses,
                ports: &mut self.ports,
                ps: self.native.ps,
                ds0: self.native.ds0,
                stop,
            };
            cpu.run(
                &mut bus,
                Budget {
                    steps,
                    cycles: u64::MAX,
                },
            )
        };
        self.native.copy_out(&cpu);
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
                    let saved_psw = self.native.psw;
                    self.native.enter_service(&mut self.memory, source, vector);
                    self.routine(saved_psw);
                    Ok(Outcome::Serviced)
                } else {
                    // NEC p. 9-1: masked INT must stay high through the
                    // instruction following HLT. The script token spans
                    // wake AND that instruction; no clocks are inferred.
                    // The adapter hands back halted = 0 for the wake.
                    self.native.halted = false;
                    match self.run(1, lines)? {
                        Outcome::Budget => Ok(Outcome::Wake),
                        outcome => Ok(outcome),
                    }
                }
            }
            // decision-9: the adapter never asserts INT, so x80 never
            // refuses an acknowledge under the profile.
            Exit::Unsupported(u) if u.acknowledge => unreachable!("{u:?}"),
            Exit::Unsupported(u) if u.opcode == 0xED => {
                // Option B: hand back EdEscape without reading past ED.
                assert_eq!(self.native.pc, u.pc);
                let saved_psw = self.native.psw;
                match self.native.escape(&mut self.memory) {
                    Escape::Retem => {
                        self.ended = true;
                        Ok(Outcome::Retem)
                    }
                    Escape::Calln => {
                        self.routine(saved_psw);
                        Ok(Outcome::Serviced)
                    }
                    Escape::Unknown { bytes } => Err(Refusal::NativeUnknown { pc: u.pc, bytes }),
                }
            }
            Exit::Unsupported(u) => Err(Refusal::Opcode {
                pc: u.pc,
                opcode: u.opcode,
            }),
            _ => Err(Refusal::UnexpectedCoreExit),
        }
    }

    /// The native routine after INT, NMI or CALLN entry, then RETI back
    /// into emulation. RETI is an operation, not a decoded instruction.
    fn routine(&mut self, saved_psw: u16) {
        self.services.push(self.native);
        self.native.mock_routine(saved_psw);
        self.routine_psws.push(self.native.psw);
        // A RETI that starts with BRK = 1 and re-enters 8080 mode would be
        // refused natively, like BRKEM; service entry clears BRK and the mock
        // routine leaves it clear.
        assert_eq!(self.native.psw & BRK, 0);
        // NEC p. 12-168 / p. 8-4 Fig. 8-6: restores MD=0 and returns.
        self.native.return_frame(&self.memory);
        assert_eq!(self.native.psw, saved_psw);
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
