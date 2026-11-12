//! Ownership and reset (decision-12 rule 9).

use super::{Latches, Registers};

/// An Intel 8085 CPU's registers and execution latches (decision-10).
///
/// `Default` uses the library-chosen register and latch defaults, not a
/// hardware power-up state. Execution is not implemented yet.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct I8085 {
    registers: Registers,
    latches: Latches,
}

impl I8085 {
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

    /// The latches, for changes between calls, including externally setting
    /// the RST 7.5 flip-flop (decision-12 rule 2).
    pub fn latches_mut(&mut self) -> &mut Latches {
        &mut self.latches
    }

    /// Applies RESET (decision-12 rule 9).
    ///
    /// Clears PC, interrupt enable, RST 7.5, TRAP and SOD; sets all three
    /// masks (Intel 205775-002, January 1983, §2.3.7 p. 2-18 reset table).
    /// Leaves halt (§2.3.6 p. 2-16). Clearing the EI delay is unverified.
    /// Registers, flags and SP are preserved by library choice: p. 2-18 and
    /// 231718-001 Table 1, handbook p. 16-13 describe indeterminate results.
    ///
    /// Sampler history and TRAP's saved enable/first-RIM flag are cleared,
    /// and TRAP is armed for a fresh rising sample (rule 9 API bookkeeping).
    /// There is no bus access, poll or `sod_changed` callback.
    pub fn reset(&mut self) {
        self.registers.pc = 0;
        self.latches = Latches {
            interrupt_enable: false,
            ei_delay: false,
            halted: false,
            mask5_5: true,
            mask6_5: true,
            mask7_5: true,
            rst7_5_pending: false,
            trap_pending: false,
            trap_armed: true,
            trap_saved_interrupt_enable: false,
            trap_rim_pending: false,
            sod: false,
            last_trap: false,
            last_rst7_5: false,
        };
    }
}
