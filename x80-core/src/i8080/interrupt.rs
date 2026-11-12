//! The 8080 input lines and external interrupt contract (decision-5 item 4,
//! plan §2.5).

use crate::Control;

/// The 8080 input lines that the embedder drives, beside the host stop.
///
/// The 8080 has one interrupt request line, INT (98-153B p. 5-14, pin
/// definitions). The core polls it once at each instruction boundary that
/// passes the budget check, after [`Control::stop_requested`], whatever its
/// result and the CPU state (decision-5 item 4). The poll should be a
/// side-effect-free level read.
///
/// INT is eligible when interrupt enable is set and the EI delay has expired.
/// Disabled requests are not latched by the core. Acceptance clears interrupt
/// enable, leaves halt and reads an instruction at the unchanged PC using
/// [`MemoryAccess::InterruptAcknowledge`](crate::MemoryAccess::InterruptAcknowledge)
/// (Intel 98-153B, September 1975, Ch. 2 Interrupt Sequences/Fig. 2-8,
/// p. 2-11; Halt Sequences p. 2-13; Ch. 5 INT/INTE p. 5-14).
/// Only RST n is supported on acknowledge; it counts as one step and 11 states.
/// Other bytes return [`Exit::Unsupported`](crate::Exit::Unsupported) with the
/// CPU's boundary state unchanged (decision-5 items 6/8). A host stop takes
/// precedence over acceptance (item 5).
pub trait Inputs: Control {
    /// Returns the level of INT: `true` requests an interrupt.
    ///
    /// The default never requests one.
    fn interrupt_requested(&mut self) -> bool {
        false
    }
}
