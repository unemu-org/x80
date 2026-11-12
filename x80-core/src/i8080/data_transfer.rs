//! The 8080 Data Transfer Group: its encodings and its executor.
//!
//! Primary source: 98-153B Ch. 4, "Data Transfer Group", pp. 4-4 and 4-5,
//! checked against the page images. Field letters are those of
//! `Encoding` in `opcodes`.
//!
//! Status: not implemented. Every opcode of the group returns
//! `Exit::Unsupported` (decision-5 item 7) until the group's bead lands; its
//! state totals are not cited yet.

use super::opcodes::{Encoding, Instruction, Path};
use super::I8080;
use crate::Bus;

/// The group's encodings, in page order.
pub(super) const ENCODINGS: &[Encoding] = &[
    // MOV r1,r2, p. 4-4.
    Encoding::new(b"01DDDSSS", 1),
    // MOV r,M, p. 4-4.
    Encoding::new(b"01DDD110", 1),
    // MOV M,r, p. 4-4.
    Encoding::new(b"01110SSS", 1),
    // MVI r,data: opcode and data byte, p. 4-4.
    Encoding::new(b"00DDD110", 2),
    // MVI M,data, p. 4-4.
    Encoding::new(b"00110110", 2),
    // LXI rp,data 16: opcode, low-order and high-order data, p. 4-4.
    Encoding::new(b"00RP0001", 3),
    // LDA addr: opcode, low-order and high-order address, p. 4-5.
    Encoding::new(b"00111010", 3),
    // STA addr, p. 4-5.
    Encoding::new(b"00110010", 3),
    // LHLD addr, p. 4-5.
    Encoding::new(b"00101010", 3),
    // SHLD addr, p. 4-5.
    Encoding::new(b"00100010", 3),
    // LDAX rp, 00RP1010: "only register pairs rp=B (registers B and C) or
    // rp=D (registers D and E) may be specified", p. 4-5.
    Encoding::new(b"00001010", 1),
    Encoding::new(b"00011010", 1),
    // STAX rp, 00RP0010, with the same restriction, p. 4-5.
    Encoding::new(b"00000010", 1),
    Encoding::new(b"00010010", 1),
    // XCHG, p. 4-5.
    Encoding::new(b"11101011", 1),
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
