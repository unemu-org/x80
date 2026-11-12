//! The Intel 8080 model (decision-6): its register file, execution latches,
//! input lines and CPU type.
//!
//! Primary source: Intel 8080 Microcomputer Systems User's Manual, 98-153B;
//! second witness: Intel 8080/8085 Assembly Language Programming Manual,
//! 9800301-04 (PROVENANCE.md §1). Every hardware fact in this module cites its
//! page. Status: the types exist; nothing executes yet.
//!
//! ```
//! use x80_core::i8080::{I8080, Latches, Registers};
//!
//! let mut registers = Registers::default();
//! registers.set_hl(0x1234);
//! registers.sp = 0x8000;
//! let mut cpu = I8080::new(registers, Latches::default());
//! cpu.registers_mut().pc = 0x0100;
//! assert_eq!(cpu.registers().h, 0x12);
//! assert!(!cpu.latches().halted);
//! ```

mod cpu;
mod flags;
mod interrupt;
mod state;

pub use cpu::I8080;
pub use flags::Flags;
pub use interrupt::Inputs;
pub use state::{Latches, Registers};
