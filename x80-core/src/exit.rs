//! Typed return reasons (decision-3, decision-5 items 5, 6 and 10).

/// Why a call that executes instructions returned to the embedder.
///
/// When several conditions hold at one instruction boundary, the first in
/// decision-5 item 5 wins: budget, stop, interrupt acceptance, halt, then fetch
/// and execute. The enum is non-exhaustive so that integration profiles and
/// later models can add exits (decision-5 item 10).
#[non_exhaustive]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Exit {
    /// A [`Budget`](crate::Budget) limit was reached.
    Budget,
    /// The host requested a stop through
    /// [`Control::stop_requested`](crate::Control::stop_requested).
    Stop,
    /// The CPU is halted and no interrupt was accepted.
    Halt,
    /// The next instruction has no implemented behaviour (decision-3).
    ///
    /// Only instruction-stream reads of the refused instruction have reached
    /// the bus (decision-5 item 6, decision-15 rule 13); see [`Unsupported`].
    Unsupported(Unsupported),
}

/// An instruction byte the model does not execute (decision-3, decision-5
/// item 6, decision-14 rule 3, decision-15 rule 13).
///
/// The exit contributes no steps and no cycles. Registers, flags, memory and
/// the execution latches are as they were at the instruction boundary; only the
/// instruction-stream reads of the refused instruction have reached the bus.
/// On the Z80 these include every leading DD/FD prefix of a chain, even when
/// the `preceding` payload omits earlier selectors. Device side effects of those
/// reads cannot be rolled back. The struct is non-exhaustive so that a
/// later model can extend the diagnostic payload.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Unsupported {
    /// For a fetched opcode, the address of the instruction's first byte. For
    /// an interrupt acknowledge, the address placed on the bus for the
    /// acknowledge read.
    pub pc: u16,
    /// The byte that failed to decode.
    pub opcode: u8,
    /// `true` if `opcode` came from an interrupt acknowledge rather than an
    /// opcode fetch.
    pub acknowledge: bool,
    /// The used prefix holds, in read order, the instruction-stream bytes of
    /// the final decode context read before the refused byte, including any
    /// displacement (decision-14 rule 3). For a Z80 DD/FD chain only the last
    /// selector is kept, e.g. `[DD, CB, d]` for an indexed-CB refusal or `[CB]`
    /// for SLL; earlier selectors are omitted, although they were read
    /// (decision-20 §2, library convention). Unused elements are zero;
    /// [`Self::preceding_len`] gives the used length. The 8080 and 8085 always report `[0; 3]` and length zero.
    pub preceding: [u8; 3],
    /// Number of used elements in [`Self::preceding`], in `0..=3`.
    pub preceding_len: u8,
}

impl Unsupported {
    /// The final-context instruction-stream bytes read before the refused byte,
    /// in read order, as described for the `preceding` field.
    ///
    /// Clamps an invalid [`Self::preceding_len`] to the array's capacity so
    /// this accessor cannot panic even if the public length was changed.
    pub fn preceding(&self) -> &[u8] {
        let len = usize::from(self.preceding_len).min(self.preceding.len());
        &self.preceding[..len]
    }
}
