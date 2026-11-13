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
    /// Read of an instruction byte classified by the model as an opcode fetch.
    ///
    /// A model may document more than one such read per instruction
    /// (decision-15 rule 5). The 8080 and 8085 use it only for the first byte
    /// (decision-5 item 3, decision-12).
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
    /// The port number is 8 bits wide (decision-5 item 3). Models requiring
    /// the full port address use [`Bus::input16`] (decision-15 rule 4).
    fn input(&mut self, port: u8) -> u8;

    /// Writes `value` to the device numbered `port`.
    fn output(&mut self, port: u8, value: u8);

    /// Reads a byte at the full 16-bit port address presented on A0-A15.
    ///
    /// Only models whose decision records specify it call this method.
    /// No current model does; the planned z80 model will (decision-15 rule 4).
    /// The default forwards A0-A7 to [`Bus::input`] so existing embedders keep
    /// working. An embedder may override it to observe the full address.
    fn input16(&mut self, port: u16) -> u8 {
        self.input(port as u8)
    }

    /// Writes `value` at the full 16-bit port address presented on A0-A15.
    ///
    /// Only models whose decision records specify it call this method.
    /// No current model does; the planned z80 model will (decision-15 rule 4).
    /// The default forwards A0-A7 to [`Bus::output`] so existing embedders keep
    /// working. An embedder may override it to observe the full address.
    fn output16(&mut self, port: u16, value: u8) {
        self.output(port as u8, value);
    }
}
