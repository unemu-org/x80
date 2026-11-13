//! 16-Bit Load Group (decision-13 rules 2–5).
//!
//! Primary: Zilog UM008011-0816, rev. 11 (August 2016), pp. 98–122.
//! Entry citations below use printed pages; PDF = printed + 14.
//! Independent cross-read: 03-0029-01 (1977), Table 7.0-2, p. 45
//! (PDF = printed + 6). Whole-instruction lengths and T-state totals include
//! every prefix, operand and displacement (decision-13 rules 3–5).
//! All loads and PUSH preserve flags; POP AF loads the six stored flags.
//! AF uses decision-14 rule 5's library image, with unused F bits 5/3 zero.
//! Access kinds and operand fetching follow decision-15 rule 5. Address,
//! PC and SP wraps are derived from 16-bit widths (UM CPU Register p. 2).

use super::cpu::{Decoded, IndexRegister, Table};
use super::opcodes::{Encoding, Instruction, States};
use super::{Registers, Z80};
use crate::decode::Path;
use crate::{Bus, MemoryAccess};

pub(super) const MAIN: &[Encoding] = &[
    // UM LD dd,nn, p. 99.
    Encoding::new(b"00PP0001", 3, States::single(10)),
    // UM LD HL,(nn), p. 102.
    Encoding::new(b"00101010", 3, States::single(16)),
    // UM LD (nn),HL, p. 107.
    Encoding::new(b"00100010", 3, States::single(16)),
    // UM LD SP,HL, p. 112.
    Encoding::new(b"11111001", 1, States::single(6)),
    // UM PUSH qq, pp. 115–116 (compact misprints: PROVENANCE §1).
    Encoding::new(b"11QQ0101", 1, States::single(11)),
    // UM POP qq, pp. 119–120 (compact misprint: PROVENANCE §1).
    Encoding::new(b"11QQ0001", 1, States::single(10)),
];

pub(super) const ED: &[Encoding] = &[
    // UM LD dd,(nn), pp. 103–104; includes HL (PROVENANCE §1).
    Encoding::new(b"01PP1011", 4, States::single(20)),
    // UM LD (nn),dd, pp. 108–109; includes HL (PROVENANCE §1).
    Encoding::new(b"01PP0011", 4, States::single(20)),
];

pub(super) const INDEX: &[Encoding] = &[
    // UM LD IX/IY,nn, pp. 100–101.
    Encoding::new(b"00100001", 4, States::single(14)),
    // UM LD IX/IY,(nn), pp. 105–106.
    Encoding::new(b"00101010", 4, States::single(20)),
    // UM LD (nn),IX/IY, pp. 110–111.
    Encoding::new(b"00100010", 4, States::single(20)),
    // UM LD SP,IX/IY, pp. 113–114.
    Encoding::new(b"11111001", 2, States::single(10)),
    // UM PUSH IX/IY, pp. 117–118 (compact misprints: PROVENANCE §1).
    Encoding::new(b"11100101", 2, States::single(15)),
    // UM POP IX/IY, pp. 121–122.
    Encoding::new(b"11100001", 2, States::single(14)),
];

/// PC is after prefix/key bytes; operand reads advance it through
/// Z80::fetch_operand (decision-13 rule 4). Every form has a fixed cost.
pub(super) fn execute<S: Bus>(
    cpu: &mut Z80,
    system: &mut S,
    _instruction: &Instruction,
    decoded: &Decoded,
) -> Path {
    let pair = target(decoded);
    match (decoded.table, decoded.opcode) {
        // UM pp. 99–101: first operand is low, then high; no flags.
        (Table::Main, 0x01 | 0x11 | 0x21 | 0x31) | (Table::Index, 0x21) => {
            let value = fetch_word(cpu, system);
            pair.set(cpu.registers_mut(), value);
        }
        // UM pp. 102–106: low at nn, high at nn+1; ED includes HL.
        (Table::Main | Table::Index, 0x2A) | (Table::Ed, 0x4B | 0x5B | 0x6B | 0x7B) => {
            let address = fetch_word(cpu, system);
            let low = system.read(address, MemoryAccess::Data);
            let high = system.read(address.wrapping_add(1), MemoryAccess::Data);
            pair.set(cpu.registers_mut(), u16::from_le_bytes([low, high]));
        }
        // UM pp. 107–111: store low then high, including ED 63's HL.
        (Table::Main | Table::Index, 0x22) | (Table::Ed, 0x43 | 0x53 | 0x63 | 0x73) => {
            let address = fetch_word(cpu, system);
            let [low, high] = pair.get(cpu.registers()).to_le_bytes();
            system.write(address, low, MemoryAccess::Data);
            system.write(address.wrapping_add(1), high, MemoryAccess::Data);
        }
        // UM pp. 112–114: register transfer only; no stack access.
        (Table::Main | Table::Index, 0xF9) => {
            cpu.registers_mut().sp = pair.get(cpu.registers());
        }
        // UM pp. 115–118 explicitly order decrement/high, decrement/low.
        (Table::Main, 0xC5 | 0xD5 | 0xE5 | 0xF5) | (Table::Index, 0xE5) => {
            let [low, high] = pair.get(cpu.registers()).to_le_bytes();
            let registers = cpu.registers_mut();
            registers.sp = registers.sp.wrapping_sub(1);
            system.write(registers.sp, high, MemoryAccess::Stack);
            registers.sp = registers.sp.wrapping_sub(1);
            system.write(registers.sp, low, MemoryAccess::Stack);
        }
        // UM pp. 119–122 descriptions: low, increment, high, increment.
        // POP IY p. 122's operation-line typo is resolved in PROVENANCE §1.
        (Table::Main, 0xC1 | 0xD1 | 0xE1 | 0xF1) | (Table::Index, 0xE1) => {
            let registers = cpu.registers_mut();
            let low = system.read(registers.sp, MemoryAccess::Stack);
            registers.sp = registers.sp.wrapping_add(1);
            let high = system.read(registers.sp, MemoryAccess::Stack);
            registers.sp = registers.sp.wrapping_add(1);
            pair.set(registers, u16::from_le_bytes([low, high]));
        }
        // The group-owned encoding lists admit only the arms above.
        _ => {}
    }
    Path::Untaken
}

// UM pp. 99–111: every immediate value/address is low-byte first.
fn fetch_word<S: Bus>(cpu: &mut Z80, system: &mut S) -> u16 {
    let low = cpu.fetch_operand(system);
    let high = cpu.fetch_operand(system);
    u16::from_le_bytes([low, high])
}

#[derive(Clone, Copy)]
enum Pair {
    Bc,
    De,
    Hl,
    Sp,
    Af,
    Ix,
    Iy,
}

// Pair fields: UM pp. 99, 103, 108 (dd), 115, 119 (qq).
// Index selection: corresponding entries pp. 100–122. TM p. 45 agrees.
fn target(decoded: &Decoded) -> Pair {
    if let Some(index) = decoded.index {
        return match index {
            IndexRegister::Ix => Pair::Ix,
            IndexRegister::Iy => Pair::Iy,
        };
    }
    if decoded.opcode == 0xF9 {
        return Pair::Hl;
    }
    match (decoded.opcode >> 4) & 3 {
        0 => Pair::Bc,
        1 => Pair::De,
        2 => Pair::Hl,
        // Only main PUSH/POP use qq=11 (AF); dd=11 is SP.
        _ if decoded.table == Table::Main && decoded.opcode & 0xC0 == 0xC0 => Pair::Af,
        _ => Pair::Sp,
    }
}

impl Pair {
    fn get(self, registers: &Registers) -> u16 {
        match self {
            Self::Bc => registers.bc(),
            Self::De => registers.de(),
            Self::Hl => registers.hl(),
            Self::Sp => registers.sp,
            // Library image (decision-14 rule 5), not physical unused bits.
            Self::Af => registers.af(),
            Self::Ix => registers.ix,
            Self::Iy => registers.iy,
        }
    }

    fn set(self, registers: &mut Registers, value: u16) {
        match self {
            Self::Bc => registers.set_bc(value),
            Self::De => registers.set_de(value),
            Self::Hl => registers.set_hl(value),
            Self::Sp => registers.sp = value,
            Self::Af => registers.set_af(value),
            Self::Ix => registers.ix = value,
            Self::Iy => registers.iy = value,
        }
    }
}
