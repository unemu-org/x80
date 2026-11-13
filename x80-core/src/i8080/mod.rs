//! The Intel 8080 model (decision-6): its register file, execution latches,
//! input lines, CPU type, opcode map and instruction groups.
//!
//! Primary source: Intel 8080 Microcomputer Systems User's Manual, 98-153B;
//! second witness: Intel 8080/8085 Assembly Language Programming Manual,
//! 9800301-04 (PROVENANCE.md §1). Every hardware fact in this module cites its
//! page.
//!
//! [`I8080::run`] and [`I8080::step`] execute under a [`Budget`](crate::Budget)
//! with the boundary sequence of decision-5. The private opcode map records the
//! 244 opcode bytes that 98-153B Ch. 4 documents; the other 12 return
//! [`Exit::Unsupported`](crate::Exit::Unsupported). Each Ch. 4 instruction
//! group has its own module.
//!
//! All documented instructions execute. Eligible interrupts accept an
//! acknowledged RST, and [`I8080::reset`] applies the model's reset behavior
//! (decision-5 items 4, 7 and 9).
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

mod alu;
mod arithmetic;
mod branch;
mod cpu;
mod data_transfer;
mod flags;
mod interrupt;
mod logical;
mod opcodes;
mod stack_io_control;
mod state;

pub use cpu::I8080;
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
