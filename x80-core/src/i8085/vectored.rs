//! Internally vectored acceptance (decision-12 rules 2-4/7).
//!
//! Intel 205775-002 (January 1983), §2.2.7 pp. 2-4/2-5 (PDF 27/28),
//! §2.3.5 Fig. 2-19 p. 2-15 (PDF 38), RST p. 5-14 (PDF 99), RIM/SIM
//! pp. 5-17/5-18 (PDF 102/103). Second witnesses: Intel 9800301-04
//! (May 1981), RST pp. 3-54/3-55 (PDF 108/109), HLT p. 3-24 (PDF 78);
//! Intel 231718-001 (September 1987), Table 2 p. 16-13, TRAP p. 16-15,
//! Table 3 p. 16-20 (handbook PDF 960/962/967). Values checked on page
//! images; source hashes match PROVENANCE.md §1. Independent witnesses:
//! `tests/i8085_interrupts_vectored.rs`.

use super::{branch, I8085};
use crate::Bus;

/// Internally vectored sources in priority order: Intel 231718-001
/// (September 1987), handbook p. 16-13 Table 2; decision-12 rule 3.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Source {
    Trap,
    Rst7_5,
    Rst6_5,
    Rst5_5,
}

/// The run loop has selected an eligible source; every source completes.
/// The optional return type is the existing acceptance-seam contract.
pub(super) fn accept<S: Bus>(cpu: &mut I8085, system: &mut S, source: Source) -> Option<u8> {
    let latches = cpu.latches_mut();
    // Vectors: 205775-002 p. 2-5; 9800301-04 p. 3-55; DS Table 2.
    let vector = match source {
        Source::Trap => {
            // First RIM preserves the preceding enable once (p. 5-17;
            // DS p. 16-15). Clear/disarm until low then high (DS Fig. 4,
            // p. 16-15; decision-12 rule 2's boundary-sampler model).
            latches.trap_saved_interrupt_enable = latches.interrupt_enable;
            latches.trap_rim_pending = true;
            latches.trap_pending = false;
            latches.trap_armed = false;
            0x0024
        }
        Source::Rst7_5 => {
            // Its own acceptance resets the flip-flop (p. 5-18;
            // 9800301-04 p. 3-60). Other sources leave it pending.
            latches.rst7_5_pending = false;
            0x003C
        }
        Source::Rst6_5 => 0x0034,
        Source::Rst5_5 => 0x002C,
    };
    // All acceptance disables interrupts (DS p. 16-15); valid interrupts
    // leave halt (205775-002 p. 2-16; 9800301-04 p. 3-24).
    latches.interrupt_enable = false;
    latches.halted = false;

    // Fig. 2-19: BI generates the opcode internally, without RD or INTA;
    // DS Table 3 agrees. No read/callback for BI. Push the unchanged
    // boundary PC (p. 2-5 note 1), high then low Stack writes via RST's
    // operation (p. 5-14; Fig. 2-18 p. 2-14; 9800301-04 pp. 3-54/3-55).
    let return_pc = cpu.registers().pc;
    branch::restart(cpu, system, return_pc, vector);

    // DERIVED: six BI states in Fig. 2-19 replace RST's six-state fetch;
    // the two three-state writes remain: 6 + 3 + 3 = 12. RST p. 5-14
    // and Table 5-1 p. 5-19 (S W W; Table 5-2 is the opcode list),
    // also 9800301-04 p. 3-55, independently give RST 12 states on 8085.
    // Decision-12 rule 4 counts acceptance as one step.
    Some(12)
}
