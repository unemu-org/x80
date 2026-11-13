//! INTR acceptance (decision-12 rule 5; decision-5 items 4-6/8).
//!
//! Intel 205775-002 (January 1983), §2.3.4 pp. 2-13/2-14 (PDF 36/37),
//! Figs 2-17/2-18: clear INTE, INA reads with PC increment inhibited,
//! then push boundary PC high/low and load the supplied target. RST/CALL
//! encodings, targets and unchanged flags: §5.6.4 pp. 5-13/5-14;
//! cycle sequences: Table 5-1 p. 5-19 (Table 5-2 p. 5-20 pins bytes).
//! Second witness: Intel 9800301-04 (May 1981), CALL pp. 3-8/3-9,
//! RST pp. 3-54/3-55. Intel 231718-001 (September 1987), handbook
//! pp. 16-12/16-15: INTR inhibits PC increment, accepts RST/CALL and
//! disables interrupts. Halt exit: 205775-002 §2.3.5 p. 2-16;
//! 9800301-04 p. 3-24. Page images checked; hashes: PROVENANCE.md §1.
//! Independent witnesses: tests/i8085_interrupts_intr.rs.

use super::I8085;
use crate::intel::access::push_and_jump;
use crate::{Bus, MemoryAccess, Unsupported};

/// Called only for the selected, eligible INTR source. Always consumes the
/// boundary; the optional seam result is retained for the boundary-loop API.
pub(super) fn accept<S: Bus>(cpu: &mut I8085, system: &mut S) -> Option<Result<u8, Unsupported>> {
    let pc = cpu.registers().pc;
    cpu.latches_mut().interrupt_enable = false;
    cpu.latches_mut().halted = false;
    let opcode = system.read(pc, MemoryAccess::InterruptAcknowledge);
    let (target, states) = match opcode {
        // DERIVED: INA replaces the six-state S fetch in Table 5-1's
        // RST S W W: 6+3+3=12 (Fig. 2-17; §5.6.4 p. 5-14).
        // 9800301-04 p. 3-55 independently gives 12 on the 8085.
        0xC7 | 0xCF | 0xD7 | 0xDF | 0xE7 | 0xEF | 0xF7 | 0xFF => (u16::from(opcode & 0x38), 12),
        0xCD => {
            // Figs 2-17/2-18 show PCH/PCL on all three INA cycles;
            // M2 supplies the low target byte, M3 the high byte.
            let low = system.read(pc, MemoryAccess::InterruptAcknowledge);
            let high = system.read(pc, MemoryAccess::InterruptAcknowledge);
            // DERIVED: INA 6+3+3, then MW 3+3 = 18. CALL p. 5-13,
            // Table 5-1 and 9800301-04 p. 3-9 agree on 18 (8085).
            (u16::from_le_bytes([low, high]), 18)
        }
        _ => {
            // Decision-12 rule 5 supports only RST/CALL; EI/DI are also
            // prohibited by 205775-002 p. 5-17. No register/memory effect.
            // The run loop restores pre-sampling boundary latches, a stated
            // departure from hardware for retry (decision-5 item 6).
            return Some(Err(Unsupported {
                pc,
                opcode,
                acknowledge: true,
                preceding: [0; 3],
                preceding_len: 0,
            }));
        }
    };
    push_and_jump(&mut super::view(cpu.registers_mut()), system, pc, target);
    Some(Ok(states))
}
