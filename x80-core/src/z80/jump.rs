//! Jump Group: encoding data only (decision-13 rule 2).
//!
//! Primary: Zilog UM008011-0816, rev. 11 (August 2016), pp. 261–279.
//! Entry citations below use printed pages; PDF = printed + 14.
//! Independent cross-read: 03-0029-01 (1977), Table 7.0-9, p. 52
//! (PDF = printed + 6). Whole-instruction lengths and T-state totals include
//! every prefix, operand and displacement (decision-13 rules 3–5).

use super::cpu::Decoded;
use super::opcodes::{Encoding, Instruction, States};
use super::Z80;
use crate::decode::Path;
use crate::Bus;

pub(super) const MAIN: &[Encoding] = &[
    // UM JP nn, p. 262.
    Encoding::new(b"11000011", 3, States::single(10)),
    // UM JP cc,nn, pp. 263–264; both paths are 10.
    Encoding::new(b"11CCC010", 3, States::single(10)),
    // UM JR e, p. 265.
    Encoding::new(b"00011000", 2, States::single(12)),
    // UM JR C,e, p. 267.
    Encoding::new(
        b"00111000",
        2,
        States {
            untaken: 7,
            taken: 12,
        },
    ),
    // UM JR NC,e, p. 269.
    Encoding::new(
        b"00110000",
        2,
        States {
            untaken: 7,
            taken: 12,
        },
    ),
    // UM JR Z,e, p. 271.
    Encoding::new(
        b"00101000",
        2,
        States {
            untaken: 7,
            taken: 12,
        },
    ),
    // UM JR NZ,e, p. 273.
    Encoding::new(
        b"00100000",
        2,
        States {
            untaken: 7,
            taken: 12,
        },
    ),
    // UM JP (HL), p. 275.
    Encoding::new(b"11101001", 1, States::single(4)),
    // UM DJNZ e, p. 278.
    Encoding::new(
        b"00010000",
        2,
        States {
            untaken: 8,
            taken: 13,
        },
    ),
];

pub(super) const INDEX: &[Encoding] = &[
    // UM JP (IX/IY), pp. 276–277.
    Encoding::new(b"11101001", 2, States::single(8)),
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
