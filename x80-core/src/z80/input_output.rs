//! Input and Output Group: encoding data only (decision-13 rule 2).
//!
//! Primary: Zilog UM008011-0816, rev. 11 (August 2016), pp. 294–316.
//! Entry citations below use printed pages; PDF = printed + 14.
//! Independent cross-read: 03-0029-01 (1977), Table 7.0-11, p. 54
//! (PDF = printed + 6). Whole-instruction lengths and T-state totals include
//! every prefix, operand and displacement (decision-13 rules 3–5).

use super::cpu::Decoded;
use super::opcodes::{Encoding, Instruction, States};
use super::Z80;
use crate::decode::Path;
use crate::Bus;

// Repeats use terminating/repeating slots (decision-15 rule 11).

pub(super) const MAIN: &[Encoding] = &[
    // UM IN A,(n), p. 295.
    Encoding::new(b"11011011", 2, States::single(11)),
    // UM OUT (n),A, p. 306.
    Encoding::new(b"11010011", 2, States::single(11)),
];

pub(super) const ED: &[Encoding] = &[
    // UM IN r,(C), pp. 296–297; r=110 excluded (decision-14).
    Encoding::new(b"01RRR000", 2, States::single(12)),
    // UM OUT (C),r, pp. 307–308; r=110 absent.
    Encoding::new(b"01RRR001", 2, States::single(12)),
    // UM INI, p. 298.
    Encoding::new(b"10100010", 2, States::single(16)),
    // UM INIR, p. 300.
    Encoding::new(
        b"10110010",
        2,
        States {
            untaken: 16,
            taken: 21,
        },
    ),
    // UM IND, p. 302.
    Encoding::new(b"10101010", 2, States::single(16)),
    // UM INDR, p. 304.
    Encoding::new(
        b"10111010",
        2,
        States {
            untaken: 16,
            taken: 21,
        },
    ),
    // UM OUTI, p. 309.
    Encoding::new(b"10100011", 2, States::single(16)),
    // UM OTIR, pp. 311–312.
    Encoding::new(
        b"10110011",
        2,
        States {
            untaken: 16,
            taken: 21,
        },
    ),
    // UM OUTD, p. 313.
    Encoding::new(b"10101011", 2, States::single(16)),
    // UM OTDR, pp. 315–316.
    Encoding::new(
        b"10111011",
        2,
        States {
            untaken: 16,
            taken: 21,
        },
    ),
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
