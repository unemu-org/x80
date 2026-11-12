//! The 8080 Arithmetic Group: its encodings and its executor.
//!
//! Primary source: 98-153B Ch. 4, "Arithmetic Group", pp. 4-6 to 4-8 (DAA is
//! the last entry before "Logical Group" on p. 4-8), checked against the page
//! images. Field letters are those of `Encoding` in `opcodes`.
//!
//! Status: not implemented. Every opcode of the group returns
//! `Exit::Unsupported` (decision-5 item 7) until the group's bead lands; its
//! state totals are not cited yet.

use super::opcodes::{Encoding, Instruction, Path};
use super::I8080;
use crate::Bus;

/// The group's encodings, in page order.
pub(super) const ENCODINGS: &[Encoding] = &[
    // ADD r, p. 4-6.
    Encoding::new(b"10000SSS", 1),
    // ADD M, p. 4-6.
    Encoding::new(b"10000110", 1),
    // ADI data: opcode and data byte, p. 4-6.
    Encoding::new(b"11000110", 2),
    // ADC r, p. 4-6.
    Encoding::new(b"10001SSS", 1),
    // ADC M, p. 4-6.
    Encoding::new(b"10001110", 1),
    // ACI data, p. 4-6.
    Encoding::new(b"11001110", 2),
    // SUB r, p. 4-6.
    Encoding::new(b"10010SSS", 1),
    // SUB M, p. 4-7.
    Encoding::new(b"10010110", 1),
    // SUI data, p. 4-7.
    Encoding::new(b"11010110", 2),
    // SBB r, p. 4-7.
    Encoding::new(b"10011SSS", 1),
    // SBB M, p. 4-7.
    Encoding::new(b"10011110", 1),
    // SBI data, p. 4-7.
    Encoding::new(b"11011110", 2),
    // INR r, p. 4-7.
    Encoding::new(b"00DDD100", 1),
    // INR M, p. 4-7.
    Encoding::new(b"00110100", 1),
    // DCR r, p. 4-7.
    Encoding::new(b"00DDD101", 1),
    // DCR M, p. 4-8.
    Encoding::new(b"00110101", 1),
    // INX rp, p. 4-8.
    Encoding::new(b"00RP0011", 1),
    // DCX rp, p. 4-8.
    Encoding::new(b"00RP1011", 1),
    // DAD rp, p. 4-8.
    Encoding::new(b"00RP1001", 1),
    // DAA, p. 4-8.
    Encoding::new(b"00100111", 1),
];

/// Executes `opcode`, an instruction of this group.
///
/// Returns `None`, having changed nothing and accessed nothing, for an opcode
/// the group does not execute yet; the caller then returns `Unsupported`
/// (decision-5 item 7). No opcode is executed yet.
pub(super) fn execute<S: Bus>(
    _cpu: &mut I8080,
    _system: &mut S,
    _opcode: u8,
    _instruction: Instruction,
) -> Option<Path> {
    None
}
