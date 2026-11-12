//! The host stop request, common to every model (decision-5 item 4).

/// The host's cooperative stop request.
///
/// A stop is not an architectural interrupt: it returns control to the
/// embedder at an instruction boundary and changes no CPU state. Each model
/// extends this trait with its own input lines, such as
/// [`i8080::Inputs`](crate::i8080::Inputs).
///
/// The core polls the request once at each instruction boundary that passes
/// the budget check, whatever the CPU state (decision-5 items 4 and 5). The
/// poll should be a side-effect-free level read.
pub trait Control {
    /// Returns `true` to end the current call with
    /// [`Exit::Stop`](crate::Exit::Stop) at this boundary.
    ///
    /// The default never requests a stop.
    fn stop_requested(&mut self) -> bool {
        false
    }
}
