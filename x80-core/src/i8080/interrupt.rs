//! The 8080 input lines (decision-5 item 4, plan §2.5).

use crate::Control;

/// The 8080 input lines that the embedder drives, beside the host stop.
///
/// The 8080 has one interrupt request line, INT (98-153B p. 5-14, pin
/// definitions). The core polls it once at each instruction boundary that
/// passes the budget check, after [`Control::stop_requested`], whatever its
/// result and the CPU state (decision-5 item 4). The poll should be a
/// side-effect-free level read.
pub trait Inputs: Control {
    /// Returns the level of INT: `true` requests an interrupt.
    ///
    /// The default never requests one.
    fn interrupt_requested(&mut self) -> bool {
        false
    }
}
