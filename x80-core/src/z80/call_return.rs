//! Call and Return Group: encoding data only (decision-13 rule 2).
//!
//! Primary: Zilog UM008011-0816, rev. 11 (August 2016), pp. 280–293.
//! Entry citations below use printed pages; PDF = printed + 14.
//! Independent cross-read: 03-0029-01 (1977), Table 7.0-10, p. 53
//! (PDF = printed + 6). Whole-instruction lengths and T-state totals include
//! every prefix, operand and displacement (decision-13 rules 3–5).

use super::cpu::Decoded;
use super::opcodes::{Encoding, Instruction, States};
use super::Z80;
use crate::decode::Path;
use crate::Bus;

pub(super) const MAIN: &[Encoding] = &[
    // UM CALL nn, p. 281.
    Encoding::new(b"11001101", 3, States::single(17)),
    // UM CALL cc,nn, pp. 283–284.
    Encoding::new(
        b"11CCC100",
        3,
        States {
            untaken: 10,
            taken: 17,
        },
    ),
    // UM RET, p. 285.
    Encoding::new(b"11001001", 1, States::single(10)),
    // UM RET cc, pp. 286–287.
    Encoding::new(
        b"11CCC000",
        1,
        States {
            untaken: 5,
            taken: 11,
        },
    ),
    // UM RST p, pp. 292–293.
    Encoding::new(b"11TTT111", 1, States::single(11)),
];

pub(super) const ED: &[Encoding] = &[
    // UM RETI, p. 288.
    Encoding::new(b"01001101", 2, States::single(14)),
    // UM RETN, p. 290.
    Encoding::new(b"01000101", 2, States::single(14)),
];

/// Flip only this group-owned switch when its executor is complete. Until
/// then dispatch refuses before any operand/data/stack/port access
/// (decision-5 item 7 staging; decision-14 rule 4).
pub(super) const LANDED: bool = false;

/// Final executor seam (decision-13 rule 4). PC is after prefix/key bytes;
/// read remaining operands through Z80::fetch_operand. INDEX d is executor-
/// owned; INDEX_CB d is in Decoded. Return the complete instruction's path.
/// Repeating blocks rewind PC by two and return Path::Taken (rule 11).
pub(super) fn execute<S: Bus>(
    _cpu: &mut Z80,
    _system: &mut S,
    _instruction: &Instruction,
    _decoded: &Decoded,
) -> Path {
    // Unreachable while LANDED is false; never a panic or a public stub.
    Path::Untaken
}
