//! 8-Bit Load Group (decision-13 rule 2).
//!
//! Primary: Zilog UM008011-0816, rev. 11 (August 2016), pp. 70–97.
//! Entry citations below use printed pages; PDF = printed + 14.
//! Independent cross-read: 03-0029-01 (1977), Table 7.0-1, p. 44
//! (PDF = printed + 6). Whole-instruction lengths and T-state totals include
//! effective prefixes, operands and displacements (decision-13 rules 3–5).
//! Extra ignored/repeated selectors are accounted in cpu.rs (decision-20 §2).

use super::cpu::{Decoded, IndexRegister, Table};
use super::opcodes::{Encoding, Instruction, States};
use super::{Registers, Z80};
use crate::decode::Path;
use crate::{Bus, MemoryAccess};

pub(super) const MAIN: &[Encoding] = &[
    // UM LD r,r', p. 71.
    Encoding::new(b"01RRRSSS", 1, States::single(4)),
    // UM LD r,n, p. 72.
    Encoding::new(b"00RRR110", 2, States::single(7)),
    // UM LD r,(HL), p. 74.
    Encoding::new(b"01RRR110", 1, States::single(7)),
    // UM LD (HL),r, p. 79.
    Encoding::new(b"01110SSS", 1, States::single(7)),
    // UM LD (HL),n, p. 85.
    Encoding::new(b"00110110", 2, States::single(10)),
    // UM LD A,(BC), p. 88.
    Encoding::new(b"00001010", 1, States::single(7)),
    // UM LD A,(DE), p. 89.
    Encoding::new(b"00011010", 1, States::single(7)),
    // UM LD A,(nn), p. 90.
    Encoding::new(b"00111010", 3, States::single(13)),
    // UM LD (BC),A, p. 91.
    Encoding::new(b"00000010", 1, States::single(7)),
    // UM LD (DE),A, p. 92.
    Encoding::new(b"00010010", 1, States::single(7)),
    // UM LD (nn),A, p. 93.
    Encoding::new(b"00110010", 3, States::single(13)),
];

pub(super) const INDEX: &[Encoding] = &[
    // H operation: decision-20 §2; Young v0.91 §3.2 pp. 11–12 /
    // §8.1 p. 26; Rak v1.2a .ld_x_n/.ld_x_x. New costs L.
    Encoding::new(b"00100110", 3, States::single(11)),
    Encoding::new(b"00101110", 3, States::single(11)),
    // Any register destination from an index half (R excludes memory).
    Encoding::new(b"01RRR100", 2, States::single(8)),
    Encoding::new(b"01RRR101", 2, States::single(8)),
    // Half destinations from B/C/D/E/A; half sources above include all
    // cross/self loads. Keep patterns disjoint and memory forms separate.
    Encoding::new(b"01100000", 2, States::single(8)),
    Encoding::new(b"01100001", 2, States::single(8)),
    Encoding::new(b"01100010", 2, States::single(8)),
    Encoding::new(b"01100011", 2, States::single(8)),
    Encoding::new(b"01100111", 2, States::single(8)),
    Encoding::new(b"01101000", 2, States::single(8)),
    Encoding::new(b"01101001", 2, States::single(8)),
    Encoding::new(b"01101010", 2, States::single(8)),
    Encoding::new(b"01101011", 2, States::single(8)),
    Encoding::new(b"01101111", 2, States::single(8)),
    // UM LD r,(IX/IY+d), pp. 75, 77.
    Encoding::new(b"01RRR110", 3, States::single(19)),
    // UM LD (IX/IY+d),r, pp. 81, 83.
    Encoding::new(b"01110SSS", 3, States::single(19)),
    // UM LD (IX/IY+d),n, pp. 86–87.
    Encoding::new(b"00110110", 4, States::single(19)),
];

pub(super) const ED: &[Encoding] = &[
    // UM LD A,I, p. 94.
    Encoding::new(b"01010111", 2, States::single(9)),
    // UM LD A,R, p. 95.
    Encoding::new(b"01011111", 2, States::single(9)),
    // UM LD I,A, p. 96.
    Encoding::new(b"01000111", 2, States::single(9)),
    // UM LD R,A, p. 97.
    Encoding::new(b"01001111", 2, States::single(9)),
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
    let opcode = decoded.opcode;
    if decoded.table == Table::Ed {
        // UM pp. 94–97; TM Table 7.0-1 p. 44. Decision-15 rule 6:
        // both fetch increments precede execution, including LD R,A.
        match opcode {
            0x47 => cpu.registers_mut().i = cpu.registers().a,
            0x4F => cpu.registers_mut().r = cpu.registers().a,
            0x57 | 0x5F => {
                let value = if opcode == 0x57 {
                    cpu.registers().i
                } else {
                    cpu.registers().r
                };
                let iff2 = cpu.latches().iff2;
                let registers = cpu.registers_mut();
                registers.a = value;
                registers.flags.sign = value & 0x80 != 0;
                registers.flags.zero = value == 0;
                registers.flags.half_carry = false;
                registers.flags.parity_overflow = iff2;
                registers.flags.subtract = false;
                // Carry is unchanged. Decision-15 rule 10 adopts this
                // unverified boundary interpretation of the interrupt note.
                cpu.latches_mut().parity_note_pending = true;
            }
            _ => {}
        }
        return Path::Untaken;
    }

    // UM pp. 74–87; TM p. 44. INDEX d is fetched before n and is a
    // two's-complement displacement. Wrapping the sum is derived from
    // the 16-bit index width (UM Registers p. 3). Decision-15 rule 5
    // maps instruction operands to OperandFetch and memory data to Data.
    let address = if decoded.table == Table::Index
        && (opcode == 0x36
            || (opcode & 0xC0 == 0x40 && (opcode & 7 == 6 || (opcode >> 3) & 7 == 6)))
    {
        let displacement = i16::from(cpu.fetch_operand(system) as i8);
        let base = match decoded.index {
            Some(IndexRegister::Ix) => cpu.registers().ix,
            Some(IndexRegister::Iy) => cpu.registers().iy,
            None => cpu.registers().hl(),
        };
        base.wrapping_add_signed(displacement)
    } else {
        cpu.registers().hl()
    };

    // A/H exceptions: decision-20 §2; UM pp. 78–85; Young v0.91
    // §3.2 pp. 11–12; Rak v1.2a .ld_r_xyd/.ld_xyd_r. Beside memory,
    // H/L remain ordinary; only register-only forms use selected halves.
    let index = if opcode & 0xC0 == 0x40 && (opcode & 7 == 6 || (opcode >> 3) & 7 == 6) {
        None
    } else {
        decoded.index
    };
    // The map admits only the listed load encodings (decision-14 rule 1).
    // UM pp. 71–87; TM p. 44. Capture source/address before destination
    // writes, including ordinary H/L beside indexed memory.
    if opcode & 0xC0 == 0x40 {
        let source = opcode & 7;
        let destination = (opcode >> 3) & 7;
        let value = if source == 6 {
            system.read(address, MemoryAccess::Data)
        } else {
            read_register(cpu.registers(), source, index)
        };
        if destination == 6 {
            system.write(address, value, MemoryAccess::Data);
        } else {
            write_register(cpu.registers_mut(), destination, value, index);
        }
    } else if opcode & 0xC7 == 0x06 {
        let value = cpu.fetch_operand(system);
        let destination = (opcode >> 3) & 7;
        if destination == 6 {
            system.write(address, value, MemoryAccess::Data);
        } else {
            write_register(cpu.registers_mut(), destination, value, index);
        }
    } else {
        // UM pp. 88–93; TM p. 44. Absolute addresses have low n first;
        // both operand reads precede the data access (decision-15 rule 5).
        let address = match opcode {
            0x02 | 0x0A => cpu.registers().bc(),
            0x12 | 0x1A => cpu.registers().de(),
            0x32 | 0x3A => {
                let low = cpu.fetch_operand(system);
                let high = cpu.fetch_operand(system);
                u16::from_le_bytes([low, high])
            }
            _ => address,
        };
        if opcode & 0x08 != 0 {
            cpu.registers_mut().a = system.read(address, MemoryAccess::Data);
        } else {
            system.write(address, cpu.registers().a, MemoryAccess::Data);
        }
    }
    // All ordinary loads leave every flag unchanged (UM pp. 71–93).
    // Both States slots agree for this group (decision-13 rule 5).
    Path::Untaken
}

// Register fields: UM p. 71 (also pp. 72,74–83); TM p. 44.
// Only register codes reach these helpers: memory code 110 is handled above.
// H index halves: decision-20 §2; Young v0.91 §3.2 pp. 11–12 /
// §8.1 p. 26; Rak v1.2a .ld_x_x/.ld_x_n. Callers exclude memory forms.
pub(super) fn read_register(registers: &Registers, code: u8, index: Option<IndexRegister>) -> u8 {
    if let Some(index) = index {
        let value = match index {
            IndexRegister::Ix => registers.ix,
            IndexRegister::Iy => registers.iy,
        };
        if code == 4 {
            return (value >> 8) as u8;
        }
        if code == 5 {
            return value as u8;
        }
    }
    match code {
        0 => registers.b,
        1 => registers.c,
        2 => registers.d,
        3 => registers.e,
        4 => registers.h,
        5 => registers.l,
        _ => registers.a,
    }
}

pub(super) fn write_register(
    registers: &mut Registers,
    code: u8,
    value: u8,
    index: Option<IndexRegister>,
) {
    if let Some(index) = index {
        let word = match index {
            IndexRegister::Ix => &mut registers.ix,
            IndexRegister::Iy => &mut registers.iy,
        };
        if code == 4 {
            *word = (*word & 0x00FF) | (u16::from(value) << 8);
            return;
        }
        if code == 5 {
            *word = (*word & 0xFF00) | u16::from(value);
            return;
        }
    }
    match code {
        0 => registers.b = value,
        1 => registers.c = value,
        2 => registers.d = value,
        3 => registers.e = value,
        4 => registers.h = value,
        5 => registers.l = value,
        _ => registers.a = value,
    }
}
