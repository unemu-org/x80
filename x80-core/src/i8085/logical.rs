//! The 8085 Logical encodings and execution seam.
//!
//! Intel 205775-002 (January 1983), §5.6.3, pp. 5-9 to 5-12, checked on page images.
//! Totals below are the 8085 states, not machine cycles or 8080 states.
//! Independent witness: `opcodes/tests.rs`, Table 5-1 p. 5-19 cross-read
//! with Intel 9800301-04 (May 1981), App. A pp. A-1 to A-5.
//! Design: decision-10, decision-11; decision-5 item 7's interim path.

use super::opcodes::{Encoding, Instruction, Path, States};
use super::I8085;
use crate::Bus;

/// Complete group list, in per-instruction page order.
/// Register-field exclusions live in `opcodes` (§5.2 p. 5-1).
pub(super) const ENCODINGS: &[Encoding] = &[
    // ANA r: 205775-002 p. 5-9.
    Encoding::new(b"10100SSS", 1, States::single(4)),
    // ANA M: 205775-002 p. 5-10.
    Encoding::new(b"10100110", 1, States::single(7)),
    // ANI data: 205775-002 p. 5-10.
    Encoding::new(b"11100110", 2, States::single(7)),
    // XRA r: 205775-002 p. 5-10.
    Encoding::new(b"10101SSS", 1, States::single(4)),
    // XRA M: 205775-002 p. 5-10.
    Encoding::new(b"10101110", 1, States::single(7)),
    // XRI data: 205775-002 p. 5-10.
    Encoding::new(b"11101110", 2, States::single(7)),
    // ORA r: 205775-002 p. 5-10.
    Encoding::new(b"10110SSS", 1, States::single(4)),
    // ORA M: 205775-002 p. 5-11.
    Encoding::new(b"10110110", 1, States::single(7)),
    // ORI data: 205775-002 p. 5-11.
    Encoding::new(b"11110110", 2, States::single(7)),
    // CMP r: 205775-002 p. 5-11.
    Encoding::new(b"10111SSS", 1, States::single(4)),
    // CMP M: 205775-002 p. 5-11.
    Encoding::new(b"10111110", 1, States::single(7)),
    // CPI data: 205775-002 p. 5-11.
    Encoding::new(b"11111110", 2, States::single(7)),
    // RLC: 205775-002 p. 5-11.
    Encoding::new(b"00000111", 1, States::single(4)),
    // RRC: 205775-002 p. 5-12.
    Encoding::new(b"00001111", 1, States::single(4)),
    // RAL: 205775-002 p. 5-12.
    Encoding::new(b"00010111", 1, States::single(4)),
    // RAR: 205775-002 p. 5-12.
    Encoding::new(b"00011111", 1, States::single(4)),
    // CMA: 205775-002 p. 5-12.
    Encoding::new(b"00101111", 1, States::single(4)),
    // CMC: 205775-002 p. 5-12.
    Encoding::new(b"00111111", 1, States::single(4)),
    // STC: 205775-002 p. 5-12.
    Encoding::new(b"00110111", 1, States::single(4)),
];

/// Returns no path until this executor lands; touches no state or bus.
pub(super) fn execute<S: Bus>(
    _cpu: &mut I8085,
    _system: &mut S,
    _opcode: u8,
    _instruction: Instruction,
) -> Option<Path> {
    None
}
