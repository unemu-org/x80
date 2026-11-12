//! Ownership, reset and the boundary loop (decision-12 rules 1-7, 9).

use super::opcodes::{Group, OPCODES};
use super::vectored::Source;
use super::Inputs;
use super::{
    acknowledge, arithmetic, branch, data_transfer, logical, rim_sim, stack_io_control, vectored,
};
use super::{Latches, Registers};
use crate::progress::Progress;
use crate::{Budget, Bus, Exit, MemoryAccess, Report, Unsupported};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Interrupt {
    Vectored(Source),
    Intr,
}

/// An Intel 8085 CPU's registers and execution latches (decision-10).
///
/// `Default` uses the library-chosen register and latch defaults, not a
/// hardware power-up state. Group executors land per bead (decision-5 item 7).
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

    /// Executes until a budget limit or another exit (decision-12 rules 1-7).
    ///
    /// Each boundary checks budget, then polls host stop, TRAP, RST 7.5,
    /// RST 6.5, RST 5.5 and INTR exactly once in that order, whatever the
    /// state. Low TRAP always re-arms; it clears pending only when the EI/DI
    /// delay has expired. Its rising edge sets pending. A rising RST 7.5 sets its
    /// flip-flop even while masked or disabled (rules 2 and 6).
    ///
    /// After stop, the highest eligible interrupt goes through its acceptance
    /// seam. Both seams currently decline without effects. Halt follows,
    /// then opcode fetch. Undocumented bytes, and documented instructions whose
    /// group executor has not landed yet (decision-5 item 7), return
    /// [`Exit::Unsupported`] with no instruction effects or progress.
    /// Boundary sampling is retained on Halt, interrupt acceptance and
    /// instruction completion. Stop leaves CPU state unchanged ([`crate::Control`]);
    /// Unsupported restores the latches from before sampling (decision-5 item 6).
    /// A pulse seen only at a Stop or Unsupported boundary is sampled again
    /// next run only if the embedder still holds it.
    /// SID and SOD are untouched; they belong to RIM/SIM (rule 8).
    ///
    /// A zero or reached budget returns without polling or sampling. Started
    /// instructions complete even if they overshoot the cycle budget. Steps
    /// and cycles saturate; an Unsupported report retains earlier progress.
    pub fn run<S: Bus + Inputs>(&mut self, system: &mut S, budget: Budget) -> Report {
        let mut progress = Progress::default();
        loop {
            if progress.reached(budget) {
                return progress.report(Exit::Budget);
            }
            let stop = system.stop_requested();
            let trap = system.trap();
            let rst7_5 = system.rst7_5();
            let rst6_5 = system.rst6_5();
            let rst5_5 = system.rst5_5();
            let intr = system.intr();
            if stop {
                return progress.report(Exit::Stop);
            }
            let boundary_latches = self.latches;
            self.sample_edges(trap, rst7_5);
            match self.pending_interrupt(trap, rst6_5, rst5_5, intr) {
                Some(Interrupt::Vectored(source)) => {
                    if let Some(states) = vectored::accept(self, system, source) {
                        progress.retire(states);
                        continue;
                    }
                }
                Some(Interrupt::Intr) => {
                    if let Some(result) = acknowledge::accept(self, system) {
                        match result {
                            Ok(states) => progress.retire(states),
                            Err(unsupported) => {
                                self.latches = boundary_latches;
                                return progress.report(Exit::Unsupported(unsupported));
                            }
                        }
                        continue;
                    }
                }
                None => {}
            }
            if self.latches.halted {
                return progress.report(Exit::Halt);
            }
            match self.execute(system) {
                Ok(states) => {
                    // Only NOP completes here. EI's one-instruction delay:
                    // 205775-002 p. 5-17; decision-12 rule 6. Unsupported
                    // does not advance it. EI/DI executors land later.
                    self.latches.ei_delay = false;
                    progress.retire(states);
                }
                Err(unsupported) => {
                    self.latches = boundary_latches;
                    return progress.report(Exit::Unsupported(unsupported));
                }
            }
        }
    }

    /// Runs at most one instruction, with an unlimited cycle budget
    /// (decision-5 item 8; decision-12 rule 10).
    pub fn step<S: Bus + Inputs>(&mut self, system: &mut S) -> Report {
        self.run(
            system,
            Budget {
                steps: 1,
                cycles: u64::MAX,
            },
        )
    }

    /// Consecutive boundary levels are an API sampler, not pin-cycle timing
    /// (decision-12 rule 2). TRAP's edge-and-level and low/high re-arm:
    /// 231718-001 p. 16-15, Fig. 4. RST 7.5's latch is set regardless of
    /// mask: 205775-002 pp. 5-17/5-18, RIM/SIM.
    /// Low TRAP cancels pending only at an eligible boundary (rule 2);
    /// it still re-arms during EI/DI inhibition (rule 6).
    fn sample_edges(&mut self, trap: bool, rst7_5: bool) {
        if !trap {
            if !self.latches.ei_delay {
                self.latches.trap_pending = false;
            }
            self.latches.trap_armed = true;
        } else if !self.latches.last_trap && self.latches.trap_armed {
            self.latches.trap_pending = true;
        }
        if rst7_5 && !self.latches.last_rst7_5 {
            self.latches.rst7_5_pending = true;
        }
        self.latches.last_trap = trap;
        self.latches.last_rst7_5 = rst7_5;
    }

    /// Priority and eligibility: decision-12 rule 3; 231718-001 p. 16-13
    /// Table 2, p. 16-15; 205775-002 pp. 5-17/5-18. Suppressing TRAP
    /// during EI delay follows rule 6's explicitly unverified reading.
    fn pending_interrupt(
        &self,
        trap: bool,
        rst6_5: bool,
        rst5_5: bool,
        intr: bool,
    ) -> Option<Interrupt> {
        if self.latches.ei_delay {
            return None;
        }
        if self.latches.trap_pending && trap {
            return Some(Interrupt::Vectored(Source::Trap));
        }
        if !self.latches.interrupt_enable {
            return None;
        }
        if self.latches.rst7_5_pending && !self.latches.mask7_5 {
            Some(Interrupt::Vectored(Source::Rst7_5))
        } else if rst6_5 && !self.latches.mask6_5 {
            Some(Interrupt::Vectored(Source::Rst6_5))
        } else if rst5_5 && !self.latches.mask5_5 {
            Some(Interrupt::Vectored(Source::Rst5_5))
        } else if intr {
            Some(Interrupt::Intr)
        } else {
            None
        }
    }

    /// Dispatch does not advance PC or fetch operands until an executor
    /// completes (decision-5 items 6/7; decision-11).
    fn execute<S: Bus + Inputs>(&mut self, system: &mut S) -> Result<u8, Unsupported> {
        let pc = self.registers.pc;
        let opcode = system.read(pc, MemoryAccess::OpcodeFetch);
        let unsupported = Unsupported {
            pc,
            opcode,
            acknowledge: false,
        };
        let Some(instruction) = OPCODES[usize::from(opcode)] else {
            return Err(unsupported);
        };
        let path = match instruction.group {
            Group::DataTransfer => data_transfer::execute(self, system, opcode, instruction),
            Group::Arithmetic => arithmetic::execute(self, system, opcode, instruction),
            Group::Logical => logical::execute(self, system, opcode, instruction),
            Group::Branch => branch::execute(self, system, opcode, instruction),
            Group::StackIoControl => stack_io_control::execute(self, system, opcode, instruction),
            Group::RimSim => rim_sim::execute(self, system, opcode, instruction),
        };
        path.map(|path| instruction.states.of(path))
            .ok_or(unsupported)
    }
}

#[cfg(test)]
mod tests {
    use super::{Interrupt, Source, I8085};

    // Decision-12 rule 3, 231718-001 Table 2 p. 16-13 and p. 16-15;
    // masks and flip-flop: 205775-002 RIM/SIM pp. 5-17/5-18.
    // Selection is observed here because both acceptance seams still decline.
    #[test]
    fn priority_masks_enable_delay_and_trap_level_select_one_source() {
        let mut cpu = I8085::default();
        cpu.latches.interrupt_enable = true;
        cpu.latches.trap_pending = true;
        cpu.latches.rst7_5_pending = true;
        cpu.latches.mask7_5 = false;
        cpu.latches.mask6_5 = false;
        cpu.latches.mask5_5 = false;
        assert_eq!(
            cpu.pending_interrupt(true, true, true, true),
            Some(Interrupt::Vectored(Source::Trap))
        );
        cpu.latches.interrupt_enable = false;
        assert_eq!(
            cpu.pending_interrupt(true, true, true, true),
            Some(Interrupt::Vectored(Source::Trap))
        );
        assert_eq!(cpu.pending_interrupt(false, true, true, true), None);
        cpu.latches.interrupt_enable = true;
        cpu.latches.ei_delay = true;
        // Rule 6's unverified EI/DI recognition reading includes TRAP.
        assert_eq!(cpu.pending_interrupt(true, true, true, true), None);
        cpu.latches.ei_delay = false;
        assert_eq!(
            cpu.pending_interrupt(false, true, true, true),
            Some(Interrupt::Vectored(Source::Rst7_5))
        );
        cpu.latches.mask7_5 = true;
        assert_eq!(
            cpu.pending_interrupt(false, true, true, true),
            Some(Interrupt::Vectored(Source::Rst6_5))
        );
        cpu.latches.mask6_5 = true;
        assert_eq!(
            cpu.pending_interrupt(false, true, true, true),
            Some(Interrupt::Vectored(Source::Rst5_5))
        );
        cpu.latches.mask5_5 = true;
        assert_eq!(
            cpu.pending_interrupt(false, true, true, true),
            Some(Interrupt::Intr)
        );
        assert_eq!(cpu.pending_interrupt(false, true, true, false), None);
        cpu.latches.mask7_5 = false;
        cpu.latches.mask6_5 = false;
        cpu.latches.mask5_5 = false;
        cpu.latches.rst7_5_pending = false;
        assert_eq!(
            cpu.pending_interrupt(false, false, false, true),
            Some(Interrupt::Intr)
        );
        assert_eq!(cpu.pending_interrupt(false, false, false, false), None);
    }
}
