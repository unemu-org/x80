//! Bit Set, Reset, and Test Group: encodings and executor (decision-13 rule 2).
//!
//! Primary: Zilog UM008011-0816, rev. 11 (August 2016), pp. 242–260.
//! Entry citations below use printed pages; PDF = printed + 14.
//! Independent cross-read: 03-0029-01 (1977), Table 7.0-8, p. 51
//! (PDF = printed + 6). Whole-instruction lengths and T-state totals include
//! every prefix, operand and displacement (decision-13 rules 3–5).
//! Execution witnesses: tests/z80_bit_set_reset_test.rs. BIT unknown flags
//! follow decision-14 rule 6; bus kinds follow decision-15 rule 5.

use super::cpu::{Decoded, IndexRegister, Table};
use super::opcodes::{Encoding, Instruction, States};
use super::Z80;
use crate::decode::Path;
use crate::{Bus, MemoryAccess};

// Compact Table 14 is corrupt: entry patterns prevail (PROVENANCE.md §1).

pub(super) const CB: &[Encoding] = &[
    // UM BIT b,r, pp. 243–244.
    Encoding::new(b"01BBBRRR", 2, States::single(8)),
    // UM BIT b,(HL), pp. 245–246.
    Encoding::new(b"01BBB110", 2, States::single(12)),
    // UM SET b,r, pp. 251–252.
    Encoding::new(b"11BBBRRR", 2, States::single(8)),
    // UM SET b,(HL), pp. 253–254.
    Encoding::new(b"11BBB110", 2, States::single(15)),
    // UM RES b,r, pp. 259–260.
    Encoding::new(b"10BBBRRR", 2, States::single(8)),
    // UM RES b,(HL), pp. 259–260.
    Encoding::new(b"10BBB110", 2, States::single(15)),
];

pub(super) const INDEX_CB: &[Encoding] = &[
    // UM BIT b,(IX/IY+d), pp. 247–250.
    Encoding::new(b"01BBB110", 4, States::single(20)),
    // UM SET b,(IX/IY+d), pp. 255–258.
    Encoding::new(b"11BBB110", 4, States::single(23)),
    // UM RES b,(IX/IY+d), pp. 259–260.
    Encoding::new(b"10BBB110", 4, States::single(23)),
];

/// Enables this completed group in the dispatcher (decision-13 rule 4).
pub(super) const LANDED: bool = true;

/// Executes only the CB/INDEX_CB entries above (decision-13 rule 4).
/// The dispatcher has consumed all stream bytes, including indexed d/op.
pub(super) fn execute<S: Bus>(
    cpu: &mut Z80,
    system: &mut S,
    _instruction: &Instruction,
    decoded: &Decoded,
) -> Path {
    let registers = cpu.registers_mut();
    // UM BIT pp. 247/249, SET pp. 255/257, RES p. 259; TM Table
    // 7.0-8 p. 51: indexed memory uses IX/IY plus signed two's-complement
    // d. DERIVED wrap from the 16-bit index width (UM Registers p. 3).
    let address = if decoded.table == Table::IndexCb {
        match (decoded.index, decoded.displacement) {
            (Some(index), Some(displacement)) => {
                let base = match index {
                    IndexRegister::Ix => registers.ix,
                    IndexRegister::Iy => registers.iy,
                };
                base.wrapping_add_signed(i16::from(displacement as i8))
            }
            // Unreachable through the dispatcher: INDEX_CB always has d
            // and an index selection. No new failure path at this seam.
            _ => registers.hl(),
        }
    } else {
        registers.hl()
    };
    // Register codes: UM pp. 243/251/260; HL memory: pp. 245/253/259.
    // The documented indexed keys also have low bits 110 (pp. 247–260).
    let code = decoded.opcode & 7;
    let operand = match code {
        0 => registers.b,
        1 => registers.c,
        2 => registers.d,
        3 => registers.e,
        4 => registers.h,
        5 => registers.l,
        6 => system.read(address, MemoryAccess::Data),
        _ => registers.a,
    };
    let bit = 1u8 << ((decoded.opcode >> 3) & 7);
    let value = match decoded.opcode >> 6 {
        // UM BIT pp. 243–250; TM p. 51: Z is the complement of bit b,
        // H set, N reset, C unaffected. S/P/V are unknown: preserve by
        // library choice, compare hardware under 53h (decision-14 rule 6).
        // PROVENANCE §1 resolves pp. 246/250's second H label as N.
        1 => {
            registers.flags.zero = operand & bit == 0;
            registers.flags.half_carry = true;
            registers.flags.subtract = false;
            return Path::Untaken;
        }
        // UM RES pp. 259–260 and SET pp. 251–258; TM p. 51:
        // change only the selected bit; no flags are affected.
        2 => operand & !bit,
        _ => operand | bit,
    };
    match code {
        0 => registers.b = value,
        1 => registers.c = value,
        2 => registers.d = value,
        3 => registers.e = value,
        4 => registers.h = value,
        5 => registers.l = value,
        // Read/modify/write memory, including an already-set/reset bit
        // (UM SET pp. 253/255/257, RES pp. 259–260; decision-15 rule 5).
        6 => system.write(address, value, MemoryAccess::Data),
        _ => registers.a = value,
    }
    Path::Untaken
}
