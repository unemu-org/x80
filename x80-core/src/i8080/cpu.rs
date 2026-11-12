//! The 8080 model type (decision-5 items 1 and 2, decision-6).
//!
//! It owns the register file and the execution latches. Execution and reset
//! are not implemented yet.

use super::{Latches, Registers};

/// An Intel 8080 CPU: its registers and execution latches.
///
/// The embedder's bus and input lines are borrowed per call, never owned. The
/// type does not execute instructions yet.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct I8080 {
    registers: Registers,
    latches: Latches,
}

impl I8080 {
    /// A CPU with the given registers and execution latches.
    pub const fn new(registers: Registers, latches: Latches) -> Self {
        Self { registers, latches }
    }

    /// The register file.
    pub const fn registers(&self) -> &Registers {
        &self.registers
    }

    /// The register file, for the embedder to change between calls.
    pub fn registers_mut(&mut self) -> &mut Registers {
        &mut self.registers
    }

    /// The execution latches.
    pub const fn latches(&self) -> &Latches {
        &self.latches
    }

    /// The execution latches, for the embedder to change between calls.
    pub fn latches_mut(&mut self) -> &mut Latches {
        &mut self.latches
    }
}

impl Default for I8080 {
    /// All registers and flags zero, interrupts disabled, no EI delay pending
    /// and not halted.
    ///
    /// A library choice, not a hardware claim: 98-153B p. 2-13 says the
    /// register contents are indeterminate at power-up (decision-5 item 2).
    fn default() -> Self {
        Self::new(Registers::default(), Latches::default())
    }
}
