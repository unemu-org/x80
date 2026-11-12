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
    Unsupported(Unsupported),
}

/// An instruction byte the model does not execute (decision-3, decision-5
/// item 6).
///
/// The exit contributes no steps and no cycles. Registers, flags, memory and
/// the execution latches are as they were at the instruction boundary; only the
/// read of `opcode` has reached the bus. The struct is non-exhaustive so that a
/// later model can add prefix bytes.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Unsupported {
    /// For a fetched opcode, the address of the instruction's first byte. For
    /// an interrupt acknowledge, the address placed on the bus for the
    /// acknowledge read.
    pub pc: u16,
    /// The instruction byte that was read.
    pub opcode: u8,
    /// `true` if `opcode` came from an interrupt acknowledge rather than an
    /// opcode fetch.
    pub acknowledge: bool,
}
