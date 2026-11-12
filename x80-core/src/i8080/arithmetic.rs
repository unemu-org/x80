//! The 8080 Arithmetic Group: its encodings and its executor.
//!
//! Primary source: 98-153B Ch. 4, "Arithmetic Group", pp. 4-6 to 4-8 (DAA is
//! the last entry before "Logical Group" on p. 4-8), checked against the page
//! images. Field letters are those of `Encoding` in `opcodes`.
//!
//! Design: plan §2.2; decision-5 items 3 and 8; decision-6. Arithmetic
//! executes except DAA, which remains deferred to its own bead (decision-5
//! item 7). Flag definitions live in `alu`.

use super::opcodes::{Encoding, Instruction, Path, States};
use super::{alu, Registers, I8080};
use crate::{Bus, MemoryAccess};

/// The group's encodings, in page order. Each total is the entry's
/// "States:" value on the cited printed page (PDF page = 44 + printed page).
pub(super) const ENCODINGS: &[Encoding] = &[
    // ADD r, p. 4-6.
    Encoding::new(b"10000SSS", 1).with_states(States::single(4)),
    // ADD M, p. 4-6.
    Encoding::new(b"10000110", 1).with_states(States::single(7)),
    // ADI data: opcode and data byte, p. 4-6.
    Encoding::new(b"11000110", 2).with_states(States::single(7)),
    // ADC r, p. 4-6.
    Encoding::new(b"10001SSS", 1).with_states(States::single(4)),
    // ADC M, p. 4-6.
    Encoding::new(b"10001110", 1).with_states(States::single(7)),
    // ACI data, p. 4-6.
    Encoding::new(b"11001110", 2).with_states(States::single(7)),
    // SUB r, p. 4-6.
    Encoding::new(b"10010SSS", 1).with_states(States::single(4)),
    // SUB M, p. 4-7.
    Encoding::new(b"10010110", 1).with_states(States::single(7)),
    // SUI data, p. 4-7.
    Encoding::new(b"11010110", 2).with_states(States::single(7)),
    // SBB r, p. 4-7.
    Encoding::new(b"10011SSS", 1).with_states(States::single(4)),
    // SBB M, p. 4-7.
    Encoding::new(b"10011110", 1).with_states(States::single(7)),
    // SBI data, p. 4-7.
    Encoding::new(b"11011110", 2).with_states(States::single(7)),
    // INR r, p. 4-7.
    Encoding::new(b"00DDD100", 1).with_states(States::single(5)),
    // INR M, p. 4-7.
    Encoding::new(b"00110100", 1).with_states(States::single(10)),
    // DCR r, p. 4-7.
    Encoding::new(b"00DDD101", 1).with_states(States::single(5)),
    // DCR M, p. 4-8.
    Encoding::new(b"00110101", 1).with_states(States::single(10)),
    // INX rp, p. 4-8.
    Encoding::new(b"00RP0011", 1).with_states(States::single(5)),
    // DCX rp, p. 4-8.
    Encoding::new(b"00RP1011", 1).with_states(States::single(5)),
    // DAD rp, p. 4-8.
    Encoding::new(b"00RP1001", 1).with_states(States::single(10)),
    // DAA, p. 4-8.
    Encoding::new(b"00100111", 1).with_states(States::single(4)),
];

/// Executes the arithmetic instructions except deferred DAA (decision-5 §7).
/// An unimplemented opcode returns before any access or state change.
pub(super) fn execute<S: Bus>(
    cpu: &mut I8080,
    system: &mut S,
    opcode: u8,
    instruction: Instruction,
) -> Option<Path> {
    let registers = cpu.registers_mut();
    match opcode {
        // ADD/ADC/SUB/SBB: 98-153B pp. 4-6/4-7. Register sources are
        // unchanged, including A sampled before the result replaces it.
        0x80..=0x9F | 0xC6 | 0xCE | 0xD6 | 0xDE => {
            let operand = if opcode & 0xC0 == 0xC0 {
                // M2 reads byte 2 at PC+1: pp. 2-16/2-17, p. 2-20
                // note 6. OperandFetch is decision-5 item 3.
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
        // INR/DCR: pp. 4-7/4-8, all flags except CY. M reads in M2
        // and writes the same HL address in M3 (pp. 2-16/2-17).
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
        // INX/DCX: p. 4-8, 16-bit pair +/- 1, no flags. Only M1
        // accesses memory (pp. 2-16/2-17). RP codes: p. 4-3.
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
        // DAD: p. 4-8, HL <- HL + rp, CY only. M2/M3 are internal,
        // with no memory reference (p. 2-20 note 8; pp. 2-16/2-17).
        _ if opcode & 0xCF == 0x09 => {
            let operand = read_pair(registers, (opcode >> 4) & 3);
            let (value, carry) = registers.hl().overflowing_add(operand);
            registers.set_hl(value);
            registers.flags.carry = carry;
        }
        _ => return None,
    }
    // Sequential instruction bytes: p. 4-2; fetch increments PC:
    // p. 2-16. Wrapping follows its 16-bit width (p. 2-2), as for NOP.
    // The API exposes the completed result at this boundary (decision-5 §8);
    // the hardware's overlapped writeback in p. 2-20 note 9 adds no bus read.
    registers.pc = registers.pc.wrapping_add(u16::from(instruction.length));
    Some(Path::Taken)
}

/// Register codes, 98-153B p. 4-3; M2 data read at HL, pp. 2-16/2-17,
/// p. 2-20 note 6 (Data access under decision-5 item 3).
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

/// INR/DCR destinations: pp. 4-7/4-8; M3 data write at HL,
/// pp. 2-16/2-17, p. 2-20 note 7 (decision-5 item 3).
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

/// RP codes including SP, 98-153B p. 4-3; INX/DCX/DAD p. 4-8.
fn read_pair(registers: &Registers, pair: u8) -> u16 {
    match pair {
        0 => registers.bc(),
        1 => registers.de(),
        2 => registers.hl(),
        _ => registers.sp,
    }
}

/// RP destinations, 98-153B p. 4-3; INX/DCX p. 4-8.
fn write_pair(registers: &mut Registers, pair: u8, value: u16) {
    match pair {
        0 => registers.set_bc(value),
        1 => registers.set_de(value),
        2 => registers.set_hl(value),
        _ => registers.sp = value,
    }
}
