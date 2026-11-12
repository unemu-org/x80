//! The Intel 8085 model's register file, execution latches, input contract and
//! reset and boundary execution (decision-6, decision-10, decision-11 and
//! decision-12).
//!
//! Primary source: Intel MCS-80/85 Family User's Manual, 205775-002 (January
//! 1983). Second sources: Intel 8085AH data sheet 231718-001 (September 1987),
//! in the 1988 Embedded Controller Handbook, and the 8080/8085 Assembly Language
//! Programming Manual, 9800301-04 (May 1981). See PROVENANCE.md §1 for pins.
//! Treating the 8085A and 8085AH as one model is an assumption (decision-10).
//!
//! [`I8085`] owns its [`Registers`] and [`Latches`]; the embedder supplies the
//! [`Inputs`]. [`I8085::run`] and [`I8085::step`] follow the full decision-12
//! boundary sequence, including persistent TRAP/RST 7.5 edge sampling.
//! Stop preserves CPU state ([`crate::Control`]); Unsupported restores the
//! latches from before sampling (decision-5 item 6). A pulse seen only at
//! either boundary is sampled again next run only if the embedder still holds it.
//! Every documented instruction executes, including RIM/SIM. Undocumented
//! bytes return Unsupported (decision-11; decision-5 item 7 closure). All
//! five interrupt sources execute; INTR supports RST and CALL acknowledges
//! and returns Unsupported for other acknowledge bytes (decision-12 rule 5).
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

mod acknowledge;
mod alu;
mod arithmetic;
mod branch;
mod cpu;
mod data_transfer;
mod flags;
mod interrupt;
mod logical;
mod opcodes;
mod rim_sim;
mod stack_io_control;
mod state;
mod vectored;

pub use cpu::I8085;
pub use flags::Flags;
pub use interrupt::Inputs;
pub use state::{Latches, Registers};
