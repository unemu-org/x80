//! Run limits and the progress a call reports (decision-2, decision-5).

use crate::Exit;

/// Limits for one call that executes instructions (decision-2).
///
/// A call returns [`Exit::Budget`] at the first instruction boundary at which
/// either limit is reached; a zero limit executes nothing. An instruction that
/// has started always completes, so the reported cycles may exceed `cycles`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Budget {
    /// Maximum number of instructions to execute.
    pub steps: u64,
    /// Cycle (clock state) limit, checked at instruction boundaries.
    pub cycles: u64,
}

/// What one call did and why it returned (decision-2, decision-5).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Report {
    /// Instructions executed by the call. Saturates at `u64::MAX`.
    pub steps: u64,
    /// Cycles (clock states) spent by the call, including any overshoot of the
    /// budget. Saturates at `u64::MAX`.
    pub cycles: u64,
    /// Why the call returned.
    pub exit: Exit,
}
