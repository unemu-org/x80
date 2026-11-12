//! Host-side 8080 test support shared by the integration tests (std, no
//! dependencies). A test file includes it with `mod support;`.
//!
//! [`RecordingBus`] is a flat 64 KiB memory with scripted port input, scripted
//! interrupt-acknowledge bytes, per-poll INT levels and settable stop and INT
//! levels. It implements
//! [`Bus`], [`Control`] and [`Inputs`] and appends every call, with its
//! arguments and result, to [`RecordingBus::log`], so a test can compare the
//! whole trace with a literal list of [`Event`]s.
//!
//! [`sample_registers`] and [`sample_latches`] give a CPU state with a
//! distinct value everywhere, and [`assert_unsupported`] checks the
//! unsupported exit of decision-5 items 5 and 6 for one opcode.

// Each integration test is its own crate and uses only part of this module.
#![allow(dead_code)]

use std::collections::VecDeque;

use x80_core::i8080::{Flags, Inputs, Latches, Registers, I8080};
use x80_core::{Bus, Control, Exit, MemoryAccess, Unsupported};

/// One call into the test bus, in call order.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Event {
    /// [`Bus::read`]: `value` is the byte returned.
    Read {
        address: u16,
        kind: MemoryAccess,
        value: u8,
    },
    /// [`Bus::write`]: `value` is the byte written.
    Write {
        address: u16,
        kind: MemoryAccess,
        value: u8,
    },
    /// [`Bus::input`]: `value` is the byte returned.
    Input { port: u8, value: u8 },
    /// [`Bus::output`]: `value` is the byte written.
    Output { port: u8, value: u8 },
    /// [`Control::stop_requested`], with the level returned.
    StopPoll(bool),
    /// [`Inputs::interrupt_requested`], with the level returned.
    InterruptPoll(bool),
}

/// A recording bus with scriptable responses; see the module docs.
///
/// - Memory reads of every kind except [`MemoryAccess::InterruptAcknowledge`]
///   return `memory[address]`; writes store into it.
/// - An acknowledge read returns the next byte of `acknowledges` and leaves
///   memory alone. It also lowers `interrupt` if `lower_interrupt_on_acknowledge`
///   is set.
/// - [`Bus::input`] returns the next byte of `inputs`, whatever the port.
/// - Stop polls return `stop`. INT polls consume `interrupt_levels`, falling
///   back to `interrupt` when the script is empty.
///
/// A read from an empty script panics: the test did not expect that access.
pub struct RecordingBus {
    /// The 65,536 bytes of memory, indexed by address.
    pub memory: Vec<u8>,
    /// Bytes returned by successive [`Bus::input`] calls.
    pub inputs: VecDeque<u8>,
    /// Bytes returned by successive interrupt-acknowledge reads.
    pub acknowledges: VecDeque<u8>,
    /// Level returned by [`Control::stop_requested`].
    pub stop: bool,
    /// Level returned by [`Inputs::interrupt_requested`].
    pub interrupt: bool,
    /// Optional levels returned by successive INT polls, before `interrupt`.
    pub interrupt_levels: VecDeque<bool>,
    /// Lower the fallback INT level when an acknowledge read occurs.
    pub lower_interrupt_on_acknowledge: bool,
    /// Every call so far, oldest first.
    pub log: Vec<Event>,
}

impl RecordingBus {
    /// Zeroed memory, empty scripts, both levels low and an empty log.
    pub fn new() -> Self {
        Self {
            memory: vec![0; 0x1_0000],
            inputs: VecDeque::new(),
            acknowledges: VecDeque::new(),
            stop: false,
            interrupt: false,
            interrupt_levels: VecDeque::new(),
            lower_interrupt_on_acknowledge: false,
            log: Vec::new(),
        }
    }

    /// Copies `bytes` into memory from `address` on, wrapping at 0xFFFF. Not
    /// logged.
    pub fn load(&mut self, address: u16, bytes: &[u8]) {
        let mut at = address;
        for &byte in bytes {
            self.memory[usize::from(at)] = byte;
            at = at.wrapping_add(1);
        }
    }

    /// Returns the log and leaves it empty.
    pub fn take_log(&mut self) -> Vec<Event> {
        std::mem::take(&mut self.log)
    }
}

impl Default for RecordingBus {
    fn default() -> Self {
        Self::new()
    }
}

impl Bus for RecordingBus {
    fn read(&mut self, address: u16, kind: MemoryAccess) -> u8 {
        let value = if kind == MemoryAccess::InterruptAcknowledge {
            if self.lower_interrupt_on_acknowledge {
                self.interrupt = false;
            }
            self.acknowledges
                .pop_front()
                .unwrap_or_else(|| panic!("unscripted interrupt acknowledge at {address:#06x}"))
        } else {
            self.memory[usize::from(address)]
        };
        self.log.push(Event::Read {
            address,
            kind,
            value,
        });
        value
    }

    fn write(&mut self, address: u16, value: u8, kind: MemoryAccess) {
        self.memory[usize::from(address)] = value;
        self.log.push(Event::Write {
            address,
            kind,
            value,
        });
    }

    fn input(&mut self, port: u8) -> u8 {
        let value = self
            .inputs
            .pop_front()
            .unwrap_or_else(|| panic!("unscripted input from port {port:#04x}"));
        self.log.push(Event::Input { port, value });
        value
    }

    fn output(&mut self, port: u8, value: u8) {
        self.log.push(Event::Output { port, value });
    }
}

impl Control for RecordingBus {
    fn stop_requested(&mut self) -> bool {
        self.log.push(Event::StopPoll(self.stop));
        self.stop
    }
}

impl Inputs for RecordingBus {
    fn interrupt_requested(&mut self) -> bool {
        let level = self.interrupt_levels.pop_front().unwrap_or(self.interrupt);
        self.log.push(Event::InterruptPoll(level));
        level
    }
}

/// Distinct values in every register and a mixed flag pattern, so a stray
/// change is visible. `pc` is where the next opcode is read.
pub fn sample_registers(pc: u16) -> Registers {
    Registers {
        a: 0xA1,
        flags: Flags {
            sign: true,
            zero: false,
            auxiliary_carry: true,
            parity: false,
            carry: true,
        },
        b: 0xB2,
        c: 0xC3,
        d: 0xD4,
        e: 0xE5,
        h: 0xF6,
        l: 0x07,
        sp: 0x8F90,
        pc,
    }
}

/// Latches with interrupts enabled and an EI delay pending, not halted, so a
/// stray change to either is visible.
pub fn sample_latches() -> Latches {
    Latches {
        interrupt_enable: true,
        ei_delay: true,
        halted: false,
    }
}

/// The sample latches after one completed non-EI instruction: the pending
/// delay expires (98-153B Ch. 4 EI p. 4-14; decision-5 item 4).
pub fn settled_latches() -> Latches {
    Latches {
        interrupt_enable: true,
        ei_delay: false,
        halted: false,
    }
}

/// Whether `exit` is the unsupported exit of an opcode fetch (decision-5
/// item 6): `pc` and `opcode` as given, `acknowledge` false.
pub fn is_fetch_unsupported(exit: Exit, pc: u16, opcode: u8) -> bool {
    matches!(
        exit,
        Exit::Unsupported(Unsupported {
            pc: at,
            opcode: read,
            acknowledge: false,
            ..
        }) if at == pc && read == opcode
    )
}

/// Steps once over `opcode` at `pc` and checks the unsupported exit of
/// decision-5 items 5 and 6: no step and no cycle, registers, flags, latches
/// and memory unchanged, and a trace of exactly the two polls and one opcode
/// fetch at `pc`.
pub fn assert_unsupported(opcode: u8, pc: u16) {
    let mut bus = RecordingBus::new();
    // Distinct bytes around the opcode, so a stray operand read or write is
    // visible in the trace or in memory.
    bus.load(pc.wrapping_sub(1), &[0x5A, opcode, 0xA5, 0x3C]);
    let memory = bus.memory.clone();
    let mut cpu = I8080::new(sample_registers(pc), sample_latches());

    let report = cpu.step(&mut bus);

    assert_eq!(
        (report.steps, report.cycles),
        (0, 0),
        "opcode {opcode:#04x}"
    );
    assert!(
        is_fetch_unsupported(report.exit, pc, opcode),
        "opcode {opcode:#04x}: {:?}",
        report.exit
    );
    assert_eq!(
        *cpu.registers(),
        sample_registers(pc),
        "opcode {opcode:#04x}"
    );
    assert_eq!(*cpu.latches(), sample_latches(), "opcode {opcode:#04x}");
    assert!(bus.memory == memory, "opcode {opcode:#04x}: memory changed");
    assert_eq!(
        bus.take_log(),
        [
            Event::StopPoll(false),
            Event::InterruptPoll(false),
            Event::Read {
                address: pc,
                kind: MemoryAccess::OpcodeFetch,
                value: opcode,
            },
        ],
        "opcode {opcode:#04x}"
    );
}
