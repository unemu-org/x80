//! The Z80 model's register file, flags, execution latches, input contract and
//! reset, prefix dispatch and run/step (decisions 13–15). Only NOP executes
//! during group staging; interrupt acceptance is not implemented yet.
//!
//! Primary source: Zilog Z80 CPU User Manual UM008011-0816, revision 11,
//! August 2016 (printed page + 14 = PDF page). Second witness here: Zilog
//! Z80-CPU / Z80A-CPU Technical Manual 03-0029-01, 1977 (printed + 6 = PDF).
//! The other pinned second witnesses and their shared Zilog documentation
//! lineage are recorded in PROVENANCE.md §1; agreement is not a hardware
//! measurement. One model for NMOS and CMOS parts is an unverified assumption
//! and library scope choice, not a claim of silicon equivalence (decision-13
//! rule 6).
//!
//! [`Z80`] owns its [`Registers`] and [`Latches`]; the embedder supplies
//! [`Inputs`]. Construction defaults are library values, not a power-up claim.
//! The six flags occupy F/F' bits 7, 6, 4, 2, 1 and 0 (UM Tables 21/22,
//! pp. 65–66). Bits 5 and 3 are unused there; reading them as zero and
//! ignoring their input values is a library choice (decision-14 rule 5).
//! Hardware comparisons use mask D7h.
//!
//! ```
//! use x80_core::z80::{Z80, Latches, Registers};
//!
//! let mut registers = Registers::default();
//! registers.set_bc_alternate(0x1234);
//! let mut cpu = Z80::new(registers, Latches::default());
//! cpu.reset();
//! assert_eq!(cpu.registers().bc_alternate(), 0x1234);
//! assert!(!cpu.latches().iff1);
//! ```

mod arithmetic16;
mod arithmetic8;
mod bit_set_reset_test;
mod call_return;
mod exchange_block_search;
mod general_arithmetic_cpu_control;
mod input_output;
mod jump;
mod load16;
mod load8;
mod opcodes;
mod rotate_shift;

mod acknowledge;
mod cpu;
mod flags;
mod interrupt;
mod nmi;
mod state;

pub use cpu::Z80;
pub use flags::Flags;
pub use interrupt::Inputs;
pub use state::{InterruptMode, Latches, Registers};
