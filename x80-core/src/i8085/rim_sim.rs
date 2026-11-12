//! The 8085 RIM/SIM encodings and execution seam.
//!
//! Intel 205775-002 (January 1983), §5.6.5, pp. 5-17 and 5-18, checked on page images.
//! Totals below are the 8085 states, not machine cycles or 8080 states.
//! Independent witness: `opcodes/tests.rs`, Table 5-1 p. 5-19 cross-read
//! with Intel 9800301-04 (May 1981), App. A pp. A-1 to A-5.
//! Design: decision-10, decision-11; decision-5 item 7's interim path.

use super::opcodes::{Encoding, Instruction, Path, States};
use super::I8085;
use crate::Bus;

/// RIM and SIM, separated from stack/control by decision-10.
pub(super) const ENCODINGS: &[Encoding] = &[
    // RIM: 205775-002 p. 5-17.
    Encoding::new(b"00100000", 1, States::single(4)),
    // SIM: 205775-002 p. 5-18.
    Encoding::new(b"00110000", 1, States::single(4)),
];

/// Returns no path until this executor lands; touches no state or bus.
pub(super) fn execute<S: Bus + super::Inputs>(
    _cpu: &mut I8085,
    _system: &mut S,
    _opcode: u8,
    _instruction: Instruction,
) -> Option<Path> {
    None
}
