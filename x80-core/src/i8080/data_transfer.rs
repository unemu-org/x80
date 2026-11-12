//! The 8080 Data Transfer Group: its encodings and its executor.
//!
//! Primary source: 98-153B Ch. 4, "Data Transfer Group", pp. 4-4 and 4-5,
//! checked against the page images. Field letters are those of
//! `Encoding` in `opcodes`.
//!
//! Design: decision-5 items 3, 7, 8; decision-6 and plan §2.2/§2.6.
//! Access order: 98-153B Ch. 2 machine-cycle table pp. 2-16/2-17,
//! notes 1, 6, 7 on p. 2-20. The caller has already fetched the opcode.
//! All instructions preserve flags (Ch. 4 p. 4-4, group introduction).
//! Independent witnesses: tests/i8080_data_transfer.rs, also checked against
//! Intel 9800301-04 (May 1981), Ch. 3 entries cited with the encodings below.

use super::opcodes::{Encoding, Instruction, Path, States};
use super::{Registers, I8080};
use crate::{Bus, MemoryAccess};

/// The group's encodings, in page order.
pub(super) const ENCODINGS: &[Encoding] = &[
    // MOV r1,r2: 5 states, 98-153B Ch. 4 p. 4-4;
    // 9800301-04 Ch. 3 p. 3-36 (8080 total).
    Encoding::new(b"01DDDSSS", 1).with_states(States::single(5)),
    // MOV r,M: 7 states, 98-153B p. 4-4; 9800301-04 p. 3-37.
    Encoding::new(b"01DDD110", 1).with_states(States::single(7)),
    // MOV M,r: 7 states, 98-153B p. 4-4; 9800301-04 p. 3-36.
    Encoding::new(b"01110SSS", 1).with_states(States::single(7)),
    // MVI r,data: 7 states, opcode and data byte, 98-153B p. 4-4;
    // 9800301-04 p. 3-37.
    Encoding::new(b"00DDD110", 2).with_states(States::single(7)),
    // MVI M,data: 10 states, 98-153B p. 4-4; 9800301-04 p. 3-38.
    Encoding::new(b"00110110", 2).with_states(States::single(10)),
    // LXI rp,data 16: 10 states; opcode, low data, high data,
    // 98-153B p. 4-4; 9800301-04 p. 3-35.
    Encoding::new(b"00RP0001", 3).with_states(States::single(10)),
    // LDA addr: 13 states; opcode, low address, high address,
    // 98-153B p. 4-5; 9800301-04 pp. 3-32/3-33.
    Encoding::new(b"00111010", 3).with_states(States::single(13)),
    // STA addr: 13 states, 98-153B p. 4-5; 9800301-04 pp. 3-61/3-62.
    Encoding::new(b"00110010", 3).with_states(States::single(13)),
    // LHLD addr: 16 states, 98-153B p. 4-5; 9800301-04 p. 3-34.
    Encoding::new(b"00101010", 3).with_states(States::single(16)),
    // SHLD addr: 16 states, 98-153B p. 4-5; 9800301-04 pp. 3-58/3-59.
    Encoding::new(b"00100010", 3).with_states(States::single(16)),
    // LDAX rp, 00RP1010: "only register pairs rp=B (registers B and C) or
    // rp=D (registers D and E) may be specified", 7 states, 98-153B p. 4-5;
    // 9800301-04 p. 3-33.
    Encoding::new(b"00001010", 1).with_states(States::single(7)),
    Encoding::new(b"00011010", 1).with_states(States::single(7)),
    // STAX rp, 00RP0010, same restriction, 7 states, 98-153B p. 4-5;
    // 9800301-04 p. 3-62.
    Encoding::new(b"00000010", 1).with_states(States::single(7)),
    Encoding::new(b"00010010", 1).with_states(States::single(7)),
    // XCHG: 4 states, 98-153B p. 4-5; 9800301-04 p. 3-65.
    Encoding::new(b"11101011", 1).with_states(States::single(4)),
];

/// Executes `opcode`, an instruction of this group.
///
/// Returns `None`, having changed nothing and accessed nothing, for an opcode
/// outside this group (decision-5 item 7).
pub(super) fn execute<S: Bus>(
    cpu: &mut I8080,
    system: &mut S,
    opcode: u8,
    instruction: Instruction,
) -> Option<Path> {
    let registers = cpu.registers_mut();
    match opcode {
        // MOV: 98-153B Ch. 4 p. 4-4. Ch. 2 p. 2-16: memory variants
        // access original HL in M2, register moves have only M1.
        0x40..=0x7F if opcode != 0x76 => {
            let value = read_location(registers, system, opcode & 7);
            write_location(registers, system, (opcode >> 3) & 7, value);
        }
        // MVI: Ch. 4 p. 4-4; Ch. 2 pp. 2-16/2-17: byte 2 read in M2,
        // then (memory form only) HL write in M3.
        0x06 | 0x0E | 0x16 | 0x1E | 0x26 | 0x2E | 0x36 | 0x3E => {
            let value = operand(registers, system, 1);
            write_location(registers, system, (opcode >> 3) & 7, value);
        }
        // LXI: Ch. 4 pp. 4-3/4-4 RP table, byte 2 low / byte 3 high;
        // Ch. 2 pp. 2-16/2-17: operand reads in M2 then M3.
        0x01 | 0x11 | 0x21 | 0x31 => {
            let value = operand_word(registers, system);
            match opcode {
                0x01 => registers.set_bc(value),
                0x11 => registers.set_de(value),
                0x21 => registers.set_hl(value),
                _ => registers.sp = value,
            }
        }
        // Direct transfers: Ch. 4 p. 4-5; Ch. 2 pp. 2-16/2-17:
        // M2 low address, M3 high address, M4 A or L, M5 H if present.
        0x3A | 0x32 | 0x2A | 0x22 => {
            let address = operand_word(registers, system);
            match opcode {
                0x3A => registers.a = system.read(address, MemoryAccess::Data),
                0x32 => system.write(address, registers.a, MemoryAccess::Data),
                0x2A => {
                    registers.l = system.read(address, MemoryAccess::Data);
                    registers.h = system.read(address.wrapping_add(1), MemoryAccess::Data);
                }
                _ => {
                    system.write(address, registers.l, MemoryAccess::Data);
                    system.write(address.wrapping_add(1), registers.h, MemoryAccess::Data);
                }
            }
            // Ch. 2 p. 2-17 increments WZ between L and H accesses; p. 2-2
            // gives 16-bit address logic. FFFF -> 0000 is derived from that
            // width; neither page explicitly verifies the wrap edge.
        }
        // LDAX/STAX: Ch. 4 p. 4-5; Ch. 2 p. 2-16: one M2 data
        // read/write at BC or DE. Access kinds: decision-5 item 3.
        0x0A | 0x1A | 0x02 | 0x12 => {
            let address = if opcode & 0x10 == 0 {
                registers.bc()
            } else {
                registers.de()
            };
            if opcode & 0x08 != 0 {
                registers.a = system.read(address, MemoryAccess::Data);
            } else {
                system.write(address, registers.a, MemoryAccess::Data);
            }
        }
        // XCHG: Ch. 4 p. 4-5 and Ch. 2 p. 2-16, M1 only.
        0xEB => {
            core::mem::swap(&mut registers.h, &mut registers.d);
            core::mem::swap(&mut registers.l, &mut registers.e);
        }
        _ => return None,
    }
    // Ch. 4 p. 4-2: sequential execution over the instruction's bytes;
    // Ch. 2 pp. 2-16/2-17: PC increments at each instruction-byte read.
    // PC is 16-bit (p. 2-2); wrap is derived, not explicitly stated there.
    // Flags are untouched: Ch. 4 p. 4-4, Data Transfer Group introduction.
    registers.pc = registers.pc.wrapping_add(u16::from(instruction.length));
    Some(Path::Taken)
}

// Design: decision-5 item 3. 98-153B Ch. 2 pp. 2-16/2-17 puts instruction
// bytes after the opcode at PC+1/PC+2. The API calls these OperandFetch.
fn operand<S: Bus>(registers: &Registers, system: &mut S, offset: u16) -> u8 {
    system.read(
        registers.pc.wrapping_add(offset),
        MemoryAccess::OperandFetch,
    )
}

// Ch. 4 pp. 4-4/4-5: byte 2 is low, byte 3 high; Ch. 2 pp. 2-16/2-17:
// M2 precedes M3. Capture the address before any data writes.
fn operand_word<S: Bus>(registers: &Registers, system: &mut S) -> u16 {
    let low = operand(registers, system, 1);
    let high = operand(registers, system, 2);
    u16::from_le_bytes([low, high])
}

// Ch. 4 p. 4-3 DDD/SSS table; p. 4-4 memory form uses HL. Ch. 2 p. 2-16
// gives the memory read/write cycle. The caller masks the code to 3 bits;
// the remaining code 111 names A. Design: decision-5 item 3.
fn read_location<S: Bus>(registers: &Registers, system: &mut S, code: u8) -> u8 {
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

// 98-153B Ch. 4 pp. 4-3/4-4: DDD selects the destination register or HL
// memory; Ch. 2 pp. 2-16/2-17 supplies the write cycle. decision-5 item 3.
fn write_location<S: Bus>(registers: &mut Registers, system: &mut S, code: u8, value: u8) {
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
