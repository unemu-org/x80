//! The 8080 Stack, I/O and Machine Control Group: its encodings and its
//! executor.
//!
//! Primary source: 98-153B Ch. 4, "Stack, I/O, and Machine Control Group",
//! pp. 4-13 and 4-14, checked against the page images. Field letters are
//! those of `Encoding` in `opcodes`.
//!
//! Status: only NOP executes. Every other opcode of the group returns
//! `Exit::Unsupported` (decision-5 item 7) until its bead lands; its state
//! totals are not cited yet.

use super::opcodes::{Encoding, Instruction, Path, States};
use super::I8080;
use crate::Bus;

/// NOP, 00000000 (98-153B p. 4-14).
const NOP: u8 = 0b0000_0000;

/// The group's encodings, in page order.
pub(super) const ENCODINGS: &[Encoding] = &[
    // PUSH rp, 11RP0101: "Register pair rp = SP may not be specified",
    // p. 4-13. Listed for B, D and H.
    Encoding::new(b"11000101", 1),
    Encoding::new(b"11010101", 1),
    Encoding::new(b"11100101", 1),
    // PUSH PSW, p. 4-13.
    Encoding::new(b"11110101", 1),
    // POP rp, 11RP0001, with the same restriction, p. 4-13.
    Encoding::new(b"11000001", 1),
    Encoding::new(b"11010001", 1),
    Encoding::new(b"11100001", 1),
    // POP PSW, p. 4-13.
    Encoding::new(b"11110001", 1),
    // XTHL, p. 4-14.
    Encoding::new(b"11100011", 1),
    // SPHL, p. 4-14.
    Encoding::new(b"11111001", 1),
    // IN port: opcode and port byte, p. 4-14.
    Encoding::new(b"11011011", 2),
    // OUT port, p. 4-14.
    Encoding::new(b"11010011", 2),
    // EI, p. 4-14.
    Encoding::new(b"11111011", 1),
    // DI, p. 4-14.
    Encoding::new(b"11110011", 1),
    // HLT, p. 4-14.
    Encoding::new(b"01110110", 1),
    // NOP, p. 4-14: "Cycles: 1, States: 4". Second witness: 9800301-04
    // p. A-4, NOP, 8080 column: 4.
    Encoding::new(b"00000000", 1).with_states(States::single(4)),
];

/// Executes `opcode`, an instruction of this group.
///
/// Returns `None`, having changed nothing and accessed nothing, for an opcode
/// the group does not execute yet; the caller then returns `Unsupported`
/// (decision-5 item 7). Only NOP executes.
pub(super) fn execute<S: Bus>(
    cpu: &mut I8080,
    _system: &mut S,
    opcode: u8,
    instruction: Instruction,
) -> Option<Path> {
    match opcode {
        // NOP (98-153B p. 4-14): "No operation is performed. The registers
        // and flags are unaffected." The program counter moves past the
        // instruction: execution "proceeds through consecutively increasing
        // memory locations" (p. 4-2) and the PC "is incremented automatically
        // during every instruction fetch" (p. 2-2). It wraps from 0xFFFF to
        // 0x0000 because it is a 16-bit register (p. 2-2); no page states the
        // wrap itself.
        NOP => {
            let registers = cpu.registers_mut();
            registers.pc = registers.pc.wrapping_add(u16::from(instruction.length));
            Some(Path::Taken)
        }
        _ => None,
    }
}
