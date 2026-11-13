//! Rotate and Shift Group: encodings and execution (decision-13 rules 2–4).
//!
//! Primary: Zilog UM008011-0816, rev. 11 (August 2016), pp. 204–241.
//! Entry citations below use printed pages; PDF = printed + 14.
//! Independent cross-read: 03-0029-01 (1977), Table 7.0-7, p. 50
//! (PDF = printed + 6). Whole-instruction lengths and T-state totals include
//! every prefix, operand and displacement (decision-13 rules 3–5).
//! Execution witnesses: tests/z80_rotate_shift.rs. Access kinds and refresh
//! follow decision-20 §§2–3 (L mapping). Only six flags are stored; bits 5/3
//! await decision-20 §4 / x80-mn3.15, retaining current flag storage.

use super::cpu::{Decoded, IndexRegister, Table};
use super::opcodes::{Encoding, Instruction, States};
use super::{Flags, Registers, Z80};
use crate::decode::Path;
use crate::{Bus, MemoryAccess};

// Entry bit-diagram slips on pp. 225, 228 and RRCA's worked example
// on p. 210: see PROVENANCE.md §1. Follow the agreeing bit routing.

pub(super) const MAIN: &[Encoding] = &[
    // UM RLCA, p. 205.
    Encoding::new(b"00000111", 1, States::single(4)),
    // UM RLA, p. 207.
    Encoding::new(b"00010111", 1, States::single(4)),
    // UM RRCA, p. 209.
    Encoding::new(b"00001111", 1, States::single(4)),
    // UM RRA, p. 211; agrees with TM Table 7.0-7, p. 50.
    Encoding::new(b"00011111", 1, States::single(4)),
];

pub(super) const CB: &[Encoding] = &[
    // UM RLC r, p. 213.
    Encoding::new(b"00000RRR", 2, States::single(8)),
    // UM RLC (HL), p. 215.
    Encoding::new(b"00000110", 2, States::single(15)),
    // UM RRC r, pp. 224–226.
    Encoding::new(b"00001RRR", 2, States::single(8)),
    // UM RRC (HL), pp. 224–226.
    Encoding::new(b"00001110", 2, States::single(15)),
    // UM RL r, pp. 221–222.
    Encoding::new(b"00010RRR", 2, States::single(8)),
    // UM RL (HL), pp. 221–222.
    Encoding::new(b"00010110", 2, States::single(15)),
    // UM RR r, pp. 227–229.
    Encoding::new(b"00011RRR", 2, States::single(8)),
    // UM RR (HL), pp. 227–229.
    Encoding::new(b"00011110", 2, States::single(15)),
    // UM SLA r, pp. 230–231.
    Encoding::new(b"00100RRR", 2, States::single(8)),
    // UM SLA (HL), pp. 230–231.
    Encoding::new(b"00100110", 2, States::single(15)),
    // UM SRA r, pp. 233–234.
    Encoding::new(b"00101RRR", 2, States::single(8)),
    // UM SRA (HL), pp. 233–234.
    Encoding::new(b"00101110", 2, States::single(15)),
    // H SLL: decision-20 §3; Young v0.91 (2005-09-18) §3.1 p. 11,
    // §8.7 p. 32; Rak z80test v1.2a .slia_a/.slia_r, testtable 81–100.
    // L 2/8 and 2/15: neighbouring shift-cost extension, no timing key.
    Encoding::new(b"00110RRR", 2, States::single(8)),
    Encoding::new(b"00110110", 2, States::single(15)),
    // UM SRL r, pp. 236–237.
    Encoding::new(b"00111RRR", 2, States::single(8)),
    // UM SRL (HL), pp. 236–237.
    Encoding::new(b"00111110", 2, States::single(15)),
];

// A canonical memory forms: UM008011-0816 rev. 11 pp. 217–241;
// TM 03-0029-01 (1977) Table 7.0-7 p. 50.
// H copies/SLL: decision-20 §3; Young v0.91 §§3.5–3.6 pp. 13–14,
// §8.7 p. 32; Rak v1.2a .sro_xyd_r 671–676. L new costs: Young
// §8.7 p. 32, without independent timing capture. D includes memory 110.
pub(super) const INDEX_CB: &[Encoding] = &[Encoding::new(b"00SSSDDD", 4, States::single(23))];

pub(super) const ED: &[Encoding] = &[
    // UM RLD, p. 238.
    Encoding::new(b"01101111", 2, States::single(18)),
    // UM RRD, p. 240.
    Encoding::new(b"01100111", 2, States::single(18)),
];

/// Final executor seam (decision-13 rule 4). PC is after prefix/key bytes;
/// read remaining operands through Z80::fetch_operand. INDEX d is executor-
/// owned; INDEX_CB d is in Decoded. Return the complete instruction's path.
/// Repeating blocks rewind PC by two and return Path::Taken (rule 11).
pub(super) fn execute<S: Bus>(
    cpu: &mut Z80,
    system: &mut S,
    _instruction: &Instruction,
    decoded: &Decoded,
) -> Path {
    let registers = cpu.registers_mut();
    match decoded.table {
        Table::Main => {
            // UM RLCA/RLA/RRCA/RRA pp. 205, 207, 209, 211;
            // TM Table 7.0-7 p. 50: S/Z/P-V unaffected, H=N=0.
            let (value, carry) = rotate_shift(decoded.opcode, registers.a, registers.flags.carry);
            registers.a = value;
            registers.flags.half_carry = false;
            registers.flags.subtract = false;
            registers.flags.carry = carry;
        }
        Table::Cb => {
            let address = if decoded.opcode & 7 == 6 {
                Some(registers.hl())
            } else {
                None
            };
            execute_cb(registers, system, decoded.opcode, address);
        }
        Table::IndexCb => {
            // UM pp. 217, 219 and analogous forms pp. 221–237;
            // TM p. 50: d is signed two's complement. The dispatcher has
            // already fetched d/op. Wrap is derived from 16-bit addresses.
            if let (Some(index), Some(displacement)) = (decoded.index, decoded.displacement) {
                let base = match index {
                    IndexRegister::Ix => registers.ix,
                    IndexRegister::Iy => registers.iy,
                };
                let address = base.wrapping_add_signed(i16::from(displacement as i8));
                execute_cb(registers, system, decoded.opcode, Some(address));
            }
        }
        Table::Ed => {
            // UM RLD pp. 238–239, RRD pp. 240–241; TM p. 50.
            // Rotate the three low nibbles, preserving A's high nibble.
            // S/Z/P-V describe the resulting A; C is unaffected, H=N=0.
            let address = registers.hl();
            let memory = system.read(address, MemoryAccess::Data);
            let a = registers.a;
            let (value, memory) = if decoded.opcode == 0x6F {
                ((a & 0xF0) | (memory >> 4), (memory << 4) | (a & 0x0F))
            } else {
                (
                    (a & 0xF0) | (memory & 0x0F),
                    ((a & 0x0F) << 4) | (memory >> 4),
                )
            };
            system.write(address, memory, MemoryAccess::Data);
            registers.a = value;
            registers.flags = result_flags(value, registers.flags.carry);
        }
        // This group has no terminals in the ordinary DD/FD table.
        Table::Index => {}
    }
    Path::Untaken
}

/// CB register fields and read/modify/write memory: UM pp. 213–237;
/// TM Table 7.0-7 p. 50. H indexed copies: decision-20 §3, Young v0.91
/// §§3.5–3.6 pp. 13–14; Rak v1.2a .sro_xyd_r 671–676. L temporal/API
/// rule: write once and copy the computed byte, even if the device ignores
/// the write; do not reread memory. Ordinary H/L are always the destinations.
fn execute_cb<S: Bus>(registers: &mut Registers, system: &mut S, opcode: u8, address: Option<u16>) {
    let operand = if let Some(address) = address {
        system.read(address, MemoryAccess::Data)
    } else {
        match opcode & 7 {
            0 => registers.b,
            1 => registers.c,
            2 => registers.d,
            3 => registers.e,
            4 => registers.h,
            5 => registers.l,
            _ => registers.a,
        }
    };
    let (value, carry) = rotate_shift(opcode, operand, registers.flags.carry);
    if let Some(address) = address {
        system.write(address, value, MemoryAccess::Data);
    }
    if opcode & 7 != 6 {
        match opcode & 7 {
            0 => registers.b = value,
            1 => registers.c = value,
            2 => registers.d = value,
            3 => registers.e = value,
            4 => registers.h = value,
            5 => registers.l = value,
            _ => registers.a = value,
        }
    }
    registers.flags = result_flags(value, carry);
}

/// Bit routing: UM RLC pp. 213–220, RL pp. 221–223, RRC pp. 224–226,
/// RR pp. 227–229, SLA pp. 230–232, SRA pp. 233–235, SRL pp. 236–237;
/// TM Table 7.0-7 p. 50. Main accumulator forms share the first four
/// selectors (UM pp. 205, 207, 209, 211), but preserve S/Z/P-V separately.
fn rotate_shift(opcode: u8, value: u8, carry: bool) -> (u8, bool) {
    match opcode & 0x38 {
        0x00 => (value.rotate_left(1), value & 0x80 != 0),
        0x08 => (value.rotate_right(1), value & 1 != 0),
        0x10 => ((value << 1) | u8::from(carry), value & 0x80 != 0),
        0x18 => ((value >> 1) | (u8::from(carry) << 7), value & 1 != 0),
        0x20 => (value << 1, value & 0x80 != 0),
        0x28 => ((value >> 1) | (value & 0x80), value & 1 != 0),
        // H SLL: decision-20 §3; Young v0.91 §3.1 p. 11 / §8.7 p. 32;
        // Rak v1.2a .slia_a/.slia_r, testtable 81–100.
        0x30 => ((value << 1) | 1, value & 0x80 != 0),
        _ => (value >> 1, value & 1 != 0),
    }
}

/// UM flag lists pp. 214–215, 217–220, 222, 226, 229, 231, 234, 237–241;
/// TM p. 50: S/Z from result, P-V even parity, H=N=0. For nibble rotates
/// the caller supplies preserved C and the resulting accumulator.
fn result_flags(value: u8, carry: bool) -> Flags {
    Flags {
        sign: value & 0x80 != 0,
        zero: value == 0,
        half_carry: false,
        parity_overflow: value.count_ones() & 1 == 0,
        subtract: false,
        carry,
    }
}
