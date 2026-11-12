//! The 8080 Logical Group: its encodings and its executor.
//!
//! Primary source: Intel 98-153B (September 1975), Ch. 4, "Logical Group",
//! pp. 4-8 to 4-11 (STC is
//! the last entry before "Branch Group" on p. 4-11), checked against the page
//! images. Field letters are those of `Encoding` in `opcodes`.
//!
//! Design: plan §2.2; decision-5 items 3/8; decision-7 (provisional AND AC).
//! Source revisions and hashes are pinned in PROVENANCE.md §1. Standard
//! result flags and subtraction live in `alu`; the unverified AND AC rule
//! is isolated below pending x80-ecy.2.

use super::opcodes::{Encoding, Instruction, Path, States};
use super::{alu, Registers, I8080};
use crate::{Bus, MemoryAccess};

/// The group's encodings, in page order. Each total is the entry's "States:"
/// value on the cited printed page (PDF page = 44 + printed page).
pub(super) const ENCODINGS: &[Encoding] = &[
    // ANA r, p. 4-8.
    Encoding::new(b"10100SSS", 1).with_states(States::single(4)),
    // ANA M, p. 4-8.
    Encoding::new(b"10100110", 1).with_states(States::single(7)),
    // ANI data: opcode and data byte, p. 4-9.
    Encoding::new(b"11100110", 2).with_states(States::single(7)),
    // XRA r, p. 4-9.
    Encoding::new(b"10101SSS", 1).with_states(States::single(4)),
    // XRA M, p. 4-9.
    Encoding::new(b"10101110", 1).with_states(States::single(7)),
    // XRI data, p. 4-9.
    Encoding::new(b"11101110", 2).with_states(States::single(7)),
    // ORA r, p. 4-9.
    Encoding::new(b"10110SSS", 1).with_states(States::single(4)),
    // ORA M, p. 4-9.
    Encoding::new(b"10110110", 1).with_states(States::single(7)),
    // ORI data, p. 4-9.
    Encoding::new(b"11110110", 2).with_states(States::single(7)),
    // CMP r, p. 4-9.
    Encoding::new(b"10111SSS", 1).with_states(States::single(4)),
    // CMP M, p. 4-10.
    Encoding::new(b"10111110", 1).with_states(States::single(7)),
    // CPI data, p. 4-10.
    Encoding::new(b"11111110", 2).with_states(States::single(7)),
    // RLC, p. 4-10.
    Encoding::new(b"00000111", 1).with_states(States::single(4)),
    // RRC, p. 4-10.
    Encoding::new(b"00001111", 1).with_states(States::single(4)),
    // RAL, p. 4-10.
    Encoding::new(b"00010111", 1).with_states(States::single(4)),
    // RAR, p. 4-10.
    Encoding::new(b"00011111", 1).with_states(States::single(4)),
    // CMA, p. 4-10.
    Encoding::new(b"00101111", 1).with_states(States::single(4)),
    // CMC, p. 4-11.
    Encoding::new(b"00111111", 1).with_states(States::single(4)),
    // STC, p. 4-11.
    Encoding::new(b"00110111", 1).with_states(States::single(4)),
];

/// Executes `opcode`, an instruction of this group.
///
/// Returns `None` without effects for an opcode outside this group.
pub(super) fn execute<S: Bus>(
    cpu: &mut I8080,
    system: &mut S,
    opcode: u8,
    instruction: Instruction,
) -> Option<Path> {
    let registers = cpu.registers_mut();
    match opcode {
        // 98-153B Ch. 4 pp. 4-8 to 4-10: register, M and immediate
        // operands. Sample A before replacing it, including source A.
        0xA0..=0xBF | 0xE6 | 0xEE | 0xF6 | 0xFE => {
            let operand = if opcode & 0xC0 == 0xC0 {
                // Ch. 2 p. 2-18: M2 reads byte 2 at PC+1. p. 2-20
                // note 6: memory read. OperandFetch is decision-5 item 3.
                system.read(registers.pc.wrapping_add(1), MemoryAccess::OperandFetch)
            } else {
                read_operand(registers, system, opcode & 7)
            };
            let a = registers.a;
            match opcode & 0x18 {
                // ANA/ANI, Ch. 4 pp. 4-8/4-9: AND, CY cleared.
                // AC follows decision-7's isolated unverified rule below.
                0x00 => {
                    registers.a = a & operand;
                    registers.flags = alu::result_flags(
                        registers.a,
                        provisional_and_auxiliary_carry(a, operand),
                        false,
                    );
                }
                // XRA/XRI, p. 4-9: exclusive OR, CY and AC cleared.
                0x08 => {
                    registers.a = a ^ operand;
                    registers.flags = alu::result_flags(registers.a, false, false);
                }
                // ORA/ORI, p. 4-9: inclusive OR, CY and AC cleared.
                0x10 => {
                    registers.a = a | operand;
                    registers.flags = alu::result_flags(registers.a, false, false);
                }
                // CMP/CPI, pp. 4-9/4-10: subtraction flags without changing
                // A; Z iff equal, CY iff A < operand. No borrow input.
                // Second witness: 9800301-04 (May 1981), Ch. 3 p. 3-12.
                _ => registers.flags = alu::subtract(a, operand, false).1,
            }
        }
        // 98-153B Ch. 4 p. 4-10 RLC/RRC: circular rotation, shifted-out
        // bit enters CY as well as A. Only CY changes among flags.
        0x07 => {
            registers.flags.carry = registers.a & 0x80 != 0;
            registers.a = registers.a.rotate_left(1);
        }
        0x0F => {
            registers.flags.carry = registers.a & 1 != 0;
            registers.a = registers.a.rotate_right(1);
        }
        // RAL/RAR, p. 4-10: old CY enters A, the shifted-out bit replaces
        // CY. All other flags are preserved.
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
        // CMA, p. 4-10: complement A, no flag changes.
        0x2F => registers.a = !registers.a,
        // CMC/STC, p. 4-11: complement/set CY; all other flags unchanged.
        0x3F => registers.flags.carry = !registers.flags.carry,
        0x37 => registers.flags.carry = true,
        _ => return None,
    }
    // Sequential bytes: 98-153B Ch. 4 p. 4-2; Ch. 2 pp. 2-16/2-18.
    // Wrap derives from PC's 16-bit width (Ch. 2 p. 2-2). decision-5 item 8
    // exposes completed results here; overlapped writeback (p. 2-20 note 9)
    // does not add another bus read.
    registers.pc = registers.pc.wrapping_add(u16::from(instruction.length));
    Some(Path::Taken)
}

/// UNVERIFIED, provisional until x80-ecy.2 resolves the source disagreement
/// (decision-7). Intel 9800301-04 (May 1981), Ch. 1 p. 1-12: 8080 AND AC
/// is the OR of the operand bit 3s. Intel 98-153B (September 1975), Ch. 4
/// p. 4-8 ANA specifies CY cleared, but p. 4-9 ANI says CY and AC cleared.
/// The accepted interim reading applies the later rule to both ANA and ANI.
const fn provisional_and_auxiliary_carry(a: u8, operand: u8) -> bool {
    (a | operand) & 0x08 != 0
}

/// Register codes: 98-153B Ch. 4 p. 4-3; M uses HL (pp. 4-8 to 4-10).
/// Ch. 2 pp. 2-16/2-18, p. 2-20 note 6: M2 reads memory at HL.
/// Data is decision-5 item 3's access kind for this read.
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
