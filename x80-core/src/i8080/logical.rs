//! The 8080 Logical Group: its encodings and its executor.
//!
//! Primary source: 98-153B Ch. 4, "Logical Group", pp. 4-8 to 4-11 (STC is
//! the last entry before "Branch Group" on p. 4-11), checked against the page
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
    // ANA r, p. 4-8.
    Encoding::new(b"10100SSS", 1),
    // ANA M, p. 4-8.
    Encoding::new(b"10100110", 1),
    // ANI data: opcode and data byte, p. 4-9.
    Encoding::new(b"11100110", 2),
    // XRA r, p. 4-9.
    Encoding::new(b"10101SSS", 1),
    // XRA M, p. 4-9.
    Encoding::new(b"10101110", 1),
    // XRI data, p. 4-9.
    Encoding::new(b"11101110", 2),
    // ORA r, p. 4-9.
    Encoding::new(b"10110SSS", 1),
    // ORA M, p. 4-9.
    Encoding::new(b"10110110", 1),
    // ORI data, p. 4-9.
    Encoding::new(b"11110110", 2),
    // CMP r, p. 4-9.
    Encoding::new(b"10111SSS", 1),
    // CMP M, p. 4-10.
    Encoding::new(b"10111110", 1),
    // CPI data, p. 4-10.
    Encoding::new(b"11111110", 2),
    // RLC, p. 4-10.
    Encoding::new(b"00000111", 1),
    // RRC, p. 4-10.
    Encoding::new(b"00001111", 1),
    // RAL, p. 4-10.
    Encoding::new(b"00010111", 1),
    // RAR, p. 4-10.
    Encoding::new(b"00011111", 1),
    // CMA, p. 4-10.
    Encoding::new(b"00101111", 1),
    // CMC, p. 4-11.
    Encoding::new(b"00111111", 1),
    // STC, p. 4-11.
    Encoding::new(b"00110111", 1),
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
