//! The Z80 model's register file, flags, execution latches, input contract and
//! reset, prefix dispatch and run/step (decisions 13–15). Every documented
//! encoding executes. Decision-20 §§1–3 also admit all ED terminals, index
//! halves, ignored/repeated DD/FD, SLL and indexed-CB copies/BIT aliases.
//! No normally fetched encoding refuses; Unsupported remains for IM0 non-RST
//! and odd-IM2 acknowledgements (decision-15 rule 9). NMI and INT in modes
//! 0, 1 and 2 are accepted within those restrictions (rules 8 and 9).
//!
//! Primary source: Zilog Z80 CPU User Manual UM008011-0816, revision 11,
//! August 2016 (printed page + 14 = PDF page). Second witness here: Zilog
//! Z80-CPU / Z80A-CPU Technical Manual 03-0029-01, 1977 (printed + 6 = PDF).
//! The other pinned second witnesses and their shared Zilog documentation
//! lineage are recorded in PROVENANCE.md §1; agreement is not a hardware
//! measurement. Decision-20 §7 selects Zilog NMOS; other silicon variants
//! need a separate decision-6 scope decision.
//!
//! [`Z80`] owns its [`Registers`] and [`Latches`]; the embedder supplies
//! [`Inputs`]. Construction defaults are library values, not a power-up claim.
//! F/F' store all eight bits independently, including Y5 and X3
//! (decision-20 §4, library representation). H transfer rules: Young v0.91
//! §2.3 p. 8 / §§8.2–8.3 pp. 28–29; Rak v1.2a .selftest/.poppush_af/
//! .ex_af_af. Hardware comparisons for admitted flags use the full byte.
//! `Latches::wz` supplies the retained 16-bit WZ/MEMPTR for complete state
//! snapshots and deterministic continuation (decision-20 §5 / §7).
//! Exact update equations, Default=0 and reset preservation are L library
//! rules; only the discriminated BIT projections have hardware observations.
//! Stop/Unsupported preserve the complete state; Budget does not sample it.
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

mod alu;
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
