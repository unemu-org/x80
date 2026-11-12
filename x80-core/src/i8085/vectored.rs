//! Vectored acceptance seam for x80-4gk.17 (decision-12 rules 3/4).
//! No interrupt is accepted yet; the seam has no state or bus effects.

use super::I8085;
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

/// Future acceptance returns its completed state total, counting one step.
/// `None` means no acceptance, permitting halt or ordinary fetch.
pub(super) fn accept<S: Bus>(_cpu: &mut I8085, _system: &mut S, _source: Source) -> Option<u8> {
    None
}
