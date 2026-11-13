//! Input contract (decision-15 rules 1/2).

/// The embedder's Z80 interrupt requests, beside the host stop.
///
/// Both methods return `true` when asserted: the active-low pin is Low.
/// NMI's negative edge therefore appears as a false-to-true transition of
/// [`nmi`](Self::nmi). Source: Zilog UM008011-0816 (revision 11, August 2016),
/// Pin Functions p. 6 (PDF 20); 03-0029-01 (1977) §3 p. 8 (PDF 14).
/// Defaults request no interrupt; this is an API choice, not pin bias.
///
/// The boundary execution contract polls `stop_requested`, `nmi`, `int`
/// once each in that order after the budget check, including while halted
/// or disabled, regardless of results (decision-15 rule 1). This order is
/// an API choice implemented by `Z80::run` and `Z80::step`. Polls should
/// read levels without changing the emulated device. The consecutive-sample
/// NMI sampler can miss pulses wholly between polls; hold requests until
/// sampled (rule 2). History starts/resets not asserted by library choice.
pub trait Inputs: crate::Control {
    /// Nonmaskable request: true means pin Low; false-to-true is the
    /// negative edge (UM p. 6; 03-0029-01 §3 p. 8).
    fn nmi(&mut self) -> bool {
        false
    }

    /// Maskable request: true means the active-low INT pin is Low
    /// (UM p. 6; 03-0029-01 §3 p. 8).
    fn int(&mut self) -> bool {
        false
    }
}
