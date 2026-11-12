//! The 8085 Data Transfer encodings and executor.
//!
//! Intel 205775-002 (January 1983), §5.6.1, pp. 5-4 to 5-6, checked on page images.
//! Totals below are the 8085 states, not machine cycles or 8080 states.
//! Independent witnesses: `tests/i8085_data_transfer.rs`, cross-read with
//! Intel 9800301-04 (May 1981), Ch. 3 entries cited below. Source hashes
//! match PROVENANCE.md §1; all cited values checked on rendered page images.
//! Bus order: Table 5-1 p. 5-19 (machine cycles); Table 5-2 p. 5-20
//! lists opcodes, not cycles. Ch. 2 §§2.3.1-2.3.3 pp. 2-8 to 2-12
//! distinguishes opcode fetch, memory read and memory write. OperandFetch
//! versus Data is the decision-5 item 3 API distinction.
//! Every instruction preserves flags (§5.6.1 p. 5-4).
//! Design: decision-10, decision-11, decision-12 rules 1/10;
//! decision-5 items 3/7/8; plan §2.2/§2.6.

use super::opcodes::{Encoding, Instruction, Path, States};
use super::{Registers, I8085};
use crate::{Bus, MemoryAccess};

/// Complete group list, in per-instruction page order.
/// LDAX/STAX permit only B and D pairs (205775-002 pp. 5-5/5-6).
/// Register-field exclusions live in `opcodes` (§5.2 p. 5-1).
pub(super) const ENCODINGS: &[Encoding] = &[
    // MOV r1,r2: 205775-002 p. 5-4; 9800301-04 p. 3-36 (8085 total).
    Encoding::new(b"01DDDSSS", 1, States::single(4)),
    // MOV r,M: 205775-002 p. 5-4; 9800301-04 pp. 3-36/3-37.
    Encoding::new(b"01DDD110", 1, States::single(7)),
    // MOV M,r: 205775-002 p. 5-4; 9800301-04 p. 3-36.
    Encoding::new(b"01110SSS", 1, States::single(7)),
    // MVI r,data: 205775-002 p. 5-4; 9800301-04 p. 3-37.
    Encoding::new(b"00DDD110", 2, States::single(7)),
    // MVI M,data: 205775-002 p. 5-4; 9800301-04 p. 3-38.
    Encoding::new(b"00110110", 2, States::single(10)),
    // LXI rp,data16: 205775-002 p. 5-5; 9800301-04 p. 3-35.
    Encoding::new(b"00RP0001", 3, States::single(10)),
    // LDA addr: 205775-002 p. 5-5; 9800301-04 pp. 3-32/3-33.
    Encoding::new(b"00111010", 3, States::single(13)),
    // STA addr: 205775-002 p. 5-5; 9800301-04 pp. 3-61/3-62.
    Encoding::new(b"00110010", 3, States::single(13)),
    // LHLD addr: 205775-002 p. 5-5; 9800301-04 p. 3-34.
    Encoding::new(b"00101010", 3, States::single(16)),
    // SHLD addr: 205775-002 p. 5-5; 9800301-04 pp. 3-58/3-59.
    Encoding::new(b"00100010", 3, States::single(16)),
    // LDAX B: 205775-002 p. 5-5; 9800301-04 p. 3-33.
    // 231718-001 Table 6 p. 16-28 misprints "LDAX B" as "00101010".
    // Use "00RP1010", RP=00 here, from 205775-002 pp. 5-1/5-5;
    // Tables 5-1/5-2 pp. 5-19/5-20 and 9800301-04 p. 3-33 agree on 0A.
    Encoding::new(b"00001010", 1, States::single(7)),
    // LDAX D: 205775-002 p. 5-5; 9800301-04 p. 3-33.
    Encoding::new(b"00011010", 1, States::single(7)),
    // STAX B: 205775-002 p. 5-6; 9800301-04 p. 3-62.
    Encoding::new(b"00000010", 1, States::single(7)),
    // STAX D: 205775-002 p. 5-6; 9800301-04 p. 3-62.
    Encoding::new(b"00010010", 1, States::single(7)),
    // XCHG: 205775-002 p. 5-6; 9800301-04 p. 3-65.
    Encoding::new(b"11101011", 1, States::single(4)),
];

/// Executes an opcode already fetched and dispatched to this group.
/// The opcode map guarantees that the byte belongs to this group.
pub(super) fn execute<S: Bus>(
    cpu: &mut I8085,
    system: &mut S,
    opcode: u8,
    instruction: Instruction,
) -> Path {
    let registers = cpu.registers_mut();
    match opcode {
        // MOV: §5.6.1 p. 5-4; 9800301-04 pp. 3-36/3-37.
        // Table 5-1 p. 5-19: F, FR or FW. Read original HL before
        // changing H/L, and read the source before writing the destination.
        0x40..=0x7F if opcode != 0x76 => {
            let value = read_location(registers, system, opcode & 7);
            write_location(registers, system, (opcode >> 3) & 7, value);
        }
        // MVI: p. 5-4; 9800301-04 pp. 3-37/3-38. Table 5-1:
        // FR for a register, FRW for memory; byte 2 precedes the HL write.
        0x06 | 0x0E | 0x16 | 0x1E | 0x26 | 0x2E | 0x36 | 0x3E => {
            let value = operand(registers, system, 1);
            write_location(registers, system, (opcode >> 3) & 7, value);
        }
        // LXI: pp. 5-1/5-5; 9800301-04 p. 3-35. Table 5-1: FRR.
        // Byte 2 is low, byte 3 high; RP selects BC/DE/HL/SP.
        0x01 | 0x11 | 0x21 | 0x31 => {
            let value = operand_word(registers, system);
            match opcode {
                0x01 => registers.set_bc(value),
                0x11 => registers.set_de(value),
                0x21 => registers.set_hl(value),
                _ => registers.sp = value,
            }
        }
        // Direct transfers: p. 5-5; 9800301-04 pp. 3-32 to 3-34,
        // 3-58/3-59, 3-61/3-62. Table 5-1: FRRR/FRRW/FRRRR/FRRWW.
        // Fetch the entire address before accessing data, even on overlap.
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
            // Address+1 is p. 5-5; FFFFh -> 0000h is DERIVED from the
            // 16-bit address width (§5.3 p. 5-2), not explicitly verified.
        }
        // LDAX/STAX: pp. 5-5/5-6; 9800301-04 pp. 3-33/3-62.
        // Table 5-1: FR/FW; only B or D pairs select the data address.
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
        // XCHG: p. 5-6; 9800301-04 p. 3-65. Table 5-1: F only.
        0xEB => {
            core::mem::swap(&mut registers.h, &mut registers.d);
            core::mem::swap(&mut registers.l, &mut registers.e);
        }
        _ => unreachable!("opcode map dispatches only this group's encodings"),
    }
    // §2.2.1 p. 2-2: PC points at the next instruction; §5.3 p. 5-2:
    // successive instruction bytes. PC wrap is DERIVED from its 16-bit
    // width; neither page explicitly verifies FFFFh -> 0000h.
    // Flags remain untouched (§5.6.1 p. 5-4). These instructions have one
    // total in both state slots (§5.2 p. 5-2; decision-5 item 7).
    registers.pc = registers.pc.wrapping_add(u16::from(instruction.length));
    Path::Taken
}

// §5.3 p. 5-2: bytes follow the opcode at successive addresses. Table 5-1
// p. 5-19 and Ch. 2 pp. 2-9/2-12: OF then MR. Access kind: decision-5
// item 3. PC+offset wrap is DERIVED, as described above.
fn operand<S: Bus>(registers: &Registers, system: &mut S, offset: u16) -> u8 {
    system.read(
        registers.pc.wrapping_add(offset),
        MemoryAccess::OperandFetch,
    )
}

// §5.6.1 p. 5-5: byte 2 low, byte 3 high. Table 5-1 orders both reads
// before any data cycle. Design: decision-5 item 3, plan §2.6.
fn operand_word<S: Bus>(registers: &Registers, system: &mut S) -> u16 {
    let low = operand(registers, system, 1);
    let high = operand(registers, system, 2);
    u16::from_le_bytes([low, high])
}

// §5.2 p. 5-1 register codes; §5.6.1 p. 5-4: M addresses HL.
// Table 5-1 p. 5-19: FR. Callers supply masked three-bit fields, with
// 111 selecting A. Access kind: decision-5 item 3.
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

// Same field/address rules; Table 5-1 p. 5-19: FW or FRW.
// Design: decision-5 item 3, plan §2.2/§2.6.
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
