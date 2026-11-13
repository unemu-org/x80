//! 8-Bit Arithmetic Group (decision-13 rule 2).
//!
//! Primary: Zilog UM008011-0816, rev. 11 (August 2016), pp. 144–171.
//! Entry citations below use printed pages; PDF = printed + 14.
//! Independent cross-read: 03-0029-01 (1977), Table 7.0-4, p. 47
//! (PDF = printed + 6). Whole-instruction lengths and T-state totals include
//! effective prefixes, operands and displacements (decision-13 rules 3–5).
//! Extra ignored/repeated selectors are accounted in cpu.rs (decision-20 §2).

use super::cpu::{Decoded, IndexRegister, Table};
use super::opcodes::{Encoding, Instruction, States};
use super::{alu, load8};
use super::{Registers, Z80};
use crate::decode::Path;
use crate::{Bus, MemoryAccess};

// Entry bit-diagram slips on pp. 151, 159: see PROVENANCE.md §1.

pub(super) const MAIN: &[Encoding] = &[
    // UM ADD A,r, p. 145.
    Encoding::new(b"10000RRR", 1, States::single(4)),
    // UM ADD A,n, p. 147.
    Encoding::new(b"11000110", 2, States::single(7)),
    // UM ADD A,(HL), p. 148.
    Encoding::new(b"10000110", 1, States::single(7)),
    // UM ADC A,r, pp. 151–152.
    Encoding::new(b"10001RRR", 1, States::single(4)),
    // UM ADC A,n, pp. 151–152.
    Encoding::new(b"11001110", 2, States::single(7)),
    // UM ADC A,(HL), pp. 151–152.
    Encoding::new(b"10001110", 1, States::single(7)),
    // UM SUB r, pp. 153–154.
    Encoding::new(b"10010RRR", 1, States::single(4)),
    // UM SUB n, pp. 153–154.
    Encoding::new(b"11010110", 2, States::single(7)),
    // UM SUB (HL), pp. 153–154.
    Encoding::new(b"10010110", 1, States::single(7)),
    // UM SBC A,r, pp. 155–156.
    Encoding::new(b"10011RRR", 1, States::single(4)),
    // UM SBC A,n, pp. 155–156.
    Encoding::new(b"11011110", 2, States::single(7)),
    // UM SBC A,(HL), pp. 155–156.
    Encoding::new(b"10011110", 1, States::single(7)),
    // UM AND r, pp. 157–158.
    Encoding::new(b"10100RRR", 1, States::single(4)),
    // UM AND n, pp. 157–158.
    Encoding::new(b"11100110", 2, States::single(7)),
    // UM AND (HL), pp. 157–158.
    Encoding::new(b"10100110", 1, States::single(7)),
    // UM XOR r, pp. 161–162.
    Encoding::new(b"10101RRR", 1, States::single(4)),
    // UM XOR n, pp. 161–162.
    Encoding::new(b"11101110", 2, States::single(7)),
    // UM XOR (HL), pp. 161–162.
    Encoding::new(b"10101110", 1, States::single(7)),
    // UM OR r, pp. 159–160.
    Encoding::new(b"10110RRR", 1, States::single(4)),
    // UM OR n, pp. 159–160.
    Encoding::new(b"11110110", 2, States::single(7)),
    // UM OR (HL), pp. 159–160.
    Encoding::new(b"10110110", 1, States::single(7)),
    // UM CP r, pp. 163–164.
    Encoding::new(b"10111RRR", 1, States::single(4)),
    // UM CP n, pp. 163–164.
    Encoding::new(b"11111110", 2, States::single(7)),
    // UM CP (HL), pp. 163–164.
    Encoding::new(b"10111110", 1, States::single(7)),
    // UM INC r, p. 165.
    Encoding::new(b"00RRR100", 1, States::single(4)),
    // UM INC (HL), p. 167.
    Encoding::new(b"00110100", 1, States::single(11)),
    // UM DEC r, pp. 170–171.
    Encoding::new(b"00RRR101", 1, States::single(4)),
    // UM DEC (HL), pp. 170–171.
    Encoding::new(b"00110101", 1, States::single(11)),
];

pub(super) const INDEX: &[Encoding] = &[
    // H operation/flags: decision-20 §2; Young v0.91 §3.2 pp. 11–12 /
    // §8.4 p. 30; Rak v1.2a .inc_x/.dec_x/.alo_a_x/.alo_a_y.
    // New costs L. O selects all eight ordinary ALU operations.
    Encoding::new(b"00100100", 2, States::single(8)),
    Encoding::new(b"00100101", 2, States::single(8)),
    Encoding::new(b"00101100", 2, States::single(8)),
    Encoding::new(b"00101101", 2, States::single(8)),
    Encoding::new(b"10OOO100", 2, States::single(8)),
    Encoding::new(b"10OOO101", 2, States::single(8)),
    // UM ADD A,(IX/IY+d), pp. 149–150.
    Encoding::new(b"10000110", 3, States::single(19)),
    // UM ADC A,(IX/IY+d), pp. 151–152.
    Encoding::new(b"10001110", 3, States::single(19)),
    // UM SUB (IX/IY+d), pp. 153–154.
    Encoding::new(b"10010110", 3, States::single(19)),
    // UM SBC A,(IX/IY+d), pp. 155–156.
    Encoding::new(b"10011110", 3, States::single(19)),
    // UM AND (IX/IY+d), pp. 157–158.
    Encoding::new(b"10100110", 3, States::single(19)),
    // UM XOR (IX/IY+d), pp. 161–162.
    Encoding::new(b"10101110", 3, States::single(19)),
    // UM OR (IX/IY+d), pp. 159–160.
    Encoding::new(b"10110110", 3, States::single(19)),
    // UM CP (IX/IY+d), pp. 163–164.
    Encoding::new(b"10111110", 3, States::single(19)),
    // UM INC (IX/IY+d), pp. 168–169.
    Encoding::new(b"00110100", 3, States::single(23)),
    // UM DEC (IX/IY+d), pp. 170–171.
    Encoding::new(b"00110101", 3, States::single(23)),
];

/// PC is after prefix/key bytes (decision-13 rule 4). All flag rules come
/// from the model's landed ALU helpers; no flag equations are repeated here.
/// UM pp. 145–171; 03-0029-01 Table 7.0-4 p. 47. Existing PROVENANCE §1
/// rows resolve SBC's overflow and AND/OR's parity wording slips.
pub(super) fn execute<S: Bus>(
    cpu: &mut Z80,
    system: &mut S,
    _instruction: &Instruction,
    decoded: &Decoded,
) -> Path {
    // INDEX d belongs to the executor and precedes the data read (decision-15
    // rule 5). UM pp. 149–150,168–171; TM p. 47: signed two's-complement
    // displacement. The u16 address wrap is derived from index width.
    let opcode = decoded.opcode;
    let code = if opcode & 0xC0 == 0 {
        (opcode >> 3) & 7
    } else {
        opcode & 7
    };
    let address = if decoded.table == Table::Index && code == 6 {
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
    // H: decision-20 §2; Young v0.91 §3.2 / §8.4 p. 30;
    // Rak v1.2a .inc_x/.dec_x, .alo_a_x/.alo_a_y. Register-only
    // H/L use selected halves; memory still follows documented INDEX d.
    let carry = cpu.registers().flags.carry;
    if opcode & 0xC0 == 0 {
        // INC/DEC r and memory: UM pp. 165–171; TM p. 47. Read before
        // writing the same location, both as Data (decision-15 rule 5).
        // Carry is preserved by the ALU and does not enter the byte result.
        let code = (opcode >> 3) & 7;
        let value = read_operand(cpu.registers(), system, code, address, decoded.index);
        let (result, flags) = if opcode & 1 == 0 {
            alu::increment(value, carry)
        } else {
            alu::decrement(value, carry)
        };
        write_operand(
            cpu.registers_mut(),
            system,
            code,
            address,
            result,
            decoded.index,
        );
        cpu.registers_mut().flags = flags;
    } else {
        // UM pp. 145–164; TM p. 47. Only n uses an immediate byte; r and
        // memory share the low field. Capture A/source before replacement,
        // including A as its own source. Only ADC/SBC consume incoming C.
        let source = if opcode & 0xC0 == 0xC0 {
            cpu.fetch_operand(system)
        } else {
            read_operand(cpu.registers(), system, opcode & 7, address, decoded.index)
        };
        let a = cpu.registers().a;
        let (result, flags) = match (opcode >> 3) & 7 {
            0 => alu::add(a, source, false),
            1 => alu::add(a, source, carry),
            2 => alu::subtract(a, source, false),
            3 => alu::subtract(a, source, carry),
            4 => alu::and(a, source),
            5 => alu::xor(a, source),
            6 => alu::or(a, source),
            // CP supplies subtraction flags without replacing A (UM p. 164).
            _ => (a, alu::compare(a, source)),
        };
        cpu.registers_mut().a = result;
        cpu.registers_mut().flags = flags;
    }
    // Fixed whole-instruction costs, including prefixes, are retired by the
    // dispatcher (decision-13 rules 4–5); there is no conditional path here.
    Path::Untaken
}

// Register fields: UM pp. 145,152,154,156,158,160,162,164–165,170;
// TM Table 7.0-4 p. 47. Index halves follow decision-20 §2 as cited above.
fn read_operand<S: Bus>(
    registers: &Registers,
    system: &mut S,
    code: u8,
    address: u16,
    index: Option<IndexRegister>,
) -> u8 {
    if code == 6 {
        system.read(address, MemoryAccess::Data)
    } else {
        load8::read_register(registers, code, index)
    }
}

// INC/DEC destinations: UM pp. 165–171; TM p. 47; decision-20 §2
// (H index-half operation, sources above). Data kind: decision-15 rule 5.
fn write_operand<S: Bus>(
    registers: &mut Registers,
    system: &mut S,
    code: u8,
    address: u16,
    value: u8,
    index: Option<IndexRegister>,
) {
    if code == 6 {
        system.write(address, value, MemoryAccess::Data);
    } else {
        load8::write_register(registers, code, value, index);
    }
}
