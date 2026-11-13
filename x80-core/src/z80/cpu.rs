//! State ownership and reset (decision-13 rule 1; decision-15 rule 12).

use super::{Latches, Registers};

/// A Z80 CPU's registers and execution latches (decision-13 rule 1).
///
/// `Default` uses library construction values, not a hardware power-up
/// state (decision-15 rule 12). Instruction execution is not implemented yet.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Z80 {
    registers: Registers,
    latches: Latches,
}

impl Z80 {
    /// Owns the supplied state unchanged.
    pub const fn new(registers: Registers, latches: Latches) -> Self {
        Self { registers, latches }
    }

    /// The register file.
    pub const fn registers(&self) -> &Registers {
        &self.registers
    }

    /// The register file, for changes between calls.
    pub fn registers_mut(&mut self) -> &mut Registers {
        &mut self.registers
    }

    /// The execution and sampler latches.
    pub const fn latches(&self) -> &Latches {
        &self.latches
    }

    /// The execution and sampler latches, for changes between calls.
    pub fn latches_mut(&mut self) -> &mut Latches {
        &mut self.latches
    }

    /// Applies RESET (decision-15 rule 12).
    ///
    /// Clears IFF1/IFF2, PC, I and R; selects IM 0 (Zilog UM008011-0816,
    /// revision 11, August 2016, RESET p. 6 and Table 1 p. 18, PDF 20/32;
    /// 03-0029-01, 1977, RESET p. 9 and Fig. 8.0-1 p. 56, PDF 15/62).
    /// Leaves halt (UM HALT p. 181, PDF 195).
    ///
    /// Preserving SP, both general/AF register sets, IX, IY and flags is a
    /// library choice: those reset descriptions do not specify their values.
    /// Clearing EI delay and the parity-note latch and setting NMI history
    /// to not asserted are library bookkeeping choices, not hardware claims.
    /// No bus access or poll occurs, and no elapsed cycles are reported.
    pub fn reset(&mut self) {
        self.registers.pc = 0;
        self.registers.i = 0;
        self.registers.r = 0;
        self.latches = Latches::default();
    }
}
