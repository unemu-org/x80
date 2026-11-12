//! The 8085 Branch encodings and executor.
//!
//! Intel 205775-002 (January 1983), §5.6.4, pp. 5-13 to 5-15, checked on page images.
//! Totals below are the 8085 states, not machine cycles or 8080 states.
//! Independent witnesses: `tests/i8085_branch.rs` and `opcodes/tests.rs`.
//! Intel 9800301-04 (May 1981), Ch. 3 instruction entries below, supplies
//! the second readings. Source hashes match PROVENANCE.md §1.
//! Table 5-1 p. 5-19 (PDF 104) gives cycle sequences; Table 5-2 p. 5-20
//! (PDF 105) gives the opcode list. Ch. 2 §2.3.1 p. 2-7 (PDF 30) describes
//! stream reads; §2.2.3 p. 2-2 (PDF 25) the stack; Fig. 2-18 p. 2-14
//! (PDF 37) fixes high-then-low stack writes.
//! Design: plan §2.2/2.3; decisions 10/11; decision-5 item 3 and
//! decision-12 rules 4/5 (explicit return PC/vector for later interrupts).

use super::opcodes::{Encoding, Instruction, Path, States};
use super::{Flags, I8085};
use crate::{Bus, MemoryAccess};

/// Complete group list, in per-instruction page order.
/// CCC takes all eight conditions (205775-002 p. 5-13); NNN takes
/// all eight restarts (p. 5-14). Costs are untaken/taken respectively;
/// 9800301-04 p. A-1 explains that order. Its CPE 9/17 disagrees with
/// this primary source's 9/18, pinned in `opcodes/tests.rs`.
pub(super) const ENCODINGS: &[Encoding] = &[
    // JMP addr: 205775-002 p. 5-13.
    Encoding::new(b"11000011", 3, States::single(10)),
    // Jcondition addr: 205775-002 p. 5-13.
    Encoding::new(
        b"11CCC010",
        3,
        States {
            untaken: 7,
            taken: 10,
        },
    ),
    // CALL addr: 205775-002 p. 5-13.
    Encoding::new(b"11001101", 3, States::single(18)),
    // Ccondition addr: 205775-002 p. 5-14.
    Encoding::new(
        b"11CCC100",
        3,
        States {
            untaken: 9,
            taken: 18,
        },
    ),
    // RET: 205775-002 p. 5-14.
    Encoding::new(b"11001001", 1, States::single(10)),
    // Rcondition: 205775-002 p. 5-14.
    Encoding::new(
        b"11CCC000",
        1,
        States {
            untaken: 6,
            taken: 12,
        },
    ),
    // RST n: 205775-002 p. 5-14.
    Encoding::new(b"11NNN111", 1, States::single(12)),
    // PCHL: 205775-002 p. 5-15.
    Encoding::new(b"11101001", 1, States::single(6)),
];

/// Executes after opcode fetch; every listed encoding returns a path.
/// No instruction changes flags (205775-002 §5.6.4 p. 5-13).
/// The opcode map guarantees that the byte belongs to this group.
pub(super) fn execute<S: Bus>(
    cpu: &mut I8085,
    system: &mut S,
    opcode: u8,
    _instruction: Instruction,
) -> Path {
    let pc = cpu.registers().pc;
    match opcode {
        // JMP/Jcond: 205775-002 p. 5-13 (PDF 98); 9800301-04 Ch. 3
        // pp. 3-26..3-32 (PDF 80..86). Table 5-1: F R / F R R for Jcond,
        // F R R for JMP. Untaken skips only the high operand read.
        0xC3 | 0xC2 | 0xCA | 0xD2 | 0xDA | 0xE2 | 0xEA | 0xF2 | 0xFA => {
            if opcode == 0xC3 || condition(cpu.registers().flags, opcode) {
                cpu.registers_mut().pc = read_address(system, pc);
                Path::Taken
            } else {
                skip_address(cpu, system);
                Path::Untaken
            }
        }
        // CALL/Ccond: 205775-002 pp. 5-13/5-14 (PDF 98/99);
        // 9800301-04 pp. 3-8..3-11, 3-14..3-18 (PDF 62..65, 68..72).
        // Table 5-1: S R R W W taken, S R untaken; bullet footnote's
        // extra untaken read applies to the 8080A only. Return PC is PC+3.
        0xCD | 0xC4 | 0xCC | 0xD4 | 0xDC | 0xE4 | 0xEC | 0xF4 | 0xFC => {
            if opcode == 0xCD || condition(cpu.registers().flags, opcode) {
                let target = read_address(system, pc);
                restart(cpu, system, pc.wrapping_add(3), target);
                Path::Taken
            } else {
                skip_address(cpu, system);
                Path::Untaken
            }
        }
        // RET/Rcond: 205775-002 p. 5-14 (PDF 99); 9800301-04 pp. 3-47/
        // 3-48, 3-50..3-53, 3-55/3-56 (PDF 101/102, 104..107, 109/110).
        // Table 5-1: RET F R R, Rcond S / S R R. Pop low then high.
        0xC9 | 0xC0 | 0xC8 | 0xD0 | 0xD8 | 0xE0 | 0xE8 | 0xF0 | 0xF8 => {
            if opcode == 0xC9 || condition(cpu.registers().flags, opcode) {
                let registers = cpu.registers_mut();
                let low = system.read(registers.sp, MemoryAccess::Stack);
                registers.sp = registers.sp.wrapping_add(1);
                let high = system.read(registers.sp, MemoryAccess::Stack);
                registers.sp = registers.sp.wrapping_add(1);
                registers.pc = u16::from_le_bytes([low, high]);
                Path::Taken
            } else {
                cpu.registers_mut().pc = pc.wrapping_add(1);
                Path::Untaken
            }
        }
        // RST: 205775-002 p. 5-14 (PDF 99); 9800301-04 pp. 3-54/3-55
        // (PDF 108/109): save next PC, vector = 8*NNN. Table 5-1: S W W.
        0xC7 | 0xCF | 0xD7 | 0xDF | 0xE7 | 0xEF | 0xF7 | 0xFF => {
            restart(cpu, system, pc.wrapping_add(1), u16::from(opcode & 0x38));
            Path::Taken
        }
        // PCHL: 205775-002 p. 5-15 (PDF 100); 9800301-04 p. 3-42
        // (PDF 96): PCH=H, PCL=L. Table 5-1: S, no memory operand.
        0xE9 => {
            cpu.registers_mut().pc = cpu.registers().hl();
            Path::Taken
        }
        _ => unreachable!("opcode map dispatches only this group's encodings"),
    }
}

/// CCC bits 5..3: 205775-002 condition table p. 5-13; second witness
/// 9800301-04 Jcond entries pp. 3-26..3-32. AC is not tested or changed.
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

/// Ch. 2 §2.3.1 p. 2-7: M2 reads byte 2 (low), M3 byte 3 (high).
/// Table 5-1 p. 5-19 supplies both R cycles for JMP/taken Jcond/CALL/Ccond.
/// PC wrap is derived from the 16-bit width (p. 2-2), not explicitly stated.
fn read_address<S: Bus>(system: &mut S, pc: u16) -> u16 {
    let low = system.read(pc.wrapping_add(1), MemoryAccess::OperandFetch);
    let high = system.read(pc.wrapping_add(2), MemoryAccess::OperandFetch);
    u16::from_le_bytes([low, high])
}

/// Untaken Jcond/Ccond: Table 5-1 p. 5-19 has just one R after fetch;
/// Ch. 2 p. 2-7 identifies it as byte 2. Skip all three bytes (pp. 5-13/
/// 5-14 sequential path); 9800301-04 pp. 3-10, 3-27 confirm two cycles.
fn skip_address<S: Bus>(cpu: &mut I8085, system: &mut S) {
    let pc = cpu.registers().pc;
    system.read(pc.wrapping_add(1), MemoryAccess::OperandFetch);
    cpu.registers_mut().pc = pc.wrapping_add(3);
}

/// Pushes the supplied return PC and loads the supplied vector, without
/// advancing PC. Used by RST/CALL here; later interrupt callers supply their
/// boundary PC (decision-12 rules 4/5), without changing this stack operation.
/// 205775-002 pp. 5-13/5-14: high at SP-1, low at SP-2; Fig. 2-18 p. 2-14
/// fixes that order. Table 5-1: CALL's M4/M5, RST's M2/M3 are memory writes;
/// they address the stack (Ch. 2 §2.2.3 p. 2-2; decision-5 item 3).
/// Second witness: 9800301-04 pp. 3-8/3-9, 3-54/3-55. SP wrap is derived
/// from its 16-bit width (205775-002 p. 2-2), not an explicit wrap description.
pub(super) fn restart<S: Bus>(cpu: &mut I8085, system: &mut S, return_pc: u16, vector: u16) {
    let registers = cpu.registers_mut();
    let [high, low] = return_pc.to_be_bytes();
    registers.sp = registers.sp.wrapping_sub(1);
    system.write(registers.sp, high, MemoryAccess::Stack);
    registers.sp = registers.sp.wrapping_sub(1);
    system.write(registers.sp, low, MemoryAccess::Stack);
    registers.pc = vector;
}
