//! The Intel 8085 model's register file, execution latches, input contract and
//! reset (decision-6, decision-10, decision-11 and decision-12).
//!
//! Primary source: Intel MCS-80/85 Family User's Manual, 205775-002 (January
//! 1983). Second sources: Intel 8085AH data sheet 231718-001 (September 1987),
//! in the 1988 Embedded Controller Handbook, and the 8080/8085 Assembly Language
//! Programming Manual, 9800301-04 (May 1981). See PROVENANCE.md §1 for pins.
//! Treating the 8085A and 8085AH as one model is an assumption (decision-10).
//!
//! [`I8085`] owns its [`Registers`] and [`Latches`]; the embedder supplies the
//! [`Inputs`]. Construction and reset are available; instruction execution and
//! boundary sampling are not implemented yet. The input documentation fixes
//! their future call order under decision-12.
//!
//! The five flags occupy PSW bits 7, 6, 4, 2 and 0. Bits 5, 3 and 1 are
//! undefined (205775-002 §5.6.5, p. 5-15, FLAG WORD). Their stored image is
//! 0, 0, 1 by library choice (decision-11); hardware witnesses use mask D5h.
//!
//! ```
//! use x80_core::i8085::{I8085, Latches, Registers};
//!
//! let mut registers = Registers::default();
//! registers.set_bc(0x1234);
//! let mut cpu = I8085::new(registers, Latches::default());
//! cpu.reset();
//! assert_eq!(cpu.registers().bc(), 0x1234);
//! assert!(cpu.latches().mask7_5);
//! ```

mod cpu;
mod flags;
mod interrupt;
mod state;

pub use cpu::I8085;
pub use flags::Flags;
pub use interrupt::Inputs;
pub use state::{Latches, Registers};
