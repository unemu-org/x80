//! Bit Set, Reset, and Test Group: encoding data only (decision-13 rule 2).
//!
//! Primary: Zilog UM008011-0816, rev. 11 (August 2016), pp. 242–260.
//! Entry citations below use printed pages; PDF = printed + 14.
//! Independent cross-read: 03-0029-01 (1977), Table 7.0-8, p. 51
//! (PDF = printed + 6). Whole-instruction lengths and T-state totals include
//! every prefix, operand and displacement (decision-13 rules 3–5).

use super::cpu::Decoded;
use super::opcodes::{Encoding, Instruction, States};
use super::Z80;
use crate::decode::Path;
use crate::Bus;

// Compact Table 14 is corrupt: entry patterns prevail (PROVENANCE.md §1).

pub(super) const CB: &[Encoding] = &[
    // UM BIT b,r, pp. 243–244.
    Encoding::new(b"01BBBRRR", 2, States::single(8)),
    // UM BIT b,(HL), pp. 245–246.
    Encoding::new(b"01BBB110", 2, States::single(12)),
    // UM SET b,r, pp. 251–252.
    Encoding::new(b"11BBBRRR", 2, States::single(8)),
    // UM SET b,(HL), pp. 253–254.
    Encoding::new(b"11BBB110", 2, States::single(15)),
    // UM RES b,r, pp. 259–260.
    Encoding::new(b"10BBBRRR", 2, States::single(8)),
    // UM RES b,(HL), pp. 259–260.
    Encoding::new(b"10BBB110", 2, States::single(15)),
];

pub(super) const INDEX_CB: &[Encoding] = &[
    // UM BIT b,(IX/IY+d), pp. 247–250.
    Encoding::new(b"01BBB110", 4, States::single(20)),
    // UM SET b,(IX/IY+d), pp. 255–258.
    Encoding::new(b"11BBB110", 4, States::single(23)),
    // UM RES b,(IX/IY+d), pp. 259–260.
    Encoding::new(b"10BBB110", 4, States::single(23)),
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
