//! Final NMI acceptance seam (decision-15 rules 3, 6–8, 10).
//!
//! The loop selects only a fresh NMI edge, independently of IFF1/EI delay,
//! and has committed sample history. Independent literal witnesses live in
//! `tests/z80_interrupts_nmi.rs`.

use super::{call_return, Z80};
use crate::{Bus, MemoryAccess};

/// The run loop has selected a fresh edge; acceptance always completes.
/// The optional return type preserves the acceptance-seam contract.
/// Some(states) completes one interrupt step with its whole cost; no handler
/// instruction executes in that step. No retirement is done in this seam.
///
/// Implementation contract: discard one OpcodeFetch at unchanged PC, call
/// cpu.increment_r() once; call cpu.consume_parity_note() on acceptance before
/// service; clear IFF1, keep IFF2 and EI delay, leave halt, push boundary PC
/// high/low as Stack and set PC=0066h. Return Some(11), a DERIVED total.
/// Sources: UM008011-0816 rev. 11 (August 2016), Figs 9/10 pp. 13–14,
/// CPU Response/Table 1 p. 19, RST p. 293, RETN p. 290; 03-0029-01 (1977),
/// §4 p. 18, §8 p. 56 and Table 7.0-10 p. 53. Rules 7/10's EI-delay and
/// parity-note boundary interpretations remain explicitly unverified.
pub(super) fn accept<S: Bus>(cpu: &mut Z80, system: &mut S) -> Option<u8> {
    let pc = cpu.registers().pc;
    system.read(pc, MemoryAccess::OpcodeFetch);
    cpu.increment_r();
    // L: decision-20 §6 interrupt-entry classification, no hardware claim.
    cpu.latches_mut().q = false;
    cpu.consume_parity_note();
    cpu.latches_mut().iff1 = false;
    cpu.latches_mut().halted = false;
    call_return::push_pc_and_jump(cpu, system, pc, 0x0066);

    // DERIVED: restart equivalence (UM CPU Response p. 19; TM §8 p. 56)
    // plus RST's 11 (5,3,3) states (UM p. 293; TM Table 7.0-10 p. 53).
    Some(11)
}
