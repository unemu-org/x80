//! Jump Group: encodings and execution (decision-13 rules 2–5).
//!
//! Primary: Zilog UM008011-0816, rev. 11 (August 2016), pp. 261–279.
//! Entry citations below use printed pages; PDF = printed + 14.
//! Independent cross-read: 03-0029-01 (1977), Table 7.0-9, p. 52
//! (PDF = printed + 6). Whole-instruction lengths and T-state totals include
//! every prefix, operand and displacement (decision-13 rules 3–5).

use super::cpu::{Decoded, IndexRegister};
use super::opcodes::{Encoding, Instruction, States};
use super::{Flags, Z80};
use crate::decode::Path;
use crate::Bus;

pub(super) const MAIN: &[Encoding] = &[
    // UM JP nn, p. 262.
    Encoding::new(b"11000011", 3, States::single(10)),
    // UM JP cc,nn, pp. 263–264; both paths are 10.
    Encoding::new(b"11CCC010", 3, States::single(10)),
    // UM JR e, p. 265.
    Encoding::new(b"00011000", 2, States::single(12)),
    // UM JR C,e, p. 267.
    Encoding::new(
        b"00111000",
        2,
        States {
            untaken: 7,
            taken: 12,
        },
    ),
    // UM JR NC,e, p. 269.
    Encoding::new(
        b"00110000",
        2,
        States {
            untaken: 7,
            taken: 12,
        },
    ),
    // UM JR Z,e, p. 271.
    Encoding::new(
        b"00101000",
        2,
        States {
            untaken: 7,
            taken: 12,
        },
    ),
    // UM JR NZ,e, p. 273.
    Encoding::new(
        b"00100000",
        2,
        States {
            untaken: 7,
            taken: 12,
        },
    ),
    // UM JP (HL), p. 275.
    Encoding::new(b"11101001", 1, States::single(4)),
    // UM DJNZ e, p. 278.
    Encoding::new(
        b"00010000",
        2,
        States {
            untaken: 8,
            taken: 13,
        },
    ),
];

pub(super) const INDEX: &[Encoding] = &[
    // UM JP (IX/IY), pp. 276–277.
    Encoding::new(b"11101001", 2, States::single(8)),
];

/// PC starts after the prefix/key bytes (decision-13 rule 4). Operand reads
/// use decision-15 rule 5's OperandFetch mapping; flags are unaffected for
/// every form (UM pp. 262, 264–265, 267, 269, 271, 273, 275–277, 279;
/// TM Table 7.0-9 p. 52). Path selects the map's complete cost (rule 5).
pub(super) fn execute<S: Bus>(
    cpu: &mut Z80,
    system: &mut S,
    _instruction: &Instruction,
    decoded: &Decoded,
) -> Path {
    match decoded.opcode {
        // JP nn / JP cc,nn: UM pp. 262–264; TM p. 52. The low byte
        // precedes the high. Both paths have 3 cycles / 10 (4,3,3):
        // derive both operand reads even when cc is false.
        opcode if opcode == 0xC3 || opcode & 0xC7 == 0xC2 => {
            let low = cpu.fetch_operand(system);
            let high = cpu.fetch_operand(system);
            // L: decision-20 §5; MEMPTR (©2006), JP: even untaken.
            cpu.latches_mut().wz = u16::from_le_bytes([low, high]);
            if opcode == 0xC3 || condition(cpu.registers().flags, (opcode >> 3) & 7) {
                cpu.registers_mut().pc = u16::from_le_bytes([low, high]);
                Path::Taken
            } else {
                Path::Untaken
            }
        }
        // JP (HL/IX/IY): UM pp. 275–277; TM p. 52. Parentheses denote
        // a register-held target here, without a data-memory read.
        0xE9 => {
            let registers = cpu.registers_mut();
            registers.pc = match decoded.index {
                Some(IndexRegister::Ix) => registers.ix,
                Some(IndexRegister::Iy) => registers.iy,
                None => registers.hl(),
            };
            Path::Taken
        }
        // JR / JR cc / DJNZ: UM pp. 265, 267, 269, 271, 273, 278;
        // TM p. 52. The displacement is read on either path.
        0x18 | 0x20 | 0x28 | 0x30 | 0x38 | 0x10 => {
            let displacement = cpu.fetch_operand(system) as i8;
            let taken = match decoded.opcode {
                0x18 => true,
                0x10 => {
                    // Test the decremented B, independently of F.Z.
                    // Eight-bit wrap is derived (UM Registers pp. 2–3).
                    let registers = cpu.registers_mut();
                    registers.b = registers.b.wrapping_sub(1);
                    registers.b != 0
                }
                // JR's two-bit cc has NZ/Z/NC/C in the same order as
                // JP's first four conditions (UM entries cited above).
                opcode => condition(cpu.registers().flags, (opcode >> 3) & 3),
            };
            if taken {
                // The entry's e is measured from the opcode (-126..129),
                // with encoded e-2 and a "twice incremented PC". Thus
                // add the signed encoded byte to the post-operand PC.
                // Sixteen-bit wrap is derived from PC width (UM p. 2).
                let registers = cpu.registers_mut();
                registers.pc = registers.pc.wrapping_add_signed(i16::from(displacement));
                // L: decision-20 §5; MEMPTR (©2006), JR/DJNZ.
                cpu.latches_mut().wz = cpu.registers().pc;
                Path::Taken
            } else {
                Path::Untaken
            }
        }
        // The final opcode map dispatches only the forms above.
        _ => Path::Untaken,
    }
}

/// JP cc's field meanings: UM pp. 263–264; TM Table 7.0-9 p. 52.
/// This model-owned predicate is private to this instruction group
/// (decision-13 rule 7); all eight flag bits remain unchanged (decision-20 §4).
fn condition(flags: Flags, code: u8) -> bool {
    match code {
        0 => !flags.zero,
        1 => flags.zero,
        2 => !flags.carry,
        3 => flags.carry,
        4 => !flags.parity_overflow,
        5 => flags.parity_overflow,
        6 => !flags.sign,
        _ => flags.sign,
    }
}
