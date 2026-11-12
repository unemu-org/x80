//! The 8085 Arithmetic encodings and execution seam.
//!
//! Intel 205775-002 (January 1983), §5.6.2, pp. 5-6 to 5-9, checked on page images.
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
    // ADD r: 205775-002 p. 5-6.
    Encoding::new(b"10000SSS", 1, States::single(4)),
    // ADD M: 205775-002 p. 5-6.
    Encoding::new(b"10000110", 1, States::single(7)),
    // ADI data: 205775-002 p. 5-6.
    Encoding::new(b"11000110", 2, States::single(7)),
    // ADC r: 205775-002 p. 5-6.
    Encoding::new(b"10001SSS", 1, States::single(4)),
    // ADC M: 205775-002 p. 5-7.
    Encoding::new(b"10001110", 1, States::single(7)),
    // ACI data: 205775-002 p. 5-7.
    Encoding::new(b"11001110", 2, States::single(7)),
    // SUB r: 205775-002 p. 5-7.
    Encoding::new(b"10010SSS", 1, States::single(4)),
    // SUB M: 205775-002 p. 5-7.
    Encoding::new(b"10010110", 1, States::single(7)),
    // SUI data: 205775-002 p. 5-7.
    Encoding::new(b"11010110", 2, States::single(7)),
    // SBB r: 205775-002 p. 5-7.
    Encoding::new(b"10011SSS", 1, States::single(4)),
    // SBB M: 205775-002 p. 5-8.
    Encoding::new(b"10011110", 1, States::single(7)),
    // SBI data: 205775-002 p. 5-8.
    Encoding::new(b"11011110", 2, States::single(7)),
    // INR r: 205775-002 p. 5-8.
    Encoding::new(b"00DDD100", 1, States::single(4)),
    // INR M: 205775-002 p. 5-8.
    Encoding::new(b"00110100", 1, States::single(10)),
    // DCR r: 205775-002 p. 5-8.
    Encoding::new(b"00DDD101", 1, States::single(4)),
    // DCR M: 205775-002 p. 5-8.
    Encoding::new(b"00110101", 1, States::single(10)),
    // INX rp: 205775-002 p. 5-9.
    Encoding::new(b"00RP0011", 1, States::single(6)),
    // DCX rp: 205775-002 p. 5-9.
    Encoding::new(b"00RP1011", 1, States::single(6)),
    // DAD rp: 205775-002 p. 5-9.
    Encoding::new(b"00RP1001", 1, States::single(10)),
    // DAA: 205775-002 p. 5-9.
    Encoding::new(b"00100111", 1, States::single(4)),
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
