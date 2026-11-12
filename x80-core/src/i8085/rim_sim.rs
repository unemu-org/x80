//! The 8085 RIM/SIM encodings and executor (decision-10; decision-12 rule 8).
//!
//! Intel 205775-002 (January 1983), §5.6.5 pp. 5-17/5-18 (PDF 102/103),
//! Table 5-1 p. 5-19 (PDF 104), Table 5-2 p. 5-20 (PDF 105), §2.3.8
//! p. 2-19 (PDF 42). Independent source: Intel 9800301-04 (May 1981),
//! RIM p. 3-49 (PDF 103), SIM p. 3-60 (PDF 114). Values checked on page
//! images; revisions and hashes pinned in PROVENANCE.md §1.
//!
//! Both are one-byte, four-state instructions with no flag effects. Table
//! 5-1 gives only F (four-clock instruction fetch); Ch. 2 p. 2-9 describes
//! that read at PC. No operand, data, stack or port bus access follows.
//! RIM's line reads and SIM's output notification follow decision-12 rule 8.
//! Independent execution witnesses: `tests/i8085_rim_sim.rs`.

use super::opcodes::{Encoding, Instruction, Path, States};
use super::I8085;
use crate::Bus;

/// RIM and SIM, separated from stack/control by decision-10.
pub(super) const ENCODINGS: &[Encoding] = &[
    // RIM: 205775-002 p. 5-17.
    Encoding::new(b"00100000", 1, States::single(4)),
    // SIM: 205775-002 p. 5-18.
    Encoding::new(b"00110000", 1, States::single(4)),
];

/// Executes RIM or SIM. A byte outside this group has no effects.
pub(super) fn execute<S: Bus + super::Inputs>(
    cpu: &mut I8085,
    system: &mut S,
    opcode: u8,
    instruction: Instruction,
) -> Option<Path> {
    match opcode {
        // 205775-002 p. 5-17, RIM; 9800301-04 p. 3-49: A bits 7..0
        // are SID, I7.5, I6.5, I5.5, IE, M7.5, M6.5, M5.5.
        // §2.3.8 p. 2-19 transfers pin levels during RIM; decision-12
        // rule 8 fixes SID, 6.5, 5.5 read order. I7.5 is the flip-flop,
        // even when masked (p. 5-17; rule 2), not another pin read.
        0x20 => {
            let sid = system.sid();
            let rst6_5 = system.rst6_5();
            let rst5_5 = system.rst5_5();
            let latches = cpu.latches_mut();
            // The first RIM after TRAP uses the saved enable, once:
            // p. 5-17; 231718-001 (September 1987), handbook p. 16-15.
            // UNVERIFIED during EI delay: use the stored enable. Neither
            // p. 5-17 nor 9800301-04 pp. 3-23/3-49 settles that case.
            let enable = if latches.trap_rim_pending {
                latches.trap_saved_interrupt_enable
            } else {
                latches.interrupt_enable
            };
            latches.trap_rim_pending = false;
            let value = ((sid as u8) << 7)
                | ((latches.rst7_5_pending as u8) << 6)
                | ((rst6_5 as u8) << 5)
                | ((rst5_5 as u8) << 4)
                | ((enable as u8) << 3)
                | ((latches.mask7_5 as u8) << 2)
                | ((latches.mask6_5 as u8) << 1)
                | latches.mask5_5 as u8;
            cpu.registers_mut().a = value;
        }
        // 205775-002 p. 5-18, SIM; 9800301-04 p. 3-60: bit 3 is MSE,
        // bits 2..0 are masks 7.5/6.5/5.5; bit 4 resets the 7.5 latch;
        // bit 5 is unused; bit 6 enables loading SOD from bit 7. A and
        // flags are unchanged. Each enable leaves its destination alone
        // when clear; the three functions are independent (rule 8).
        0x30 => {
            let value = cpu.registers().a;
            let latches = cpu.latches_mut();
            if value & 0x08 != 0 {
                latches.mask5_5 = value & 0x01 != 0;
                latches.mask6_5 = value & 0x02 != 0;
                latches.mask7_5 = value & 0x04 != 0;
            }
            if value & 0x10 != 0 {
                latches.rst7_5_pending = false;
            }
            if value & 0x40 != 0 {
                latches.sod = value & 0x80 != 0;
                // Rule 8 notifies every enabled write, even unchanged,
                // during SIM. This simplifies §2.3.8 p. 2-19's physical
                // transfer during the instruction following SIM. Reset
                // has no callback (rule 9); only SIM writes notify here.
                system.sod_changed(latches.sod);
            }
        }
        _ => return None,
    }
    // Sequential execution: §5.4 p. 5-3. The PC's 16-bit width (§2.2.1
    // p. 2-2) motivates wrapping; wrap itself is derived, not stated.
    let registers = cpu.registers_mut();
    registers.pc = registers.pc.wrapping_add(u16::from(instruction.length));
    Some(Path::Taken)
}
