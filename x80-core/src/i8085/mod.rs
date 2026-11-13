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
//! The seven independent flags pack as S Z K AC 0 P V CY (decision-18 §4),
//! replacing decision-11's filler bits. Image D/S: W p. 145/Tundra
//! 9000000.MD302.01 Fig. 2-9 p. 2-15, independently KV bus drivers/FC
//! storage (PROVENANCE §1; part/revision qualifications in [`Flags`]).
//! POP/set_psw load V/K independently and discard bit 3 as an L library
//! rule. Default clears V/K; reset preserves them (L). The existing five
//! documented flags still occupy bits 7, 6, 4, 2 and 0; D5h masks exclude V/K.
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

// Concrete register borrowing is a library choice (decision-16 rule 2).
fn view(registers: &mut Registers) -> crate::intel::View<'_, Flags> {
    crate::intel::View {
        a: &mut registers.a,
        flags: &mut registers.flags,
        b: &mut registers.b,
        c: &mut registers.c,
        d: &mut registers.d,
        e: &mut registers.e,
        h: &mut registers.h,
        l: &mut registers.l,
        sp: &mut registers.sp,
        pc: &mut registers.pc,
    }
}
