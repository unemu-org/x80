//! Final INT acceptance seam (decision-15 rules 3, 6–7, 9–10).
//!
//! x80-bi2.23 implements this module and its own interrupt witnesses; it
//! never edits cpu.rs or tests/z80_run.rs. The loop calls only when INT is
//! asserted, IFF1 set, EI delay expired and no NMI edge has priority.

use super::Z80;
use crate::{Bus, Unsupported};

/// None declines WITHOUT any CPU/bus effect (staging only). Some(Ok(states))
/// completes one interrupt step, without retirement here or handler execution.
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
pub(super) fn accept<S: Bus>(_cpu: &mut Z80, _system: &mut S) -> Option<Result<u8, Unsupported>> {
    None
}
