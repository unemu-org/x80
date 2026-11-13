//! Final NMI acceptance seam (decision-15 rules 3, 6–8, 10).
//!
//! x80-bi2.22 implements this module and its own interrupt witnesses; it
//! never edits cpu.rs or tests/z80_run.rs. The loop selects only a fresh NMI
//! edge, independently of IFF1/EI delay, and has committed sample history.

use super::Z80;
use crate::Bus;

/// None declines acceptance WITHOUT any CPU/bus effect (staging only).
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
pub(super) fn accept<S: Bus>(_cpu: &mut Z80, _system: &mut S) -> Option<u8> {
    None
}
