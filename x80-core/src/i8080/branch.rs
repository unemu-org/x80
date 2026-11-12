//! The 8080 Branch Group: its encodings and its executor.
//!
//! Design: plan §2.2/2.3, decision-5 items 3 and 8, decision-6 (8080 only).
//! Primary source: Intel 98-153B (September 1975), Ch. 4 "Branch Group",
//! pp. 4-11/4-12; Ch. 2 machine-cycle table pp. 2-18/2-19 and notes p. 2-20.
//! Second witness: Intel 9800301-04 (May 1981), Ch. 3 instruction entries.
//! All cited values were checked against the page images.

use super::opcodes::{Encoding, Instruction, Path, States};
use super::{Flags, I8080};
use crate::{Bus, MemoryAccess};

/// The group's encodings and per-entry state totals, in page order.
pub(super) const ENCODINGS: &[Encoding] = &[
    // 98-153B Ch. 4 JMP p. 4-11: 10 states; 9800301-04 Ch. 3 p. 3-28.
    Encoding::new(b"11000011", 3).with_states(States::single(10)),
    // Jcondition p. 4-11: 10 states on either path; second witness Ch. 3
    // Jcc pp. 3-26..3-32 (8080 totals, not the different 8085 totals).
    Encoding::new(b"11CCC010", 3).with_states(States {
        untaken: 10,
        taken: 10,
    }),
    // CALL p. 4-11: 17 states; second witness Ch. 3 pp. 3-8/3-9.
    Encoding::new(b"11001101", 3).with_states(States::single(17)),
    // Ccondition p. 4-12: 11/17 states; second witness Ch. 3 pp. 3-10/
    // 3-11, 3-14..3-18. 98-153B p. 4-15 note 2: flag-dependent totals.
    Encoding::new(b"11CCC100", 3).with_states(States {
        untaken: 11,
        taken: 17,
    }),
    // RET p. 4-12: 10 states; second witness Ch. 3 p. 3-48.
    Encoding::new(b"11001001", 1).with_states(States::single(10)),
    // Rcondition p. 4-12: 5/11 states; second witness Ch. 3 pp. 3-47,
    // 3-50..3-53, 3-55/3-56.
    Encoding::new(b"11CCC000", 1).with_states(States {
        untaken: 5,
        taken: 11,
    }),
    // RST n p. 4-12: 11 states; second witness Ch. 3 pp. 3-54/3-55.
    Encoding::new(b"11NNN111", 1).with_states(States::single(11)),
    // PCHL p. 4-12: 5 states; second witness Ch. 3 p. 3-42.
    Encoding::new(b"11101001", 1).with_states(States::single(5)),
];

/// Executes a Branch Group opcode after its opcode fetch (decision-5 item 3).
/// No branch changes flags (98-153B Ch. 4 "Branch Group", p. 4-11).
/// An unrecognised opcode returns `None` without changing or accessing anything.
pub(super) fn execute<S: Bus>(
    cpu: &mut I8080,
    system: &mut S,
    opcode: u8,
    _instruction: Instruction,
) -> Option<Path> {
    match opcode {
        // JMP / Jcondition: 98-153B Ch. 4 p. 4-11. Ch. 2 pp. 2-18/2-19:
        // M2 reads B2 into Z, M3 reads B3 into W on BOTH paths. Note 12,
        // p. 2-20 changes only the following fetch address when true.
        0xC3 | 0xC2 | 0xCA | 0xD2 | 0xDA | 0xE2 | 0xEA | 0xF2 | 0xFA => {
            let taken = opcode == 0xC3 || condition(cpu.registers().flags, opcode);
            let target = read_address(cpu, system);
            if taken {
                cpu.registers_mut().pc = target;
                Some(Path::Taken)
            } else {
                Some(Path::Untaken)
            }
        }
        // CALL / Ccondition: 98-153B Ch. 4 pp. 4-11/4-12. Ch. 2 pp. 2-18/
        // 2-19 M2/M3 read both operands even if false: note 13 p. 2-20 skips
        // ONLY M4/M5 (the stack writes). Push the next instruction's PC.
        0xCD | 0xC4 | 0xCC | 0xD4 | 0xDC | 0xE4 | 0xEC | 0xF4 | 0xFC => {
            let taken = opcode == 0xCD || condition(cpu.registers().flags, opcode);
            let target = read_address(cpu, system);
            if taken {
                let return_pc = cpu.registers().pc;
                push_and_jump(cpu, system, return_pc, target);
                Some(Path::Taken)
            } else {
                Some(Path::Untaken)
            }
        }
        // RET / Rcondition: 98-153B Ch. 4 p. 4-12. Ch. 2 pp. 2-18/2-19:
        // M2 reads low at SP, M3 high at SP+1, incrementing SP each time;
        // note 15 p. 2-20 marks both Stack. Note 14 skips M2/M3 if false.
        // Increments wrap to the 16-bit SP width (Ch. 2 p. 2-2).
        0xC9 | 0xC0 | 0xC8 | 0xD0 | 0xD8 | 0xE0 | 0xE8 | 0xF0 | 0xF8 => {
            if opcode == 0xC9 || condition(cpu.registers().flags, opcode) {
                let registers = cpu.registers_mut();
                let low = system.read(registers.sp, MemoryAccess::Stack);
                registers.sp = registers.sp.wrapping_add(1);
                let high = system.read(registers.sp, MemoryAccess::Stack);
                registers.sp = registers.sp.wrapping_add(1);
                registers.pc = u16::from_le_bytes([low, high]);
                Some(Path::Taken)
            } else {
                // M1 increments PC (Ch. 2 p. 2-18); no operand bytes.
                cpu.registers_mut().pc = cpu.registers().pc.wrapping_add(1);
                Some(Path::Untaken)
            }
        }
        // RST: 98-153B Ch. 4 p. 4-12, PC <- 8*NNN (bits 5..3), push the
        // next instruction's address. Ch. 2 pp. 2-18/2-19 M2/M3 stack writes.
        0xC7 | 0xCF | 0xD7 | 0xDF | 0xE7 | 0xEF | 0xF7 | 0xFF => {
            let return_pc = cpu.registers().pc.wrapping_add(1);
            restart(cpu, system, opcode, return_pc);
            Some(Path::Taken)
        }
        // PCHL: 98-153B Ch. 4 p. 4-12 PCH <- H, PCL <- L; Ch. 2 p. 2-18
        // only M1, an internal HL -> PC transfer (no read at address HL).
        0xE9 => {
            cpu.registers_mut().pc = cpu.registers().hl();
            Some(Path::Taken)
        }
        _ => None,
    }
}

/// CCC in bits 5..3: 98-153B Ch. 4 condition table p. 4-11, repeated in
/// Ch. 2 note 17 p. 2-20. AC is never examined or changed.
fn condition(flags: Flags, opcode: u8) -> bool {
    match (opcode >> 3) & 7 {
        0 => !flags.zero,
        1 => flags.zero,
        2 => !flags.carry,
        3 => flags.carry,
        4 => !flags.parity,
        5 => flags.parity,
        6 => !flags.sign,
        _ => flags.sign,
    }
}

/// Reads B2 then B3 after M1, leaving PC at the next instruction: 98-153B
/// Ch. 2 pp. 2-18/2-19 JMP/Jcc/CALL/Ccc M1-M3, note 6 p. 2-20 memory reads.
/// `OperandFetch` is the API distinction of decision-5 item 3. PC is 16 bits
/// (Ch. 2 p. 2-2); wrapping increments derive from that width.
fn read_address<S: Bus>(cpu: &mut I8080, system: &mut S) -> u16 {
    let registers = cpu.registers_mut();
    registers.pc = registers.pc.wrapping_add(1);
    let low = system.read(registers.pc, MemoryAccess::OperandFetch);
    registers.pc = registers.pc.wrapping_add(1);
    let high = system.read(registers.pc, MemoryAccess::OperandFetch);
    registers.pc = registers.pc.wrapping_add(1);
    u16::from_le_bytes([low, high])
}

/// RST's push/jump, shared with interrupt acceptance inside the model
/// (decision-5 item 4). `return_pc` is explicit: a normal fetch increments PC,
/// an acknowledge does not (98-153B Ch. 2 "Interrupt Sequences", p. 2-11).
/// The caller supplies a documented RST opcode; bits 5..3 select 8*NNN
/// (98-153B Ch. 4 RST p. 4-12).
pub(super) fn restart<S: Bus>(cpu: &mut I8080, system: &mut S, opcode: u8, return_pc: u16) {
    push_and_jump(cpu, system, return_pc, u16::from(opcode & 0x38));
}

/// 98-153B Ch. 4 CALL/RST pp. 4-11/4-12: high byte at SP-1, low at SP-2,
/// final SP -= 2, then jump. Ch. 2 pp. 2-18/2-19 fixes the write order:
/// CALL M4 high/M5 low, RST M2 high/M3 low; note 16 p. 2-20 marks Stack
/// (decision-5 item 3). SP is 16 bits (p. 2-2); wrapping derives from its
/// width, rather than an explicit description of a push across zero.
fn push_and_jump<S: Bus>(cpu: &mut I8080, system: &mut S, return_pc: u16, target: u16) {
    let registers = cpu.registers_mut();
    let [high, low] = return_pc.to_be_bytes();
    registers.sp = registers.sp.wrapping_sub(1);
    system.write(registers.sp, high, MemoryAccess::Stack);
    registers.sp = registers.sp.wrapping_sub(1);
    system.write(registers.sp, low, MemoryAccess::Stack);
    registers.pc = target;
}
