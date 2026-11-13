//! 16-Bit Load Group: encoding data only (decision-13 rule 2).
//!
//! Primary: Zilog UM008011-0816, rev. 11 (August 2016), pp. 98–122.
//! Entry citations below use printed pages; PDF = printed + 14.
//! Independent cross-read: 03-0029-01 (1977), Table 7.0-2, p. 45
//! (PDF = printed + 6). Whole-instruction lengths and T-state totals include
//! every prefix, operand and displacement (decision-13 rules 3–5).

use super::cpu::Decoded;
use super::opcodes::{Encoding, Instruction, States};
use super::Z80;
use crate::decode::Path;
use crate::Bus;

pub(super) const MAIN: &[Encoding] = &[
    // UM LD dd,nn, p. 99.
    Encoding::new(b"00PP0001", 3, States::single(10)),
    // UM LD HL,(nn), p. 102.
    Encoding::new(b"00101010", 3, States::single(16)),
    // UM LD (nn),HL, p. 107.
    Encoding::new(b"00100010", 3, States::single(16)),
    // UM LD SP,HL, p. 112.
    Encoding::new(b"11111001", 1, States::single(6)),
    // UM PUSH qq, pp. 115–116 (compact misprints: PROVENANCE §1).
    Encoding::new(b"11QQ0101", 1, States::single(11)),
    // UM POP qq, pp. 119–120 (compact misprint: PROVENANCE §1).
    Encoding::new(b"11QQ0001", 1, States::single(10)),
];

pub(super) const ED: &[Encoding] = &[
    // UM LD dd,(nn), pp. 103–104; includes HL (PROVENANCE §1).
    Encoding::new(b"01PP1011", 4, States::single(20)),
    // UM LD (nn),dd, pp. 108–109; includes HL (PROVENANCE §1).
    Encoding::new(b"01PP0011", 4, States::single(20)),
];

pub(super) const INDEX: &[Encoding] = &[
    // UM LD IX/IY,nn, pp. 100–101.
    Encoding::new(b"00100001", 4, States::single(14)),
    // UM LD IX/IY,(nn), pp. 105–106.
    Encoding::new(b"00101010", 4, States::single(20)),
    // UM LD (nn),IX/IY, pp. 110–111.
    Encoding::new(b"00100010", 4, States::single(20)),
    // UM LD SP,IX/IY, pp. 113–114.
    Encoding::new(b"11111001", 2, States::single(10)),
    // UM PUSH IX/IY, pp. 117–118 (compact misprints: PROVENANCE §1).
    Encoding::new(b"11100101", 2, States::single(15)),
    // UM POP IX/IY, pp. 121–122.
    Encoding::new(b"11100001", 2, States::single(14)),
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
