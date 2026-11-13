//! Call and Return Group: encodings and execution (decision-13 rules 2–5).
//!
//! Primary: Zilog UM008011-0816, rev. 11 (August 2016), pp. 280–293.
//! Entry citations below use printed pages; PDF = printed + 14.
//! Independent cross-read: 03-0029-01 (1977), Table 7.0-10, p. 53
//! (PDF = printed + 6). Whole-instruction lengths and T-state totals include
//! every prefix, operand and displacement (decision-13 rules 3–5).

use super::cpu::{Decoded, Table};
use super::opcodes::{Encoding, Instruction, States};
use super::{Flags, Z80};
use crate::decode::Path;
use crate::{Bus, MemoryAccess};

pub(super) const MAIN: &[Encoding] = &[
    // UM CALL nn, p. 281.
    Encoding::new(b"11001101", 3, States::single(17)),
    // UM CALL cc,nn, pp. 283–284.
    Encoding::new(
        b"11CCC100",
        3,
        States {
            untaken: 10,
            taken: 17,
        },
    ),
    // UM RET, p. 285.
    Encoding::new(b"11001001", 1, States::single(10)),
    // UM RET cc, pp. 286–287.
    Encoding::new(
        b"11CCC000",
        1,
        States {
            untaken: 5,
            taken: 11,
        },
    ),
    // UM RST p, pp. 292–293.
    Encoding::new(b"11TTT111", 1, States::single(11)),
];

pub(super) const ED: &[Encoding] = &[
    // UM RETI, p. 288.
    Encoding::new(b"01001101", 2, States::single(14)),
    // UM RETN, p. 290.
    Encoding::new(b"01000101", 2, States::single(14)),
];

/// PC is already after the opcode/prefix bytes (decision-13 rule 4).
/// Every instruction preserves flags (UM pp. 281–290; TM Table 7.0-10 p. 53).
pub(super) fn execute<S: Bus>(
    cpu: &mut Z80,
    system: &mut S,
    _instruction: &Instruction,
    decoded: &Decoded,
) -> Path {
    let opcode = decoded.opcode;
    if decoded.table == Table::Ed {
        // UM RETI p. 288 / RETN p. 290; TM p. 53. Decision-15 rule 7
        // keeps RETI's IFFs unchanged: MK78515 V2.0 (October 1978),
        // Table 2-1 p. 2-15 (PDF 29), explicitly distinguishes it from RETN.
        pop_pc(cpu, system);
        if opcode == 0x45 {
            cpu.latches_mut().iff1 = cpu.latches().iff2;
        }
        return Path::Taken;
    }

    match opcode {
        // UM CALL pp. 281–284; TM p. 53. Both paths read low then high:
        // untaken has three cycles, 10 (4,3,3); taken saves PC after nn.
        0xCD | 0xC4 | 0xCC | 0xD4 | 0xDC | 0xE4 | 0xEC | 0xF4 | 0xFC => {
            let low = cpu.fetch_operand(system);
            let high = cpu.fetch_operand(system);
            if opcode == 0xCD || condition(cpu.registers().flags, opcode) {
                let return_pc = cpu.registers().pc;
                push_pc_and_jump(cpu, system, return_pc, u16::from_le_bytes([low, high]));
                Path::Taken
            } else {
                Path::Untaken
            }
        }
        // UM RET pp. 285–287; TM p. 53. Untaken has no stack reads;
        // PC already names the next instruction. RET cc costs 5/11.
        0xC9 | 0xC0 | 0xC8 | 0xD0 | 0xD8 | 0xE0 | 0xE8 | 0xF0 | 0xF8 => {
            if opcode == 0xC9 || condition(cpu.registers().flags, opcode) {
                pop_pc(cpu, system);
                Path::Taken
            } else {
                Path::Untaken
            }
        }
        // The map guarantees the remaining main entries are RST p.
        // UM pp. 292–293; TM p. 53: p = 00,08,...,38, save next PC.
        _ => {
            let return_pc = cpu.registers().pc;
            push_pc_and_jump(cpu, system, return_pc, u16::from(opcode & 0x38));
            Path::Taken
        }
    }
}

/// CCC bits 5..3: UM pp. 284/286; TM Table 7.0-10 p. 53.
/// UM p. 284 mislabels Carry's flag as Z; p. 286 and TM identify carry
/// (PROVENANCE.md §1). H/N are neither tested nor changed.
fn condition(flags: Flags, opcode: u8) -> bool {
    match (opcode >> 3) & 7 {
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

/// UM RET p. 285 and RETN example p. 290: read low then high, advancing
/// SP after each byte. Second witness: TM Table 7.0-10 p. 53.
/// Stack is the API mapping (decision-15 rule 7); 16-bit SP wrapping is
/// derived from UM Special-Purpose Registers p. 2 (TM §2.1 p. 3),
/// not an explicit wrap statement.
fn pop_pc(cpu: &mut Z80, system: &mut impl Bus) {
    let registers = cpu.registers_mut();
    let low = system.read(registers.sp, MemoryAccess::Stack);
    registers.sp = registers.sp.wrapping_add(1);
    let high = system.read(registers.sp, MemoryAccess::Stack);
    registers.sp = registers.sp.wrapping_add(1);
    registers.pc = u16::from_le_bytes([low, high]);
}

/// Pushes the supplied PC high byte first, then loads `target` without
/// advancing PC or changing flags/latches, fetching bytes or accounting R/time.
/// CALL/RST supply their next PC; NMI/INT supply the boundary PC
/// (decision-15 rules 8–9). This is the same stack operation for all callers.
/// UM CALL p. 281, RETN example p. 290 and RST p. 292 explicitly order
/// decrement SP, write PCH, decrement SP, write PCL; TM Table 7.0-10 p. 53
/// agrees on the layout. Stack is the API mapping. Wrapping is derived
/// from the 16-bit SP width (UM Special-Purpose Registers p. 2;
/// TM §2.1 p. 3).
pub(super) fn push_pc_and_jump(cpu: &mut Z80, system: &mut impl Bus, return_pc: u16, target: u16) {
    let registers = cpu.registers_mut();
    let [high, low] = return_pc.to_be_bytes();
    registers.sp = registers.sp.wrapping_sub(1);
    system.write(registers.sp, high, MemoryAccess::Stack);
    registers.sp = registers.sp.wrapping_sub(1);
    system.write(registers.sp, low, MemoryAccess::Stack);
    registers.pc = target;
}
