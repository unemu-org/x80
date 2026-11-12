//! Host-side test support shared by the integration tests (std, no
//! dependencies). A test file includes it with `mod support;`.
//!
//! [`RecordingBus`] is a flat 64 KiB memory with scripted port input, scripted
//! interrupt-acknowledge bytes and settable stop and INT levels. It implements
//! [`Bus`], [`Control`] and [`Inputs`] and appends every call, with its
//! arguments and result, to [`RecordingBus::log`], so a test can compare the
//! whole trace with a literal list of [`Event`]s.

// Each integration test is its own crate and uses only part of this module.
#![allow(dead_code)]

use std::collections::VecDeque;

use x80_core::i8080::Inputs;
use x80_core::{Bus, Control, MemoryAccess};

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
///   memory alone.
/// - [`Bus::input`] returns the next byte of `inputs`, whatever the port.
/// - The polls return `stop` and `interrupt`, which a test may change between
///   calls.
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
        self.log.push(Event::InterruptPoll(self.interrupt));
        self.interrupt
    }
}
