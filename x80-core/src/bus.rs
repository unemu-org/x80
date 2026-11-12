//! The embedder's memory and port interface (decision-1, decision-5 item 3).
//!
//! The core borrows the bus for the length of a call and owns no memory or
//! devices. Accesses are synchronous and infallible: no bus faults, no wait
//! states and no bus-cycle stepping. Mapping, mirroring and devices belong to
//! the embedder.

/// The kind of a memory access, passed with every memory read and write.
///
/// The model that performs the access documents, per instruction and with
/// citations, which kind each access carries. The enum is non-exhaustive so
/// that a later model can add kinds (decision-5 item 3).
#[non_exhaustive]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum MemoryAccess {
    /// Read of the first byte of an instruction.
    OpcodeFetch,
    /// Read of a byte of an instruction after its first byte.
    ///
    /// An API distinction, even where the hardware reports such a read as an
    /// ordinary memory read: it lets an embedder place instruction-stream bytes
    /// differently from data.
    OperandFetch,
    /// Memory data access that is neither an instruction byte nor a stack
    /// access.
    Data,
    /// Access at the stack pointer.
    Stack,
    /// Read of the instruction byte that a device supplies when an interrupt is
    /// accepted.
    InterruptAcknowledge,
}

/// Memory and port access supplied by the embedder (decision-1, decision-5
/// item 3).
///
/// The core calls these methods during an instruction, in program order. The
/// sequence of calls with their arguments is part of the deterministic trace.
pub trait Bus {
    /// Reads the byte at `address`.
    ///
    /// For [`MemoryAccess::InterruptAcknowledge`] the embedder returns the
    /// instruction byte its device supplies instead of a memory byte.
    fn read(&mut self, address: u16, kind: MemoryAccess) -> u8;

    /// Writes `value` to `address`.
    ///
    /// `kind` is [`MemoryAccess::Data`] or [`MemoryAccess::Stack`].
    fn write(&mut self, address: u16, value: u8, kind: MemoryAccess);

    /// Reads a byte from the device numbered `port`.
    ///
    /// The port number is 8 bits wide; its width is revisited when a later
    /// model needs more (decision-5 item 3).
    fn input(&mut self, port: u8) -> u8;

    /// Writes `value` to the device numbered `port`.
    fn output(&mut self, port: u8, value: u8);
}
