//! The 8085 Logical Group encodings and executor.
//!
//! Intel 205775-002 (January 1983), §5.6.3, pp. 5-9 to 5-12, checked on page images.
//! Totals below are the 8085 states, not machine cycles or 8080 states.
//! Independent witness: `opcodes/tests.rs`, Table 5-1 p. 5-19 cross-read
//! with Intel 9800301-04 (May 1981), App. A pp. A-1 to A-5.
//! Design: plan §2.2; decision-10 (model-owned facts); decision-5 items 3/8;
//! decision-12 (boundary API). Source hashes match PROVENANCE.md §1.
//! Bus sequences: Table 5-1 p. 5-19 (F or F R), not Table 5-2's opcode
//! list. Ch. 2 §§2.3.1/2.3.2 pp. 2-7 to 2-10 define opcode/memory reads.
//! Independent execution witnesses: tests/i8085_logical.rs. Standard flags
//! and subtraction are this model's `alu`; AND has its own 8085 rule.

use super::opcodes::{Encoding, Instruction, Path, States};
use super::{alu, Registers, I8085};
use crate::{Bus, MemoryAccess};

/// Complete group list, in per-instruction page order.
/// Register-field exclusions live in `opcodes` (§5.2 p. 5-1).
pub(super) const ENCODINGS: &[Encoding] = &[
    // ANA r: 205775-002 p. 5-9.
    Encoding::new(b"10100SSS", 1, States::single(4)),
    // ANA M: 205775-002 p. 5-10.
    Encoding::new(b"10100110", 1, States::single(7)),
    // ANI data: 205775-002 p. 5-10.
    Encoding::new(b"11100110", 2, States::single(7)),
    // XRA r: 205775-002 p. 5-10.
    Encoding::new(b"10101SSS", 1, States::single(4)),
    // XRA M: 205775-002 p. 5-10.
    Encoding::new(b"10101110", 1, States::single(7)),
    // XRI data: 205775-002 p. 5-10.
    Encoding::new(b"11101110", 2, States::single(7)),
    // ORA r: 205775-002 p. 5-10.
    Encoding::new(b"10110SSS", 1, States::single(4)),
    // ORA M: 205775-002 p. 5-11.
    Encoding::new(b"10110110", 1, States::single(7)),
    // ORI data: 205775-002 p. 5-11.
    Encoding::new(b"11110110", 2, States::single(7)),
    // CMP r: 205775-002 p. 5-11.
    Encoding::new(b"10111SSS", 1, States::single(4)),
    // CMP M: 205775-002 p. 5-11.
    Encoding::new(b"10111110", 1, States::single(7)),
    // CPI data: 205775-002 p. 5-11.
    Encoding::new(b"11111110", 2, States::single(7)),
    // RLC: 205775-002 p. 5-11.
    Encoding::new(b"00000111", 1, States::single(4)),
    // RRC: 205775-002 p. 5-12.
    Encoding::new(b"00001111", 1, States::single(4)),
    // RAL: 205775-002 p. 5-12.
    Encoding::new(b"00010111", 1, States::single(4)),
    // RAR: 205775-002 p. 5-12.
    Encoding::new(b"00011111", 1, States::single(4)),
    // CMA: 205775-002 p. 5-12.
    Encoding::new(b"00101111", 1, States::single(4)),
    // CMC: 205775-002 p. 5-12.
    Encoding::new(b"00111111", 1, States::single(4)),
    // STC: 205775-002 p. 5-12.
    Encoding::new(b"00110111", 1, States::single(4)),
];

/// Executes only encodings dispatched to this group (decision-10).
/// A declined byte leaves state and bus untouched (decision-5 item 7).
pub(super) fn execute<S: Bus>(
    cpu: &mut I8085,
    system: &mut S,
    opcode: u8,
    instruction: Instruction,
) -> Option<Path> {
    let registers = cpu.registers_mut();
    match opcode {
        // 205775-002 §5.6.3 pp. 5-9 to 5-11: A is sampled before
        // writeback, including source A. Register field: §5.2 p. 5-1.
        0xA0..=0xBF | 0xE6 | 0xEE | 0xF6 | 0xFE => {
            let operand = if opcode & 0xC0 == 0xC0 {
                // Byte 2, pp. 5-10/5-11; OF then MR at PC+1:
                // Table 5-1 p. 5-19 and §2.3.1 p. 2-7.
                system.read(registers.pc.wrapping_add(1), MemoryAccess::OperandFetch)
            } else {
                read_operand(registers, system, opcode & 7)
            };
            let a = registers.a;
            match opcode & 0x18 {
                // ANA/ANI: pp. 5-9/5-10; second witness 9800301-04
                // pp. 3-6 to 3-8 and p. 1-12 (8085 AC always set).
                0x00 => and_accumulator(registers, operand),
                // XRA/XRI: p. 5-10, CY and AC cleared;
                // 9800301-04 pp. 3-66 to 3-68.
                0x08 => {
                    registers.a = a ^ operand;
                    registers.flags = alu::result_flags(registers.a, false, false);
                }
                // ORA/ORI: pp. 5-10/5-11, CY and AC cleared;
                // 9800301-04 pp. 3-38 to 3-40.
                0x10 => {
                    registers.a = a | operand;
                    registers.flags = alu::result_flags(registers.a, false, false);
                }
                // CMP/CPI: p. 5-11, A unchanged; subtraction flags,
                // Z iff equal, CY iff A < operand, no input borrow.
                // 9800301-04 pp. 3-12/3-13 and 3-16/3-17.
                _ => registers.flags = alu::subtract(a, operand, false).1,
            }
        }
        // RLC: 205775-002 p. 5-11; 9800301-04 p. 3-49.
        // Shifted-out bit enters both A and CY; only CY changes.
        0x07 => {
            registers.flags.carry = registers.a & 0x80 != 0;
            registers.a = registers.a.rotate_left(1);
        }
        // RRC: 205775-002 p. 5-12; 9800301-04 pp. 3-53/3-54.
        0x0F => {
            registers.flags.carry = registers.a & 1 != 0;
            registers.a = registers.a.rotate_right(1);
        }
        // RAL/RAR: 205775-002 p. 5-12; 9800301-04 pp. 3-45/3-46.
        // Old CY enters A, shifted-out bit replaces CY; no other flags.
        0x17 => {
            let carry = registers.flags.carry;
            registers.flags.carry = registers.a & 0x80 != 0;
            registers.a = (registers.a << 1) | u8::from(carry);
        }
        0x1F => {
            let carry = registers.flags.carry;
            registers.flags.carry = registers.a & 1 != 0;
            registers.a = (registers.a >> 1) | (u8::from(carry) << 7);
        }
        // CMA: 205775-002 p. 5-12; 9800301-04 p. 3-11. No flags.
        0x2F => registers.a = !registers.a,
        // CMC/STC: 205775-002 p. 5-12; 9800301-04 pp. 3-12/3-63.
        // Complement/set CY only.
        0x3F => registers.flags.carry = !registers.flags.carry,
        0x37 => registers.flags.carry = true,
        _ => return None,
    }
    // Consecutive instruction bytes: 205775-002 §5.4 p. 5-3.
    // DERIVED wrap from PC's 16-bit width (§2.2.1 p. 2-2).
    // decision-5 item 8 exposes completed writeback at this boundary.
    registers.pc = registers.pc.wrapping_add(u16::from(instruction.length));
    Some(Path::Taken)
}

/// 8085 ANA/ANI: "The CY flag is cleared and AC is set (8085)."
/// Intel 205775-002 (January 1983), §5.6.3 p. 5-10 (ANA M / ANI),
/// also ANA r p. 5-9; Intel 9800301-04 (May 1981), Ch. 1 p. 1-12:
/// 8085 logical AND always sets AC. This model-owned rule (decision-10)
/// differs from the 8080 operand-bit-3 rule printed beside it.
fn and_accumulator(registers: &mut Registers, operand: u8) {
    registers.a &= operand;
    registers.flags = alu::result_flags(registers.a, true, false);
}

/// Register codes: 205775-002 §5.2 p. 5-1; M uses HL, pp. 5-10/5-11.
/// Table 5-1 p. 5-19: F R for M; §§2.3.1/2.3.2 pp. 2-7 to 2-10:
/// opcode fetch precedes memory read. Data kind: decision-5 item 3.
fn read_operand<S: Bus>(registers: &Registers, system: &mut S, code: u8) -> u8 {
    match code {
        0 => registers.b,
        1 => registers.c,
        2 => registers.d,
        3 => registers.e,
        4 => registers.h,
        5 => registers.l,
        6 => system.read(registers.hl(), MemoryAccess::Data),
        _ => registers.a,
    }
}
