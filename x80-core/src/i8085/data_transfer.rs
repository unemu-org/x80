//! The 8085 Data Transfer encodings and execution seam.
//!
//! Intel 205775-002 (January 1983), §5.6.1, pp. 5-4 to 5-6, checked on page images.
//! Totals below are the 8085 states, not machine cycles or 8080 states.
//! Independent witness: `opcodes/tests.rs`, Table 5-1 p. 5-19 cross-read
//! with Intel 9800301-04 (May 1981), App. A pp. A-1 to A-5.
//! Design: decision-10, decision-11; decision-5 item 7's interim path.

use super::opcodes::{Encoding, Instruction, Path, States};
use super::I8085;
use crate::Bus;

/// Complete group list, in per-instruction page order.
/// LDAX/STAX permit only B and D pairs (205775-002 pp. 5-5/5-6).
/// Register-field exclusions live in `opcodes` (§5.2 p. 5-1).
pub(super) const ENCODINGS: &[Encoding] = &[
    // MOV r1,r2: 205775-002 p. 5-4.
    Encoding::new(b"01DDDSSS", 1, States::single(4)),
    // MOV r,M: 205775-002 p. 5-4.
    Encoding::new(b"01DDD110", 1, States::single(7)),
    // MOV M,r: 205775-002 p. 5-4.
    Encoding::new(b"01110SSS", 1, States::single(7)),
    // MVI r,data: 205775-002 p. 5-4.
    Encoding::new(b"00DDD110", 2, States::single(7)),
    // MVI M,data: 205775-002 p. 5-4.
    Encoding::new(b"00110110", 2, States::single(10)),
    // LXI rp,data16: 205775-002 p. 5-5.
    Encoding::new(b"00RP0001", 3, States::single(10)),
    // LDA addr: 205775-002 p. 5-5.
    Encoding::new(b"00111010", 3, States::single(13)),
    // STA addr: 205775-002 p. 5-5.
    Encoding::new(b"00110010", 3, States::single(13)),
    // LHLD addr: 205775-002 p. 5-5.
    Encoding::new(b"00101010", 3, States::single(16)),
    // SHLD addr: 205775-002 p. 5-5.
    Encoding::new(b"00100010", 3, States::single(16)),
    // LDAX B: 205775-002 p. 5-5.
    Encoding::new(b"00001010", 1, States::single(7)),
    // LDAX D: 205775-002 p. 5-5.
    Encoding::new(b"00011010", 1, States::single(7)),
    // STAX B: 205775-002 p. 5-6.
    Encoding::new(b"00000010", 1, States::single(7)),
    // STAX D: 205775-002 p. 5-6.
    Encoding::new(b"00010010", 1, States::single(7)),
    // XCHG: 205775-002 p. 5-6.
    Encoding::new(b"11101011", 1, States::single(4)),
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
