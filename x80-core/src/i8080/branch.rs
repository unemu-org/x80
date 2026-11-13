//! The 8080 Branch Group: its encodings and its executor.
//!
//! Design: plan §2.2/2.3, decision-5 items 3 and 8, decision-6 (8080 only).
//! Primary source: Intel 98-153B (September 1975), Ch. 4 "Branch Group",
//! pp. 4-11/4-12; Ch. 2 machine-cycle table pp. 2-18/2-19 and notes p. 2-20.
//! Second witness: Intel 9800301-04 (May 1981), Ch. 3 instruction entries.
//! All cited values were checked against the page images.

use super::opcodes::{Encoding, Instruction, Path, States};
use super::I8080;
use crate::intel::access::{condition, push_and_jump, read_address};
use crate::{Bus, MemoryAccess};

/// The group's encodings and per-entry state totals, in page order.
pub(super) const ENCODINGS: &[Encoding] = &[
    // 98-153B Ch. 4 JMP p. 4-11: 10 states; 9800301-04 Ch. 3 p. 3-28.
    Encoding::new(b"11000011", 3, States::single(10)),
    // Jcondition p. 4-11: 10 states on either path; second witness Ch. 3
    // Jcc pp. 3-26..3-32 (8080 totals, not the different 8085 totals).
    Encoding::new(
        b"11CCC010",
        3,
        States {
            untaken: 10,
            taken: 10,
        },
    ),
    // CALL p. 4-11: 17 states; second witness Ch. 3 pp. 3-8/3-9.
    Encoding::new(b"11001101", 3, States::single(17)),
    // Ccondition p. 4-12: 11/17 states; second witness Ch. 3 pp. 3-10/
    // 3-11, 3-14..3-18. 98-153B p. 4-15 note 2: flag-dependent totals.
    Encoding::new(
        b"11CCC100",
        3,
        States {
            untaken: 11,
            taken: 17,
        },
    ),
    // RET p. 4-12: 10 states; second witness Ch. 3 p. 3-48.
    Encoding::new(b"11001001", 1, States::single(10)),
    // Rcondition p. 4-12: 5/11 states; second witness Ch. 3 pp. 3-47,
    // 3-50..3-53, 3-55/3-56.
    Encoding::new(
        b"11CCC000",
        1,
        States {
            untaken: 5,
            taken: 11,
        },
    ),
    // RST n p. 4-12: 11 states; second witness Ch. 3 pp. 3-54/3-55.
    Encoding::new(b"11NNN111", 1, States::single(11)),
    // PCHL p. 4-12: 5 states; second witness Ch. 3 p. 3-42.
    Encoding::new(b"11101001", 1, States::single(5)),
];

/// Executes a Branch Group opcode after its opcode fetch (decision-5 item 3).
/// No branch changes flags (98-153B Ch. 4 "Branch Group", p. 4-11).
/// The opcode map guarantees that `opcode` belongs to this group.
pub(super) fn execute<S: Bus>(
    cpu: &mut I8080,
    system: &mut S,
    opcode: u8,
    _instruction: Instruction,
) -> Path {
    match opcode {
        // JMP / Jcondition: 98-153B Ch. 4 p. 4-11. Ch. 2 pp. 2-18/2-19:
        // M2 reads B2 into Z, M3 reads B3 into W on BOTH paths. Note 12,
        // p. 2-20 changes only the following fetch address when true.
        0xC3 | 0xC2 | 0xCA | 0xD2 | 0xDA | 0xE2 | 0xEA | 0xF2 | 0xFA => {
            let taken =
                opcode == 0xC3 || condition(&super::view(cpu.registers_mut()), (opcode >> 3) & 7);
            let target = read_address(system, cpu.registers().pc);
            cpu.registers_mut().pc = cpu.registers().pc.wrapping_add(3);
            if taken {
                cpu.registers_mut().pc = target;
                Path::Taken
            } else {
                Path::Untaken
            }
        }
        // CALL / Ccondition: 98-153B Ch. 4 pp. 4-11/4-12. Ch. 2 pp. 2-18/
        // 2-19 M2/M3 read both operands even if false: note 13 p. 2-20 skips
        // ONLY M4/M5 (the stack writes). Push the next instruction's PC.
        0xCD | 0xC4 | 0xCC | 0xD4 | 0xDC | 0xE4 | 0xEC | 0xF4 | 0xFC => {
            let taken =
                opcode == 0xCD || condition(&super::view(cpu.registers_mut()), (opcode >> 3) & 7);
            let target = read_address(system, cpu.registers().pc);
            cpu.registers_mut().pc = cpu.registers().pc.wrapping_add(3);
            if taken {
                let return_pc = cpu.registers().pc;
                push_and_jump(
                    &mut super::view(cpu.registers_mut()),
                    system,
                    return_pc,
                    target,
                );
                Path::Taken
            } else {
                Path::Untaken
            }
        }
        // RET / Rcondition: 98-153B Ch. 4 p. 4-12. Ch. 2 pp. 2-18/2-19:
        // M2 reads low at SP, M3 high at SP+1, incrementing SP each time;
        // note 15 p. 2-20 marks both Stack. Note 14 skips M2/M3 if false.
        // Increments wrap to the 16-bit SP width (Ch. 2 p. 2-2).
        0xC9 | 0xC0 | 0xC8 | 0xD0 | 0xD8 | 0xE0 | 0xE8 | 0xF0 | 0xF8 => {
            if opcode == 0xC9 || condition(&super::view(cpu.registers_mut()), (opcode >> 3) & 7) {
                let registers = cpu.registers_mut();
                let low = system.read(registers.sp, MemoryAccess::Stack);
                registers.sp = registers.sp.wrapping_add(1);
                let high = system.read(registers.sp, MemoryAccess::Stack);
                registers.sp = registers.sp.wrapping_add(1);
                registers.pc = u16::from_le_bytes([low, high]);
                Path::Taken
            } else {
                // M1 increments PC (Ch. 2 p. 2-18); no operand bytes.
                cpu.registers_mut().pc = cpu.registers().pc.wrapping_add(1);
                Path::Untaken
            }
        }
        // RST: 98-153B Ch. 4 p. 4-12, PC <- 8*NNN (bits 5..3), push the
        // next instruction's address. Ch. 2 pp. 2-18/2-19 M2/M3 stack writes.
        0xC7 | 0xCF | 0xD7 | 0xDF | 0xE7 | 0xEF | 0xF7 | 0xFF => {
            let return_pc = cpu.registers().pc.wrapping_add(1);
            push_and_jump(
                &mut super::view(cpu.registers_mut()),
                system,
                return_pc,
                u16::from(opcode & 0x38),
            );
            Path::Taken
        }
        // PCHL: 98-153B Ch. 4 p. 4-12 PCH <- H, PCL <- L; Ch. 2 p. 2-18
        // only M1, an internal HL -> PC transfer (no read at address HL).
        0xE9 => {
            cpu.registers_mut().pc = cpu.registers().hl();
            Path::Taken
        }
        _ => unreachable!("opcode map dispatches only this group's encodings"),
    }
}
