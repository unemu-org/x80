//! INTR acceptance seam for x80-4gk.18 (decision-12 rule 5).
//! No interrupt is accepted yet; no acknowledge read or state change occurs.

use super::I8085;
use crate::{Bus, Unsupported};

/// Future acceptance returns a state total or an unsupported acknowledge;
/// `None` means no acceptance, permitting halt or ordinary fetch.
pub(super) fn accept<S: Bus>(_cpu: &mut I8085, _system: &mut S) -> Option<Result<u8, Unsupported>> {
    None
}
