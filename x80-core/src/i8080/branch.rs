//! The 8080 Branch Group: its encodings and its executor.
//!
//! Primary source: 98-153B Ch. 4, "Branch Group", pp. 4-11 and 4-12, checked
//! against the page images. The condition codes CCC are listed on p. 4-11.
//! Field letters are those of `Encoding` in `opcodes`.
//!
//! Status: not implemented. Every opcode of the group returns
//! `Exit::Unsupported` (decision-5 item 7) until the group's bead lands; its
//! state totals are not cited yet.

use super::opcodes::{Encoding, Instruction, Path};
use super::I8080;
use crate::Bus;

/// The group's encodings, in page order.
pub(super) const ENCODINGS: &[Encoding] = &[
    // JMP addr: opcode, low-order and high-order address, p. 4-11.
    Encoding::new(b"11000011", 3),
    // Jcondition addr, p. 4-11.
    Encoding::new(b"11CCC010", 3),
    // CALL addr, p. 4-11.
    Encoding::new(b"11001101", 3),
    // Ccondition addr, p. 4-12.
    Encoding::new(b"11CCC100", 3),
    // RET, p. 4-12.
    Encoding::new(b"11001001", 1),
    // Rcondition, p. 4-12.
    Encoding::new(b"11CCC000", 1),
    // RST n, p. 4-12.
    Encoding::new(b"11NNN111", 1),
    // PCHL, p. 4-12.
    Encoding::new(b"11101001", 1),
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
