//! The 8085 Branch encodings and execution seam.
//!
//! Intel 205775-002 (January 1983), §5.6.4, pp. 5-13 to 5-15, checked on page images.
//! Totals below are the 8085 states, not machine cycles or 8080 states.
//! Independent witness: `opcodes/tests.rs`, Table 5-1 p. 5-19 cross-read
//! with Intel 9800301-04 (May 1981), App. A pp. A-1 to A-5.
//! Design: decision-10, decision-11; decision-5 item 7's interim path.

use super::opcodes::{Encoding, Instruction, Path, States};
use super::I8085;
use crate::Bus;

/// Complete group list, in per-instruction page order.
/// CCC takes all eight conditions (205775-002 p. 5-13); NNN takes
/// all eight restarts (p. 5-14). Costs are untaken/taken respectively;
/// 9800301-04 p. A-1 explains that order. Its CPE 9/17 disagrees with
/// this primary source's 9/18, pinned in `opcodes/tests.rs`.
pub(super) const ENCODINGS: &[Encoding] = &[
    // JMP addr: 205775-002 p. 5-13.
    Encoding::new(b"11000011", 3, States::single(10)),
    // Jcondition addr: 205775-002 p. 5-13.
    Encoding::new(
        b"11CCC010",
        3,
        States {
            untaken: 7,
            taken: 10,
        },
    ),
    // CALL addr: 205775-002 p. 5-13.
    Encoding::new(b"11001101", 3, States::single(18)),
    // Ccondition addr: 205775-002 p. 5-14.
    Encoding::new(
        b"11CCC100",
        3,
        States {
            untaken: 9,
            taken: 18,
        },
    ),
    // RET: 205775-002 p. 5-14.
    Encoding::new(b"11001001", 1, States::single(10)),
    // Rcondition: 205775-002 p. 5-14.
    Encoding::new(
        b"11CCC000",
        1,
        States {
            untaken: 6,
            taken: 12,
        },
    ),
    // RST n: 205775-002 p. 5-14.
    Encoding::new(b"11NNN111", 1, States::single(12)),
    // PCHL: 205775-002 p. 5-15.
    Encoding::new(b"11101001", 1, States::single(6)),
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
