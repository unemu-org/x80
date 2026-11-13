//! Final INT acceptance seam (decision-15 rules 3, 6–7, 9–10).
//!
//! The loop calls only when INT is asserted, IFF1 set, EI delay expired
//! and no NMI edge has priority. Independent literal witnesses live in
//! `tests/z80_interrupts_int.rs`.

use super::{call_return, InterruptMode, Z80};
use crate::{Bus, MemoryAccess, Unsupported};

/// Always consumes the selected boundary; the optional seam type is retained.
/// Some(Ok(states)) completes one interrupt step, without retirement here
/// or handler execution.
/// Some(Err(unsupported)) refuses: the loop restores ALL pre-sampling CPU
/// state, including R/latches; only the acknowledge read may reach the bus.
///
/// Implementation contract: read InterruptAcknowledge at unchanged PC and
/// call cpu.increment_r() once. Validate IM 0 RST-only / IM 2 even-byte
/// restrictions before any write/vector read. Refusal payload: boundary PC,
/// returned opcode, acknowledge=true, preceding=[0;3], preceding_len=0.
/// On success call cpu.consume_parity_note() before service, clear both IFFs,
/// leave halt and push PC high/low as Stack. IM 0 restarts at opcode & 38h;
/// IM 1 ignores the byte and restarts at 0038h (both DERIVED 13 states).
/// IM 2 pushes PC BEFORE Data reads at (I<<8)|byte and +1 (low then high),
/// loads that target, and returns 19 states (STATED). Address wrap is derived.
/// Sources: UM008011-0816 rev. 11 (August 2016), Interrupt Request/Acknowledge
/// pp. 12–13/Fig. 9, CPU Response pp. 19–20/Fig. 17, RST p. 293;
/// 03-0029-01 (1977), §4 p. 16, §8 pp. 55–57, Table 7.0-10 p. 53.
/// Parity-note boundary mapping remains unverified (rule 10).
pub(super) fn accept<S: Bus>(cpu: &mut Z80, system: &mut S) -> Option<Result<u8, Unsupported>> {
    let pc = cpu.registers().pc;
    let opcode = system.read(pc, MemoryAccess::InterruptAcknowledge);
    cpu.increment_r();
    let mode = cpu.latches().interrupt_mode;
    let refused = match mode {
        // RST is 11ttt111 (UM RST p. 292; TM Table 7.0-10 p. 53).
        // RST-only is the library scope restriction, not all of IM 0.
        InterruptMode::Mode0 => opcode & 0xC7 != 0xC7,
        InterruptMode::Mode1 => false,
        // UM p. 20/Fig. 17; TM p. 57 require an even table pointer.
        // Refusing odd values is decision-15 rule 9's library restriction.
        InterruptMode::Mode2 => opcode & 1 != 0,
    };
    if refused {
        // Only the acknowledge read has reached the bus. The loop restores
        // all CPU state, including this M1's R increment and sample history.
        return Some(Err(Unsupported {
            pc,
            opcode,
            acknowledge: true,
            preceding: [0; 3],
            preceding_len: 0,
        }));
    }

    // Success only: UM Interrupt Enable/Disable p. 18/HALT Exit p. 14;
    // TM §8 p. 55/§4 p. 18. Rule 10 owns the parity-note interpretation.
    cpu.latches_mut().iff1 = false;
    cpu.latches_mut().iff2 = false;
    cpu.latches_mut().halted = false;
    cpu.consume_parity_note();

    let states = match mode {
        InterruptMode::Mode0 | InterruptMode::Mode1 => {
            let target = if mode == InterruptMode::Mode0 {
                u16::from(opcode & 0x38)
            } else {
                0x0038
            };
            call_return::push_pc_and_jump(cpu, system, pc, target);
            // DERIVED: UM CPU Response p. 19's +2, RST p. 293's 11
            // (5,3,3); TM §8 p. 56/Table 7.0-10 p. 53 agree.
            13
        }
        InterruptMode::Mode2 => {
            let vector = u16::from_be_bytes([cpu.registers().i, opcode]);
            // UM p. 20/TM p. 57: save PC BEFORE the vector lookup.
            // Keep PC at the boundary until the complete target is read.
            call_return::push_pc_and_jump(cpu, system, pc, pc);
            let low = system.read(vector, MemoryAccess::Data);
            // DERIVED 16-bit address wrap (UM p. 2; TM §2.1 p. 3).
            // An admitted even pointer cannot actually reach FFFFh here.
            let high = system.read(vector.wrapping_add(1), MemoryAccess::Data);
            cpu.registers_mut().pc = u16::from_le_bytes([low, high]);
            // L: decision-20 §5; MEMPTR (©2006), interrupt CALL extension.
            cpu.latches_mut().wz = cpu.registers().pc;
            // STATED: UM p. 20/Fig. 17; TM §8 p. 57: 7 acknowledge,
            // 6 PC-save and 6 table-read clocks, explicitly 19 in total.
            19
        }
    };
    Some(Ok(states))
}
