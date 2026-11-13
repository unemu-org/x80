//! 16-Bit Arithmetic Group (decision-13 rules 2–5).
//!
//! Primary: Zilog UM008011-0816, rev. 11 (August 2016), pp. 187–203.
//! Entry citations below use printed pages; PDF = printed + 14.
//! Independent cross-read: 03-0029-01 (1977), Table 7.0-6, p. 49
//! (PDF = printed + 6). Whole-instruction lengths and T-state totals include
//! every prefix, operand and displacement (decision-13 rules 3–5).
//! ADD preserves S/Z/P-V, replaces H/C and clears N; ADC/SBC replace all
//! eight flag bits through the model-owned ALU. UM defines H despite TM's unknown
//! marks (PROVENANCE §1; decision-14 rule 6). INC/DEC preserve every flag.
//! Register-only operations make no bus calls after dispatch's opcode fetches
//! (decision-15 rule 5). Pair wraps are derived from the 16-bit register
//! widths (UM CPU Register / Registers pp. 2–3).

use super::alu;
use super::cpu::{Decoded, IndexRegister, Table};
use super::opcodes::{Encoding, Instruction, States};
use super::{Registers, Z80};
use crate::decode::Path;
use crate::Bus;

pub(super) const MAIN: &[Encoding] = &[
    // UM ADD HL,ss, p. 188.
    Encoding::new(b"00PP1001", 1, States::single(11)),
    // UM INC ss, p. 198.
    Encoding::new(b"00PP0011", 1, States::single(6)),
    // UM DEC ss, p. 201 (compact misprint: PROVENANCE §1).
    Encoding::new(b"00PP1011", 1, States::single(6)),
];

pub(super) const ED: &[Encoding] = &[
    // UM ADC HL,ss, p. 190.
    Encoding::new(b"01PP1010", 2, States::single(15)),
    // UM SBC HL,ss, p. 192.
    Encoding::new(b"01PP0010", 2, States::single(15)),
];

pub(super) const INDEX: &[Encoding] = &[
    // UM ADD IX,pp / IY,rr, pp. 194, 196.
    Encoding::new(b"00PP1001", 2, States::single(15)),
    // UM INC IX/IY, pp. 199–200.
    Encoding::new(b"00100011", 2, States::single(10)),
    // UM DEC IX/IY, pp. 202–203.
    Encoding::new(b"00101011", 2, States::single(10)),
];

/// PC is already after all instruction bytes (decision-13 rule 4).
/// Every form is register-only and has a single timing path.
pub(super) fn execute<S: Bus>(
    cpu: &mut Z80,
    _system: &mut S,
    _instruction: &Instruction,
    decoded: &Decoded,
) -> Path {
    let registers = cpu.registers_mut();
    let pair = operand_pair(decoded);
    match (decoded.table, decoded.opcode) {
        // UM ADD HL/IX/IY pp. 188–189, 194–197; TM p. 49.
        // Sample both operands before writing, including self-addition.
        (Table::Main | Table::Index, 0x09 | 0x19 | 0x29 | 0x39) => {
            let destination = accumulator_pair(decoded.index);
            let (value, flags) = alu::add16(
                destination.get(registers),
                pair.get(registers),
                registers.flags,
            );
            destination.set(registers, value);
            registers.flags = flags;
        }
        // UM ADC HL,ss pp. 190–191; SBC HL,ss pp. 192–193; TM p. 49.
        // These use HL even when ss=10, and consume the old C as carry/borrow.
        (Table::Ed, 0x4A | 0x5A | 0x6A | 0x7A) => {
            let (value, flags) =
                alu::adc16(registers.hl(), pair.get(registers), registers.flags.carry);
            registers.set_hl(value);
            registers.flags = flags;
        }
        (Table::Ed, 0x42 | 0x52 | 0x62 | 0x72) => {
            let (value, flags) =
                alu::sbc16(registers.hl(), pair.get(registers), registers.flags.carry);
            registers.set_hl(value);
            registers.flags = flags;
        }
        // UM INC pp. 198–200 and DEC pp. 201–203; TM p. 49: no flags.
        // Wrapping follows the 16-bit width; no ALU flag result is installed.
        (Table::Main, 0x03 | 0x13 | 0x23 | 0x33) | (Table::Index, 0x23) => {
            pair.set(registers, pair.get(registers).wrapping_add(1));
        }
        (Table::Main, 0x0B | 0x1B | 0x2B | 0x3B) | (Table::Index, 0x2B) => {
            pair.set(registers, pair.get(registers).wrapping_sub(1));
        }
        // The group-owned encoding lists admit only the arms above.
        _ => {}
    }
    Path::Untaken
}

#[derive(Clone, Copy)]
enum Pair {
    Bc,
    De,
    Hl,
    Sp,
    Ix,
    Iy,
}

fn accumulator_pair(index: Option<IndexRegister>) -> Pair {
    match index {
        None => Pair::Hl,
        Some(IndexRegister::Ix) => Pair::Ix,
        Some(IndexRegister::Iy) => Pair::Iy,
    }
}

// ss: UM pp. 188, 190, 192, 198, 201. pp/rr: UM pp. 194/196 explicitly
// select IX/IY itself at field 10, not HL; TM Table 7.0-6 p. 49 agrees.
// Indexed INC/DEC have fixed field 10 too (UM pp. 199–200, 202–203).
fn operand_pair(decoded: &Decoded) -> Pair {
    match (decoded.opcode >> 4) & 3 {
        0 => Pair::Bc,
        1 => Pair::De,
        2 => accumulator_pair(decoded.index),
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
            Self::Ix => registers.ix = value,
            Self::Iy => registers.iy = value,
        }
    }
}
