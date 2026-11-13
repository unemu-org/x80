//! Run accounting shared by models (decision-2, decision-5, decision-10).
//! Models supply the completed instruction cost and choose when to retire it.

use crate::{Budget, Exit, Report};

/// The steps and cycles spent by one call (decision-2).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct Progress {
    steps: u64,
    cycles: u64,
}

impl Progress {
    /// Whether either limit of `budget` is reached.
    pub(crate) const fn reached(self, budget: Budget) -> bool {
        self.steps >= budget.steps || self.cycles >= budget.cycles
    }

    /// Counts one completed instruction of `states` states. Both counters
    /// saturate at `u64::MAX` (decision-2).
    pub(crate) fn retire(&mut self, states: u8) {
        self.retire_wide(u64::from(states));
    }

    /// Counts one completed instruction with a full-width cost.
    pub(crate) fn retire_wide(&mut self, states: u64) {
        self.steps = self.steps.saturating_add(1);
        self.cycles = self.cycles.saturating_add(states);
    }

    pub(crate) const fn report(self, exit: Exit) -> Report {
        Report {
            steps: self.steps,
            cycles: self.cycles,
            exit,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Progress;
    use crate::Budget;

    // Saturation cannot be reached through `run`: the boundary check stops
    // before the step counter can pass `budget.steps`, and the cycle counter
    // would need about 2^62 instructions. It is checked here directly.

    #[test]
    fn retire_saturates_both_counters() {
        let mut progress = Progress {
            steps: u64::MAX,
            cycles: u64::MAX - 2,
        };
        progress.retire(4);
        assert_eq!(
            progress,
            Progress {
                steps: u64::MAX,
                cycles: u64::MAX,
            }
        );
        progress.retire(18);
        assert_eq!(
            progress,
            Progress {
                steps: u64::MAX,
                cycles: u64::MAX,
            }
        );
    }

    #[test]
    fn retire_adds_one_step_and_the_states() {
        let mut progress = Progress {
            steps: 7,
            cycles: 100,
        };
        progress.retire(4);
        assert_eq!(
            progress,
            Progress {
                steps: 8,
                cycles: 104,
            }
        );
    }

    #[test]
    fn a_saturated_counter_reaches_the_largest_budget() {
        let budget = Budget {
            steps: u64::MAX,
            cycles: u64::MAX,
        };
        let saturated_cycles = Progress {
            steps: 0,
            cycles: u64::MAX,
        };
        let saturated_steps = Progress {
            steps: u64::MAX,
            cycles: 0,
        };
        assert!(saturated_cycles.reached(budget));
        assert!(saturated_steps.reached(budget));
        assert!(!Progress::default().reached(budget));
    }
}
