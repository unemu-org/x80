//! The 8085 Stack, I/O and Machine Control encodings and execution seam.
//!
//! Intel 205775-002 (January 1983), §5.6.5, pp. 5-15 to 5-17, checked on page images.
//! Totals below are the 8085 states, not machine cycles or 8080 states.
//! Independent witness: `opcodes/tests.rs`, Table 5-1 p. 5-19 cross-read
//! with Intel 9800301-04 (May 1981), App. A pp. A-1 to A-5.
//! Design: decision-10, decision-11; decision-5 item 7's interim path.

use super::opcodes::{Encoding, Instruction, Path, States};
use super::I8085;
use crate::Bus;

/// Complete group list, in per-instruction page order.
/// PUSH/POP allow B, D and H pairs, with PSW as a separate entry
/// (205775-002 pp. 5-15/5-16). RIM/SIM live in their own module
/// under decision-10.
pub(super) const ENCODINGS: &[Encoding] = &[
    // PUSH B: 205775-002 p. 5-15.
    Encoding::new(b"11000101", 1, States::single(12)),
    // PUSH D: 205775-002 p. 5-15.
    Encoding::new(b"11010101", 1, States::single(12)),
    // PUSH H: 205775-002 p. 5-15.
    Encoding::new(b"11100101", 1, States::single(12)),
    // PUSH PSW: 205775-002 p. 5-15.
    Encoding::new(b"11110101", 1, States::single(12)),
    // POP B: 205775-002 p. 5-15.
    Encoding::new(b"11000001", 1, States::single(10)),
    // POP D: 205775-002 p. 5-15.
    Encoding::new(b"11010001", 1, States::single(10)),
    // POP H: 205775-002 p. 5-15.
    Encoding::new(b"11100001", 1, States::single(10)),
    // POP PSW: 205775-002 p. 5-16.
    Encoding::new(b"11110001", 1, States::single(10)),
    // XTHL: 205775-002 p. 5-16.
    Encoding::new(b"11100011", 1, States::single(16)),
    // SPHL: 205775-002 p. 5-16.
    Encoding::new(b"11111001", 1, States::single(6)),
    // IN port: 205775-002 p. 5-16.
    Encoding::new(b"11011011", 2, States::single(10)),
    // OUT port: 205775-002 p. 5-16.
    Encoding::new(b"11010011", 2, States::single(10)),
    // EI: 205775-002 p. 5-17.
    Encoding::new(b"11111011", 1, States::single(4)),
    // DI: 205775-002 p. 5-17.
    Encoding::new(b"11110011", 1, States::single(4)),
    // HLT: 205775-002 p. 5-17.
    Encoding::new(b"01110110", 1, States::single(5)),
    // NOP: 205775-002 p. 5-17.
    Encoding::new(b"00000000", 1, States::single(4)),
];

/// Only NOP completes; all other entries return no path without effects.
pub(super) fn execute<S: Bus>(
    cpu: &mut I8085,
    _system: &mut S,
    opcode: u8,
    _instruction: Instruction,
) -> Option<Path> {
    // NOP alone executes: 205775-002 p. 5-17, 00000000, no register
    // or flag change, four states. Sequential PC: §5.4 p. 5-3.
    if opcode == 0x00 {
        cpu.registers_mut().pc = cpu.registers().pc.wrapping_add(1);
        Some(Path::Untaken)
    } else {
        None
    }
}
