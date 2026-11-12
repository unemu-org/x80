//! The 8085 Arithmetic encodings and executor (plan §2.2; decision-10).
//!
//! Intel 205775-002 (January 1983), §5.6.2, pp. 5-6 to 5-9, checked on page images.
//! Totals below are the 8085 states, not machine cycles or 8080 states.
//! Independent witness: `opcodes/tests.rs`, Table 5-1 p. 5-19 cross-read
//! with Intel 9800301-04 (May 1981), App. A pp. A-1 to A-5.
//! Bus order: Table 5-1 p. 5-19 (PDF 104); Table 5-2 p. 5-20 is
//! the opcode list. Ch. 2 pp. 2-8 to 2-12 (PDF 31-35) defines OF/MR/MW
//! and bus idle. Access kinds and completed-boundary state: decision-12,
//! extending decision-5 items 3/8. No wait states or HOLD are modelled.
//! All cited source values checked on rendered page images.

use super::opcodes::{Encoding, Instruction, Path, States};
use super::{alu, Registers, I8085};
use crate::{Bus, MemoryAccess};

/// Complete group list, in per-instruction page order.
/// Register-field exclusions live in `opcodes` (§5.2 p. 5-1).
pub(super) const ENCODINGS: &[Encoding] = &[
    // ADD r: 205775-002 p. 5-6.
    Encoding::new(b"10000SSS", 1, States::single(4)),
    // ADD M: 205775-002 p. 5-6.
    Encoding::new(b"10000110", 1, States::single(7)),
    // ADI data: 205775-002 p. 5-6.
    Encoding::new(b"11000110", 2, States::single(7)),
    // ADC r: 205775-002 p. 5-6.
    Encoding::new(b"10001SSS", 1, States::single(4)),
    // ADC M: 205775-002 p. 5-7.
    Encoding::new(b"10001110", 1, States::single(7)),
    // ACI data: 205775-002 p. 5-7.
    Encoding::new(b"11001110", 2, States::single(7)),
    // SUB r: 205775-002 p. 5-7.
    Encoding::new(b"10010SSS", 1, States::single(4)),
    // SUB M: 205775-002 p. 5-7.
    Encoding::new(b"10010110", 1, States::single(7)),
    // SUI data: 205775-002 p. 5-7.
    Encoding::new(b"11010110", 2, States::single(7)),
    // SBB r: 205775-002 p. 5-7.
    Encoding::new(b"10011SSS", 1, States::single(4)),
    // SBB M: 205775-002 p. 5-8.
    Encoding::new(b"10011110", 1, States::single(7)),
    // SBI data: 205775-002 p. 5-8.
    Encoding::new(b"11011110", 2, States::single(7)),
    // INR r: 205775-002 p. 5-8.
    Encoding::new(b"00DDD100", 1, States::single(4)),
    // INR M: 205775-002 p. 5-8.
    Encoding::new(b"00110100", 1, States::single(10)),
    // DCR r: 205775-002 p. 5-8.
    Encoding::new(b"00DDD101", 1, States::single(4)),
    // DCR M: 205775-002 p. 5-8.
    Encoding::new(b"00110101", 1, States::single(10)),
    // INX rp: 205775-002 p. 5-9.
    Encoding::new(b"00RP0011", 1, States::single(6)),
    // DCX rp: 205775-002 p. 5-9.
    Encoding::new(b"00RP1011", 1, States::single(6)),
    // DAD rp: 205775-002 p. 5-9.
    Encoding::new(b"00RP1001", 1, States::single(10)),
    // DAA: 205775-002 p. 5-9.
    Encoding::new(b"00100111", 1, States::single(4)),
];

/// Executes an encoding that the opcode map assigns to this group.
pub(super) fn execute<S: Bus>(
    cpu: &mut I8085,
    system: &mut S,
    opcode: u8,
    instruction: Instruction,
) -> Path {
    let registers = cpu.registers_mut();
    match opcode {
        // 205775-002 §5.6.2 p. 5-9 (PDF 94); second witness
        // 9800301-04 Ch. 3 p. 3-19 (PDF 73): low step first, then
        // the high digit "now" greater than 9. AC is the low step's
        // bit-3 carry (§5.5 p. 5-3); p. 3-19's literal 9BH -> A1H ->
        // 01H ends with AC=1 even though the high step adds 60H.
        0x27 => {
            let correction = if registers.a & 15 > 9 || registers.flags.auxiliary_carry {
                6
            } else {
                0
            };
            let (low, low_flags) = alu::add(registers.a, correction, false);
            // Derived for this model, NOT explicit in p. 5-9:
            // retain incoming CY and apply §5.5 p. 5-3's CY rule per
            // adjustment. Step 1's bit-7 carry triggers step 2, and
            // either step's carry sets final CY. Decimal addition
            // needs that carry for higher bytes (9800301-04 Ch. 6
            // pp. 6-12 to 6-14, PDF 178-180). See PROVENANCE.md §1.
            let carry_for_high = registers.flags.carry || low_flags.carry;
            let (value, high_carry) = if low >> 4 > 9 || carry_for_high {
                low.overflowing_add(0x60)
            } else {
                (low, false)
            };
            registers.a = value;
            registers.flags = alu::result_flags(
                value,
                low_flags.auxiliary_carry,
                carry_for_high || high_carry,
            );
        }
        // 205775-002 ADD/ADI/ADC p. 5-6; ADC/ACI/SUB/SUI/SBB
        // p. 5-7; SBB/SBI p. 5-8 (PDF 91-93). Second witness:
        // 9800301-04 ADD pp. 3-4/3-5, ADI p. 3-5, ADC p. 3-3,
        // ACI p. 3-2, SUB pp. 3-63/3-64, SUI pp. 3-64/3-65,
        // SBB pp. 3-56/3-57, SBI pp. 3-57/3-58 (PDF = 54+n).
        // F only (r); F R (M/immediate), Table 5-1. Sample A before
        // replacing it when A itself is the source.
        0x80..=0x9F | 0xC6 | 0xCE | 0xD6 | 0xDE => {
            let operand = if opcode & 0xC0 == 0xC0 {
                // Byte 2 (§5.4 p. 5-3), MR in M2 (Ch. 2 §2.3.2).
                // PC wrap is derived from its 16-bit width.
                system.read(registers.pc.wrapping_add(1), MemoryAccess::OperandFetch)
            } else {
                read_operand(registers, system, opcode & 7)
            };
            let operation = (opcode >> 3) & 3;
            let input = operation & 1 != 0 && registers.flags.carry;
            let (value, flags) = if operation & 2 != 0 {
                alu::subtract(registers.a, operand, input)
            } else {
                alu::add(registers.a, operand, input)
            };
            registers.a = value;
            registers.flags = flags;
        }
        // INR/DCR: 205775-002 p. 5-8 (PDF 93), all flags except CY;
        // 9800301-04 INR p. 3-25, DCR pp. 3-20/3-21 (PDF 79,74/75).
        // AC follows §5.5 p. 5-3's bit-3 carry, explicitly including
        // decrements; two's-complement arithmetic (§5.6.2 p. 5-6)
        // gives DCR AC = no low-nibble borrow (alu::subtract).
        // Second witness AC: 9800301-04 Ch. 1 pp. 1-11/1-12.
        // F only (r); F R W (M) at the same HL address, Table 5-1.
        _ if opcode & 0xC6 == 0x04 => {
            let destination = (opcode >> 3) & 7;
            let operand = read_operand(registers, system, destination);
            let (value, mut flags) = if opcode & 1 == 0 {
                alu::add(operand, 1, false)
            } else {
                alu::subtract(operand, 1, false)
            };
            flags.carry = registers.flags.carry;
            write_operand(registers, system, destination, value);
            registers.flags = flags;
        }
        // INX/DCX: 205775-002 p. 5-9 (PDF 94), no flags;
        // 9800301-04 pp. 3-26/3-22 (PDF 80/76). S fetch only,
        // 6 states (Table 5-1; Ch. 2 Fig. 2-13 p. 2-10).
        // Pair wrap is derived from the 16-bit register width.
        _ if opcode & 0xC7 == 0x03 => {
            let pair = (opcode >> 4) & 3;
            let value = read_pair(registers, pair);
            let result = if opcode & 8 == 0 {
                value.wrapping_add(1)
            } else {
                value.wrapping_sub(1)
            };
            write_pair(registers, pair, result);
        }
        // DAD: 205775-002 p. 5-9 (PDF 94), only CY, set on carry
        // and otherwise reset; 9800301-04 p. 3-20 (PDF 74).
        // F B B, 10 states (Table 5-1). Both bus-idle machine cycles
        // make no Bus call: Ch. 2 Fig. 2-10 p. 2-8 / Fig. 2-12 p. 2-9;
        // 231718-001 Tables 3/4, handbook p. 16-20 (PDF 967).
        // Wrapped HL is derived from its 16-bit width.
        _ if opcode & 0xCF == 0x09 => {
            let operand = read_pair(registers, (opcode >> 4) & 3);
            let (value, carry) = registers.hl().overflowing_add(operand);
            registers.set_hl(value);
            registers.flags.carry = carry;
        }
        _ => unreachable!("opcode map dispatches only this group's encodings"),
    }
    // Sequential execution: §5.4 p. 5-3 (PDF 88); 16-bit PC wrap
    // is derived, not stated there. Dispatch does not advance PC.
    registers.pc = registers.pc.wrapping_add(u16::from(instruction.length));
    Path::Taken
}

// Register fields: 205775-002 §5.2 p. 5-1 and Table 5-1 p. 5-19;
// second witness 9800301-04 pp. 3-3/3-4. M uses HL, §5.6.2 pp. 5-6
// to 5-8. MR/MW: Ch. 2 §§2.3.2/2.3.3 pp. 2-10 to 2-12.
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

fn write_operand<S: Bus>(registers: &mut Registers, system: &mut S, code: u8, value: u8) {
    match code {
        0 => registers.b = value,
        1 => registers.c = value,
        2 => registers.d = value,
        3 => registers.e = value,
        4 => registers.h = value,
        5 => registers.l = value,
        6 => system.write(registers.hl(), value, MemoryAccess::Data),
        _ => registers.a = value,
    }
}

// RP mapping/high-low order: 205775-002 §5.2 p. 5-1, Table 5-1 p. 5-19;
// 9800301-04 DAD/DCX/INX pp. 3-20/3-22/3-26.
fn read_pair(registers: &Registers, pair: u8) -> u16 {
    match pair {
        0 => registers.bc(),
        1 => registers.de(),
        2 => registers.hl(),
        _ => registers.sp,
    }
}

fn write_pair(registers: &mut Registers, pair: u8, value: u16) {
    match pair {
        0 => registers.set_bc(value),
        1 => registers.set_de(value),
        2 => registers.set_hl(value),
        _ => registers.sp = value,
    }
}
