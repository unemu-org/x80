//! 16-Bit Arithmetic Group: encoding data only (decision-13 rule 2).
//!
//! Primary: Zilog UM008011-0816, rev. 11 (August 2016), pp. 187–203.
//! Entry citations below use printed pages; PDF = printed + 14.
//! Independent cross-read: 03-0029-01 (1977), Table 7.0-6, p. 49
//! (PDF = printed + 6). Whole-instruction lengths and T-state totals include
//! every prefix, operand and displacement (decision-13 rules 3–5).

use super::cpu::Decoded;
use super::opcodes::{Encoding, Instruction, States};
use super::Z80;
use crate::decode::Path;
use crate::Bus;

pub(super) const MAIN: &[Encoding] = &[
    // UM ADD HL,ss, p. 188.
    Encoding::new(b"00PP1001", 1, States::single(11)),
    // UM INC ss, p. 198.
    Encoding::new(b"00PP0011", 1, States::single(6)),
    // UM DEC ss, p. 201 (compact misprint: PROVENANCE §1).
    Encoding::new(b"00PP1011", 1, States::single(6)),
];

pub(super) const ED: &[Encoding] = &[
    // UM ADC HL,ss, p. 190.
    Encoding::new(b"01PP1010", 2, States::single(15)),
    // UM SBC HL,ss, p. 192.
    Encoding::new(b"01PP0010", 2, States::single(15)),
];

pub(super) const INDEX: &[Encoding] = &[
    // UM ADD IX,pp / IY,rr, pp. 194, 196.
    Encoding::new(b"00PP1001", 2, States::single(15)),
    // UM INC IX/IY, pp. 199–200.
    Encoding::new(b"00100011", 2, States::single(10)),
    // UM DEC IX/IY, pp. 202–203.
    Encoding::new(b"00101011", 2, States::single(10)),
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
